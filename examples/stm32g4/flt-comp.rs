#![no_std]
#![no_main]

/// Example showcasing the use of the HRTIM peripheral together with a comparator to implement a current fault.
/// Once the comparator input exceeds the reference set by the DAC, the output is forced low and put into a fault state.
use cortex_m_rt::entry;
use panic_probe as _;
use stm32_hrtim::{
    compare_register::HrCompareRegister,
    control::HrControltExt,
    fault::{FaultAction, FaultMonitor},
    output::HrOutput,
    timer::HrTimer,
    HrParts, HrPwmAdvExt, Pscl4,
};
use stm32g4xx_hal::{
    self as hal,
    adc::AdcClaim,
    comparator::{self, ComparatorExt, ComparatorSplit},
    dac::{Dac3IntSig1, DacExt, DacOut},
    delay::SYSTDelayExt,
    gpio::GpioExt,
    pwr::PwrExt,
    rcc::{self, RccExt},
    stm32::{CorePeripherals, Peripherals},
};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().expect("cannot take peripherals");
    let cp = CorePeripherals::take().expect("cannot take core");
    // Set system frequency to 16MHz * 15/1/2 = 120MHz
    // This would lead to HrTim running at 120MHz * 32 = 3.84GHz...
    let pwr = dp.PWR.constrain().freeze();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::pll().pll_cfg(rcc::PllConfig {
            mux: rcc::PllSrc::HSI,
            n: rcc::PllNMul::MUL_15,
            m: rcc::PllMDiv::DIV_1,
            r: Some(rcc::PllRDiv::DIV_2),
            ..Default::default()
        }),
        pwr,
    );

    let mut delay = cp.SYST.delay(&rcc.clocks);

    let mut adc1 = dp.ADC1.claim_and_configure(
        hal::adc::ClockSource::SystemClock,
        &rcc,
        hal::adc::config::AdcConfig::default()
            .clock_mode(hal::adc::config::ClockMode::Synchronous_Div_4),
        &mut delay,
        false,
    );

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    let dac3ch1 = dp.DAC3.constrain(Dac3IntSig1, &mut rcc);
    let mut dac = dac3ch1.enable();

    // Use dac to define the fault threshold
    // 2^12 / 2 = 2^11 for about half of VCC
    let fault_limit = 60;
    dac.set_value(fault_limit);

    let (_comp1, _comp2, comp3, ..) = dp.COMP.split(&mut rcc);

    let pc1 = gpioc.pc1.into_analog();
    let comp3 = comp3
        .comparator(
            &pc1,
            &dac,
            comparator::Config::default()
                .hysteresis(comparator::Hysteresis::None)
                .output_inverted(),
            &rcc.clocks,
        )
        .enable();

    let (hr_control, flt_inputs, _) = dp.HRTIM_COMMON.hr_control(&mut rcc).wait_for_calibration();
    let mut hr_control = hr_control.constrain();

    let fault_source5 = flt_inputs
        .fault_input5
        .bind_comp(&comp3)
        .polarity(hal::pwm::Polarity::ActiveHigh)
        .finalize(&mut hr_control);

    // ...with a prescaler of 4 this gives us a HrTimer with a tick rate of 1.2GHz
    // With max the max period set, this would be 1.2GHz/2^16 ~= 18kHz...
    let prescaler = Pscl4;

    let pin_a = gpioa.pa8;

    //        .               .               .  *
    //        .  33%          .               .  *            .               .
    //        .-----.         .-----.         .--.            .               .
    //out1    |     |         |     |         |  |            .               .
    //        |     |         |     |         |  |            .               .
    //   ------     -----------     -----------  -----------------------------------
    //        .               .               .  *            .               .
    //        .               .               .  *            .               .
    //        .               .               .  *--------    .               .
    //fault   .               .               .  |        |   .               .
    //        .               .               .  |        |   .               .
    //   -----------------------------------------        --------------------------
    //        .               .               .  *            .               .
    //        .               .               .  *            .               .
    let HrParts {
        mut timer,
        mut cr1,
        out: mut out1,
        ..
    } = dp
        .HRTIM_TIMA
        .pwm_advanced(pin_a, &mut rcc)
        .prescaler(prescaler)
        .period(0xFFFF)
        .with_fault_source(fault_source5) // Set fault source
        .fault_action1(FaultAction::ForceInactive)
        .fault_action2(FaultAction::ForceInactive)
        .finalize(&mut hr_control);

    out1.enable_rst_event(&cr1); // Set low on compare match with cr1
    out1.enable_set_event(&timer); // Set high at new period
    cr1.set_duty(timer.get_period() / 3);
    //unsafe {((HRTIM_COMMON::ptr() as *mut u8).offset(0x14) as *mut u32).write_volatile(1); }
    out1.enable();
    timer.start(&mut hr_control.control);

    defmt::info!("Started");

    loop {
        for _ in 0..5 {
            //delay.delay(500_u32.millis());
            defmt::info!(
                "State: {:?}, comp: {}, is_fault_active: {}, pc1: {}",
                out1.get_state(),
                comp3.output(),
                hr_control.fault_5.is_fault_active(),
                adc1.convert(&pc1, hal::adc::config::SampleTime::Cycles_92_5)
            );
        }
        if hr_control.fault_5.is_fault_active() {
            hr_control.fault_5.clear_fault(); // Clear fault every 5s
            out1.enable();
            defmt::info!("failt cleared, and output reenabled");
        }
    }
}
