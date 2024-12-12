pub use hal::stm32;
pub use stm32h7xx_hal as hal;

// TODO: H7 does not start in input mode, it starts in Analog
pub type GpioInputMode = hal::gpio::Input;

pub use hal::pwm::Alignment;

#[cfg(feature = "stm32h7")]
pub use hal::pwm::Polarity;
