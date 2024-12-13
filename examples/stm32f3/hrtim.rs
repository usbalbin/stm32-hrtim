#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;
use stm32_hrtim::{
    compare_register::HrCompareRegister, control::HrControltExt, output::HrOutput, timer::HrTimer,
    HrParts, HrPwmAdvExt, Pscl4,
};
use stm32f3xx_hal::{
    delay::Delay,
    flash::FlashExt as _,
    gpio::GpioExt,
    pac::{CorePeripherals, Peripherals},
    prelude::{_embedded_hal_blocking_delay_DelayMs, _stm32f3xx_hal_time_rate_Extensions},
    rcc::RccExt,
};

#[entry]
fn main() -> ! {
    defmt::info!("Initializing...");

    let dp = Peripherals::take().expect("cannot take peripherals");
    let cp = CorePeripherals::take().expect("cannot take core");

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    //let mut rcc = Input to hrtim needs to be 128MHz when using HSI, 128-144 with HSE

    // Set system frequency to 64MHz using PLL, PLLCLKx2 will thus be 128MHz which
    // feeds into the HRTIM. This and the HRTIM's DLL would lead to an effective
    // HRTIM frequency of 128MHz * 32 = 4.096GHz...
    let clocks = rcc
        .cfgr
        .sysclk(64_u32.MHz())
        .use_pll()
        .freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

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
        .hr_control(&clocks, &mut rcc.apb2)
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
        .finalize(
            &mut hr_control,
            &mut gpioa.moder,
            &mut gpioa.otyper,
            &mut gpioa.afrh,
        );

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

            delay.delay_ms(500_u16);
        }
    }
}
