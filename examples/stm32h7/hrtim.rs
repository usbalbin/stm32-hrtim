#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;
use stm32_hrtim::{
    compare_register::HrCompareRegister, control::HrControltExt, output::HrOutput, timer::HrTimer,
    HrParts, HrPwmAdvExt, Pscl1,
};
use stm32h7xx_hal::{
    delay::DelayExt,
    gpio::GpioExt,
    prelude::_embedded_hal_blocking_delay_DelayMs,
    pwr::PwrExt,
    rcc::RccExt,
    stm32::{CorePeripherals, Peripherals},
};

use fugit::RateExtU32 as _;

#[entry]
fn main() -> ! {
    defmt::info!("Initializing...");

    let dp = Peripherals::take().expect("cannot take peripherals");
    let cp = CorePeripherals::take().expect("cannot take core");

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Constrain and Freeze clock
    let rcc = dp.RCC.constrain();

    // With a sys_ck of 240MHz and d1cpre of 1 if the HRTIM will be fed by 240MHz/1 = 240MHz
    // since HRTIMSEL is set to take the HRTIM's clock directly from the core clock. The
    // stm32h7 devices' HRTIM does not have a DLL, also leading to an effective HRTIM
    // frequency of 240MHz...
    let ccdr = rcc
        .sys_ck(240.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // Acquire the GPIO peripherals. This also enables the clock for
    // the GPIOs in the RCC register.
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // ...with a prescaler of 1 this gives us a HrTimer with a tick rate of 240MHz
    // With max the max period set, this would be 240MHz/2^16 ~= 3.7kHz...
    let prescaler = Pscl1;

    let pin_a = gpioc.pc6.into_input();
    let pin_b = gpioc.pc7.into_input();

    //        .               .               .               .
    //        .  30%          .               .               .
    //         ----           .               .----           .
    //out1    |    |          .               |    |          .
    //        |    |          .               |    |          .
    // --------    ----------------------------    --------------------
    //        .               .----           .               .----
    //out2    .               |    |          .               |    |
    //        .               |    |          .               |    |
    // ------------------------    ----------------------------    ----
    //        .               .               .               .
    //        .               .               .               .
    let (hr_control, ..) = dp
        .HRTIM_COMMON
        .hr_control(&ccdr.clocks, ccdr.peripheral.HRTIM)
        .wait_for_calibration();
    let mut hr_control = hr_control.constrain();

    let HrParts {
        mut timer,
        mut cr1,
        out: (mut out1, mut out2),
        ..
    } = dp
        .HRTIM_TIMA
        .pwm_advanced((pin_a, pin_b))
        .prescaler(prescaler)
        .period(0xFFFF)
        .push_pull_mode(true) // Set push pull mode, out1 and out2 are
        // alternated every period with one being
        // inactive and the other getting to output its wave form
        // as normal
        .finalize(&mut hr_control);

    out1.enable_rst_event(&cr1); // Set low on compare match with cr1
    out2.enable_rst_event(&cr1);

    out1.enable_set_event(&timer); // Set high at new period
    out2.enable_set_event(&timer);

    out1.enable();
    out2.enable();

    loop {
        // Step frequency from 3.7kHz to about 36.6kHz(half of that when only looking at one pin)
        for i in 1..10 {
            let new_period = u16::MAX / i;

            cr1.set_duty(new_period / 3);
            timer.set_period(new_period);

            delay.delay_ms(500_u16);
        }
    }
}
