
pub use stm32f3xx_hal as hal;
//pub use hal::pac as stm32;
pub use stm32f3::stm32f3x4 as stm32;

#[allow(non_camel_case_types, dead_code)]
pub enum DmaMuxResources {
    
}

pub type GpioInputMode = hal::gpio::Input;

pub enum Alignment { Left }
#[cfg(feature = "stm32f3")]
pub enum Polarity {
    ActiveHigh,
    ActiveLow,
}