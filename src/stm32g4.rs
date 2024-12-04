pub use hal::stm32;
pub use stm32g4xx_hal as hal;

#[allow(non_camel_case_types, dead_code)]
pub enum DmaMuxResources {
    HRTIM_MASTER = 95,
    HRTIM_TIMA = 96,
    HRTIM_TIMB = 97,
    HRTIM_TIMC = 98,
    HRTIM_TIMD = 99,
    HRTIM_TIME = 100,
    HRTIM_TIMF = 101,
}

pub type GpioInputMode = hal::gpio::Input<hal::gpio::Floating>;

pub use hal::pwm::Alignment;

#[cfg(feature = "stm32g4")]
pub use hal::pwm::Polarity;
