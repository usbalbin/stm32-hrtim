#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;
use stm32_hrtim::{
    compare_register::HrCompareRegister, control::HrControltExt, output::HrOutput, timer::HrTimer,
    HrParts, HrPwmAdvExt, Pscl4,
};
use stm32h7xx_hal::{
    delay::DelayExt,
    gpio::GpioExt,
    pwr::PwrExt,
    rcc::{self, RccExt},
    stm32::{CorePeripherals, Peripherals},
};

use fugit::{ExtU32, RateExtU32 as _};

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
    let ccdr = rcc.sys_ck(320.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    // Acquire the GPIO peripherals. This also enables the clock for
    // the GPIOs in the RCC register.
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // ...with a prescaler of 4 this gives us a HrTimer with a tick rate of 960MHz
    // With max the max period set, this would be 960MHz/2^16 ~= 15kHz...
    let prescaler = Pscl4;

    let pin_a = gpioa.pa8;
    let pin_b = gpioa.pa9;

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
        .hr_control(ccdr.peripheral.HRTIM)
        .wait_for_calibration();
    let mut hr_control = hr_control.constrain();

    let HrParts {
        mut timer,
        mut cr1,
        out: (mut out1, mut out2),
        ..
    } = dp
        .HRTIM_TIMA
        .pwm_advanced((pin_a, pin_b), &mut rcc)
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
        // Step frequency from 18kHz to about 180kHz(half of that when only looking at one pin)
        for i in 1..10 {
            let new_period = u16::MAX / i;

            cr1.set_duty(new_period / 3);
            timer.set_period(new_period);

            delay.delay(500_u32.millis());
        }
    }
}
