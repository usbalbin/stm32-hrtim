use crate::pac;
use core::marker::PhantomData;

/// Handle to timers reset/roll-over event
pub struct TimerReset<T>(pub(crate) PhantomData<T>);

/// Handle to timers period event
pub struct TimerPeriod<T>(pub(crate) PhantomData<T>);

macro_rules! impl_adc1234_trigger {
    ($($t:ident: [$trait_:ident, $adcXr:ident]),*) => {$(
        #[non_exhaustive]
        pub struct $t;

        impl $t {
            pub fn enable_source<T: $trait_>(&mut self, _trigger: &T) {
                let common = unsafe { &*pac::HRTIM_COMMON::ptr() };
                unsafe {
                    common.$adcXr().modify(|r, w| w.bits(r.bits() | T::BITS));
                }
            }
        }
    )*}
}

#[cfg(feature = "hrtim_v2")]
macro_rules! impl_adc5678910_trigger {
    ($($t:ident: [$trait_:ident, $adcXtrg:ident]),*) => {$(
        #[non_exhaustive]
        pub struct $t;

        impl $t {
            pub fn enable_source<T: $trait_>(&mut self, _trigger: &T) {
                let common = unsafe { &*pac::HRTIM_COMMON::ptr() };
                common
                    .adcer()
                    .modify(|_r, w| unsafe { w.$adcXtrg().bits(T::BITS as u8) });
            }
        }
    )*}
}

pub trait AdcTrigger13 {
    const BITS: u32;
}

pub trait AdcTrigger24 {
    const BITS: u32;
}

pub trait AdcTrigger579 {
    const BITS: u32;
}

pub trait AdcTrigger6810 {
    const BITS: u32;
}

impl_adc1234_trigger! {
    AdcTrigger1: [AdcTrigger13, adc1r],
    AdcTrigger2: [AdcTrigger24, adc2r],
    AdcTrigger3: [AdcTrigger13, adc3r],
    AdcTrigger4: [AdcTrigger24, adc4r]
}

#[cfg(feature = "hrtim_v2")]
impl_adc5678910_trigger! {
    AdcTrigger5: [AdcTrigger579,   adc5trg],
    AdcTrigger6: [AdcTrigger6810,  adc6trg],
    AdcTrigger7: [AdcTrigger579,   adc7trg],
    AdcTrigger8: [AdcTrigger6810,  adc8trg],
    AdcTrigger9: [AdcTrigger579,   adc9trg],
    AdcTrigger10: [AdcTrigger6810, adc10trg]
}
