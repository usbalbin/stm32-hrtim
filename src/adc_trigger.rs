use core::marker::PhantomData;

#[cfg(feature = "stm32g4")]
pub trait Adc13Trigger {
    const BITS: u32;
}

#[cfg(feature = "stm32g4")]
pub trait Adc24Trigger {
    const BITS: u32;
}

#[cfg(feature = "stm32g4")]
pub trait Adc579Trigger {
    const BITS: u32;
}

#[cfg(feature = "stm32g4")]
pub trait Adc6810Trigger {
    const BITS: u32;
}

/// Handle to timers reset/roll-over event
pub struct TimerReset<T>(pub(crate) PhantomData<T>);

/// Handle to timers period event
pub struct TimerPeriod<T>(pub(crate) PhantomData<T>);
