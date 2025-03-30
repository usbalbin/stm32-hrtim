#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use core::ops::FnMut;
use core::result::Result;
use fugit::{ExtU32, HertzU32, MicrosDurationU32};
use hal::stm32;
use stm32g4xx_hal as hal;

pub const F_SYS: HertzU32 = HertzU32::MHz(120);
pub const CYCLES_PER_US: u32 = F_SYS.raw() / 1_000_000;

pub fn enable_timer(cp: &mut stm32::CorePeripherals) {
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();
}

pub fn now() -> MicrosDurationU32 {
    (stm32::DWT::cycle_count() / CYCLES_PER_US).micros()
}

#[defmt_test::tests]
mod tests {
    use defmt::debug;
    use stm32_hrtim::{
        compare_register::HrCompareRegister, control::HrControltExt, output::HrOutput,
        stm32::Peripherals, timer::HrTimer, HrParts, HrPwmAdvExt, Pscl64,
    };
    use stm32g4xx_hal::{
        delay::SYSTDelayExt,
        gpio::GpioExt,
        pwr::PwrExt,
        rcc::{self, RccExt},
        stm32::GPIOA,
    };

    #[test]
    fn simple() {
        use super::*;

        let dp = Peripherals::take().expect("cannot take peripherals");
        let mut cp = stm32::CorePeripherals::take().expect("cannot take core");
        enable_timer(&mut cp);
        // Set system frequency to 16MHz * 15/1/2 = 120MHz
        // This would lead to HrTim running at 120MHz * 32 = 3.84...
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

        assert_eq!(rcc.clocks.sys_clk, F_SYS);

        // ...with a prescaler of 64 this gives us a HrTimer with a tick rate of 60MHz
        // With a period of 60_000, this would be 60MHz/60_000 = 1kHz...
        let prescaler = Pscl64;

        let gpioa = dp.GPIOA.split(&mut rcc);
        let pin_a = gpioa.pa8;

        let (hr_control, ..) = dp.HRTIM_COMMON.hr_control(&mut rcc).wait_for_calibration();
        let mut hr_control = hr_control.constrain();

        // 1kHz
        let period = 60_000;
        let HrParts {
            mut timer,
            mut cr1,
            out: mut out1,
            ..
        } = dp
            .HRTIM_TIMA
            .pwm_advanced(pin_a)
            .prescaler(prescaler)
            .period(period)
            .finalize(&mut hr_control);

        out1.enable_rst_event(&cr1); // Set low on compare match with cr1
        out1.enable_set_event(&timer); // Set high at new period
        out1.enable();
        cr1.set_duty(period / 2); // 50% duty
        timer.start(&mut hr_control.control);

        let gpioa = unsafe { &*GPIOA::PTR };

        let min: MicrosDurationU32 = 498u32.micros();
        let max: MicrosDurationU32 = 502u32.micros();

        delay.delay_ms(1); // Whait one period for the Hrtim to get started

        debug!("Awaiting first rising edge...");
        let duration_until_lo = await_lo(&gpioa, max).unwrap();
        let first_lo_duration = await_hi(&gpioa, max).unwrap();

        let mut hi_duration = 0.micros();
        let mut lo_duration = 0.micros();

        for _ in 0..10 {
            // Make sure the timer half periods are within 495-505us

            hi_duration = await_lo(&gpioa, max).unwrap();
            assert!(
                hi_duration > min && hi_duration < max,
                "hi: {} < {} < {}",
                min,
                hi_duration,
                max
            );

            lo_duration = await_hi(&gpioa, max).unwrap();
            assert!(
                lo_duration > min && lo_duration < max,
                "lo: {} < {} < {}",
                min,
                lo_duration,
                max
            );
        }

        // Prints deferred until here to not mess up timing
        debug!("Waited ~{} until low", duration_until_lo);
        debug!("First low half period: {}", first_lo_duration);

        debug!("High half period: {}", hi_duration);
        debug!("Low half period: {}", lo_duration);

        debug!("Done!");
    }
}

fn is_pax_low(gpioa: &stm32::gpioa::RegisterBlock, x: u8) -> bool {
    gpioa.idr().read().idr(x).is_low()
}

#[derive(Debug, defmt::Format)]
struct ErrorTimedOut;

fn await_lo(
    gpioa: &stm32::gpioa::RegisterBlock,
    timeout: MicrosDurationU32,
) -> Result<MicrosDurationU32, ErrorTimedOut> {
    await_p(|| is_pax_low(gpioa, 8), timeout)
}

fn await_hi(
    gpioa: &stm32::gpioa::RegisterBlock,
    timeout: MicrosDurationU32,
) -> Result<MicrosDurationU32, ErrorTimedOut> {
    await_p(|| !is_pax_low(gpioa, 8), timeout)
}

fn await_p(
    mut p: impl FnMut() -> bool,
    timeout: MicrosDurationU32,
) -> Result<MicrosDurationU32, ErrorTimedOut> {
    let before = now();

    loop {
        let passed_time = now() - before;
        if p() {
            return Ok(passed_time);
        }
        if passed_time > timeout {
            return Err(ErrorTimedOut);
        }
    }
}
