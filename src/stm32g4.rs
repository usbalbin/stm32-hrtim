
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