pub use hal::stm32;
pub use stm32h7xx_hal as hal;

pub type GpioInputMode = hal::gpio::Input;

pub use hal::pwm::Alignment;

#[cfg(feature = "stm32h7")]
pub use hal::pwm::Polarity;
