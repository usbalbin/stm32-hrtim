#[cfg(feature = "hrtim_v2")]
use crate::pac::HRTIM_TIMF;
use crate::{
    pac::{HRTIM_MASTER, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME},
    DacResetTrigger, NoDacTrigger,
};
use core::{marker::PhantomData, ops::Deref};

pub use super::ext::{Chan, Cmp};
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SwapPins {
    Normal,
    Swapped,
}

use super::{
    capture::{self, HrCapt, HrCapture},
    control::HrPwmCtrl,
    ext::{MasterDierW, MasterExt, MasterIcr, TimExt},
    HrtimPrescaler,
};

pub struct HrTim<TIM, PSCL, CPT1, CPT2, DacRst: DacResetTrigger = NoDacTrigger> {
    _timer: PhantomData<TIM>,
    _prescaler: PhantomData<PSCL>,
    _dac_trg: PhantomData<DacRst>,
    capture_ch1: CPT1,
    capture_ch2: CPT2,
}

pub struct Ch1;
pub struct Ch2;

pub trait ChExt {
    const CH: Chan;
}

impl ChExt for Ch1 {
    const CH: Chan = Chan::Ch1;
}

impl ChExt for Ch2 {
    const CH: Chan = Chan::Ch2;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Timer {
    Master,
    Tim(TimX),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimX {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    #[cfg(feature = "hrtim_v2")]
    F = 5,
}

/// Common trait for HRTIM_MASTER or any of HRTIM_TIMx peripherals
pub trait Instance: Deref<Target = Self::RB> {
    type RB: MasterExt;
    const TIMX: Timer;
    fn ptr() -> *const Self::RB;
}

/// Common trait for HRTIM_TIMx peripherals
pub trait InstanceX: Instance<RB: TimExt> {
    const T_X: TimX;
}

impl Instance for HRTIM_MASTER {
    type RB = crate::pac::hrtim_master::RegisterBlock;
    const TIMX: Timer = Timer::Master;
    fn ptr() -> *const Self::RB {
        Self::ptr()
    }
}

macro_rules! hrtim_timer {
    ($(
        $TIMX:ident:
        $timx:ident,
        $T:ident,
    )+) => {$(
        impl Instance for $TIMX {
            type RB = crate::pac::$timx::RegisterBlock;
            const TIMX: Timer = Timer::Tim(Self::T_X);
            fn ptr() -> *const Self::RB {
                Self::ptr()
            }
        }
        impl InstanceX for $TIMX {
            const T_X: TimX = TimX::$T;
        }

    )+}
}

hrtim_timer! {
    HRTIM_TIMA: hrtim_tima, A,
    HRTIM_TIMB: hrtim_timb, B,
    HRTIM_TIMC: hrtim_timc, C,
    HRTIM_TIMD: hrtim_timd, D,
    HRTIM_TIME: hrtim_time, E,
}

#[cfg(feature = "hrtim_v2")]
hrtim_timer! {HRTIM_TIMF: hrtim_timf, F,}

/// This is the DMA channel of a HRTIM timer
///
/// Every HRTIM timer including the master timer has a DMA channel
pub struct DmaChannel<TIM> {
    _x: PhantomData<TIM>,
}

pub trait HrTimer {
    type Timer: Instance;
    type Prescaler: HrtimPrescaler;
    type DacResetTrigger: DacResetTrigger;

    /// Get period of timer in number of ticks
    ///
    /// This is also the maximum duty usable for `HrCompareRegister::set_duty`
    ///
    /// NOTE: The effective period in number of ticks will be twice as large as
    /// returned by this function when running in UpDown mode or PushPull mode.
    /// 4 times as large when having both modes active
    fn get_period(&self) -> u16;

    /// Set period of timer in number of ticks
    ///
    /// NOTE: This will affect the maximum duty usable for `HrCompareRegister::set_duty`
    fn set_period(&mut self, period: u16);

    /// Get the current counter value
    ///
    /// NOTE: The least significant bits may not be significant depending on prescaler
    fn get_counter_value(&self) -> u16;

    /// Start timer
    fn start(&mut self, _hr_control: &mut HrPwmCtrl);

    /// Stop timer
    fn stop(&mut self, _hr_control: &mut HrPwmCtrl);

    /// Stop timer and reset counter
    fn stop_and_reset(&mut self, _hr_control: &mut HrPwmCtrl);

    fn clear_repetition_interrupt(&mut self);

    /// Make a handle to this timers reset/roll-over event to use as adc trigger
    fn as_reset_adc_trigger(&self) -> super::adc_trigger::TimerReset<Self::Timer>;

    /// Make a handle to this timers period event to use as adc trigger
    fn as_period_adc_trigger(&self) -> super::adc_trigger::TimerPeriod<Self::Timer>;

    /// Disable register updates
    ///
    /// Calling this function temporarily disables the transfer from preload to active registers,
    /// whatever the selected update event. This allows to modify several registers.
    /// The regular update event takes place once [`Self::enable_register_updates`] is called.
    fn disable_register_updates(&mut self, _hr_control: &mut HrPwmCtrl);

    /// Enable register updates
    ///
    /// See [`Self::disable_register_updates`].
    ///
    /// NOTE: Register updates are enabled by default, no need to call this
    /// unless [`Self::disable_register_updates`] has been called.
    fn enable_register_updates(&mut self, _hr_control: &mut HrPwmCtrl);
}

pub trait HrSlaveTimer: HrTimer {
    type CptCh1;
    type CptCh2;

    /// Start listening to the specified event
    fn enable_reset_event<E: super::event::TimerResetEventSource<Self::Timer, Self::Prescaler>>(
        &mut self,
        _event: &E,
    );

    /// Stop listening to the specified event
    fn disable_reset_event<E: super::event::TimerResetEventSource<Self::Timer, Self::Prescaler>>(
        &mut self,
        _event: &E,
    );

    #[cfg(feature = "stm32g4")]
    /// This is only allowed while having register preload enabled (PREEN is set to 1)
    unsafe fn swap_outputs(&self, _hr_control: &mut HrPwmCtrl, swap: SwapPins);
}

pub struct TimerSplitCapture<T, PSCL, CH1, CH2, DacRst: DacResetTrigger> {
    pub timer: HrTim<T, PSCL, (), (), DacRst>,
    pub ch1: HrCapt<T, PSCL, CH1, capture::NoDma>,
    pub ch2: HrCapt<T, PSCL, CH2, capture::NoDma>,
}

/// Trait for unsplit slave timer which still contains its capture modules
pub trait HrSlaveTimerCpt: HrSlaveTimer {
    type CaptureCh1: HrCapture;
    type CaptureCh2: HrCapture;

    fn capture_ch1(&mut self) -> &mut Self::CaptureCh1;
    fn capture_ch2(&mut self) -> &mut Self::CaptureCh2;
    fn split_capture(
        self,
    ) -> TimerSplitCapture<Self::Timer, Self::Prescaler, Ch1, Ch2, Self::DacResetTrigger>;
}

impl<TIM: Instance, PSCL: HrtimPrescaler, CPT1, CPT2, DacRst: DacResetTrigger> HrTimer
    for HrTim<TIM, PSCL, CPT1, CPT2, DacRst>
{
    type Prescaler = PSCL;
    type Timer = TIM;
    type DacResetTrigger = DacRst;

    fn get_period(&self) -> u16 {
        let tim = unsafe { &*TIM::ptr() };

        tim.perr().read().per().bits()
    }
    fn set_period(&mut self, period: u16) {
        let tim = unsafe { &*TIM::ptr() };

        tim.perr().write(|w| unsafe { w.per().bits(period as u16) });
    }

    fn get_counter_value(&self) -> u16 {
        let tim = unsafe { &*TIM::ptr() };
        tim.cntr().read().cnt().bits()
    }

    /// Start timer
    fn start(&mut self, _hr_control: &mut HrPwmCtrl) {
        // Start timer

        // SAFETY: Since we hold _hr_control there is no risk for a race condition
        let master = unsafe { &*HRTIM_MASTER::ptr() };
        master.cr().modify(|_, w| match TIM::TIMX {
            Timer::Master => w.mcen().set_bit(),
            Timer::Tim(v) => w.tcen(v as _).set_bit(),
        });
    }

    /// Stop timer
    fn stop(&mut self, _hr_control: &mut HrPwmCtrl) {
        // Stop counter
        // SAFETY: Since we hold _hr_control there is no risk for a race condition
        let master = unsafe { &*HRTIM_MASTER::ptr() };
        master.cr().modify(|_, w| match TIM::TIMX {
            Timer::Master => w.mcen().clear_bit(),
            Timer::Tim(v) => w.tcen(v as _).clear_bit(),
        });
    }

    /// Stop timer and reset counter
    fn stop_and_reset(&mut self, _hr_control: &mut HrPwmCtrl) {
        self.stop(_hr_control);

        // Reset counter
        let tim = unsafe { &*TIM::ptr() };
        unsafe {
            tim.cntr().write(|w| w.cnt().bits(0));
        }
    }

    /// Make a handle to this timers reset event to use as adc trigger
    fn as_reset_adc_trigger(&self) -> super::adc_trigger::TimerReset<Self::Timer> {
        super::adc_trigger::TimerReset(PhantomData)
    }

    /// Make a handle to this timers period event to use as adc trigger
    fn as_period_adc_trigger(&self) -> super::adc_trigger::TimerPeriod<Self::Timer> {
        super::adc_trigger::TimerPeriod(PhantomData)
    }

    fn clear_repetition_interrupt(&mut self) {
        let tim = unsafe { &*TIM::ptr() };

        tim.icr().write(|w| w.repc().clear());
    }

    /// Disable register updates
    ///
    /// Calling this function temporarily disables the transfer from preload to active registers,
    /// whatever the selected update event. This allows to modify several registers.
    /// The regular update event takes place once [`Self::enable_register_updates`] is called.
    fn disable_register_updates(&mut self, _hr_control: &mut HrPwmCtrl) {
        use super::HRTIM_COMMON;
        let common = unsafe { &*HRTIM_COMMON::ptr() };
        common.cr1().modify(|_, w| match TIM::TIMX {
            Timer::Master => w.mudis().set_bit(),
            Timer::Tim(v) => w.tudis(v as _).set_bit(),
        });
    }

    /// Enable register updates
    ///
    /// See [`Self::disable_register_updates`].
    ///
    /// NOTE: Register updates are enabled by default, no need to call this
    /// unless [`Self::disable_register_updates`] has been called.
    fn enable_register_updates<'a>(&mut self, _hr_control: &mut HrPwmCtrl) {
        use super::HRTIM_COMMON;
        let common = unsafe { &*HRTIM_COMMON::ptr() };
        common.cr1().modify(|_, w| match TIM::TIMX {
            Timer::Master => w.mudis().clear_bit(),
            Timer::Tim(v) => w.tudis(v as _).clear_bit(),
        });
    }
}

impl<TIM: Instance, PSCL, CPT1, CPT2, DacRst> HrTim<TIM, PSCL, CPT1, CPT2, DacRst>
where
    DacRst: DacResetTrigger,
{
    pub fn set_repetition_counter(&mut self, repetition_counter: u8) {
        let tim = unsafe { &*TIM::ptr() };

        unsafe {
            tim.repr().write(|w| w.rep().bits(repetition_counter));
        }
    }

    pub fn enable_repetition_interrupt(&mut self, enable: bool) {
        let tim = unsafe { &*TIM::ptr() };

        tim.dier().modify(|_r, w| w.repie().bit(enable));
    }
}

impl<TIM: InstanceX, PSCL: HrtimPrescaler, CPT1, CPT2, DacRst> HrSlaveTimer
    for HrTim<TIM, PSCL, CPT1, CPT2, DacRst>
where
    DacRst: DacResetTrigger,
{
    type CptCh1 = HrCapt<Self::Timer, Self::Prescaler, Ch1, capture::NoDma>;
    type CptCh2 = HrCapt<Self::Timer, Self::Prescaler, Ch2, capture::NoDma>;

    /// Reset this timer every time the specified event occurs
    ///
    /// Behaviour depends on `timer_mode`:
    ///
    /// * `HrTimerMode::SingleShotNonRetriggable`: Enabling the timer enables it but does not start it.
    ///   A first reset event starts the counting and any subsequent reset is ignored until the counter
    ///   reaches the PER value. The PER event is then generated and the counter is stopped. A reset event
    ///   restarts the counting from 0x0000.
    /// * `HrTimerMode:SingleShotRetriggable`: Enabling the timer enables it but does not start it.
    ///   A reset event starts the counting if the counter is stopped, otherwise it clears the counter.
    ///   When the counter reaches the PER value, the PER event is generated and the counter is stopped.
    ///   A reset event restarts the counting from 0x0000.
    /// * `HrTimerMode::Continuous`: Enabling the timer enables and starts it simultaneously.
    ///   When the counter reaches the PER value, it rolls-over to 0x0000 and resumes counting.
    ///   The counter can be reset at any time
    fn enable_reset_event<E: super::event::TimerResetEventSource<Self::Timer, Self::Prescaler>>(
        &mut self,
        _event: &E,
    ) {
        let tim = unsafe { &*TIM::ptr() };

        unsafe {
            tim.rstr().modify(|r, w| w.bits(r.bits() | E::BITS));
        }
    }

    /// Stop listening to the specified event
    fn disable_reset_event<E: super::event::TimerResetEventSource<Self::Timer, Self::Prescaler>>(
        &mut self,
        _event: &E,
    ) {
        let tim = unsafe { &*TIM::ptr() };

        unsafe {
            tim.rstr().modify(|r, w| w.bits(r.bits() & !E::BITS));
        }
    }

    #[cfg(feature = "stm32g4")]
    /// This is only allowed while having register preload enabled (PREEN is set to 1)
    unsafe fn swap_outputs(&self, _hr_control: &mut HrPwmCtrl, swap: SwapPins) {
        use super::HRTIM_COMMON;

        // SAFETY: Since we hold _hr_control there is no risk for a race condition
        let common = unsafe { &*HRTIM_COMMON::ptr() };
        common
            .cr2()
            .modify(|_r, w| w.swp(TIM::T_X as u8).bit(swap == SwapPins::Swapped));
    }
}

impl<TIM: InstanceX, PSCL, DacRst> HrSlaveTimerCpt
    for HrTim<
        TIM,
        PSCL,
        HrCapt<TIM, PSCL, Ch1, capture::NoDma>,
        HrCapt<TIM, PSCL, Ch2, capture::NoDma>,
        DacRst,
    >
where
    PSCL: HrtimPrescaler,
    DacRst: DacResetTrigger,
    HrCapt<TIM, PSCL, Ch1, capture::NoDma>: HrCapture,
    HrCapt<TIM, PSCL, Ch2, capture::NoDma>: HrCapture,
{
    type CaptureCh1 = <Self as HrSlaveTimer>::CptCh1;
    type CaptureCh2 = <Self as HrSlaveTimer>::CptCh2;

    /// Access the timers first capture channel
    fn capture_ch1(&mut self) -> &mut Self::CaptureCh1 {
        &mut self.capture_ch1
    }

    /// Access the timers second capture channel
    fn capture_ch2(&mut self) -> &mut Self::CaptureCh2 {
        &mut self.capture_ch2
    }

    fn split_capture(self) -> TimerSplitCapture<TIM, PSCL, Ch1, Ch2, DacRst> {
        let HrTim {
            _timer,
            _prescaler,
            _dac_trg,
            capture_ch1,
            capture_ch2,
        } = self;

        TimerSplitCapture {
            timer: HrTim {
                _timer,
                _prescaler,
                _dac_trg,
                capture_ch1: (),
                capture_ch2: (),
            },
            ch1: capture_ch1,
            ch2: capture_ch2,
        }
    }
}

/// Timer Period event
impl<TIM: InstanceX, DST, PSCL, CPT1, CPT2, DacRst> super::event::EventSource<DST, PSCL>
    for HrTim<TIM, PSCL, CPT1, CPT2, DacRst>
where
    DacRst: DacResetTrigger,
{
    // $rstXr
    const BITS: u32 = 1 << 2;
}

/// Timer Update event
impl<TIM: InstanceX, PSCL, CPT1, CPT2> super::capture::CaptureEvent<TIM, PSCL>
    for HrTim<TIM, PSCL, CPT1, CPT2>
{
    const BITS: u32 = 1 << 1;
}

#[cfg(feature = "stm32g4")]
use super::adc_trigger::{
    AdcTrigger13 as Adc13, AdcTrigger24 as Adc24, AdcTrigger579 as Adc579,
    AdcTrigger6810 as Adc6810,
};

#[cfg(feature = "stm32g4")]
macro_rules! hrtim_timer_adc_trigger {
    ($($TIMX:ident:
        [$(($AdcTrigger:ident: [
            $((PER: $adc_trigger_bits_period:expr),)*
            $((RST: $adc_trigger_bits_reset:expr)),*
        ])),+]
    ),+) => {
        $($(
            $(impl $AdcTrigger for super::adc_trigger::TimerReset<$TIMX> {
                const BITS: u32 = $adc_trigger_bits_reset;
            })*

            $(impl $AdcTrigger for super::adc_trigger::TimerPeriod<$TIMX> {
                const BITS: u32 = $adc_trigger_bits_period;
            })*
        )*)*
    }
}

#[cfg(feature = "stm32g4")]
hrtim_timer_adc_trigger! {
    HRTIM_MASTER: [(Adc13: [(PER: 1 << 4),]), (Adc24: [(PER: 1 << 4),]), (Adc579: [(PER: 4),]), (Adc6810: [(PER: 4),])],

    HRTIM_TIMA: [(Adc13: [(PER: 1 << 13), (RST: 1 << 14)]), (Adc24: [(PER: 1 << 13),               ]), (Adc579: [(PER: 12), (RST: 13)]), (Adc6810: [(PER: 12),          ])],
    HRTIM_TIMB: [(Adc13: [(PER: 1 << 18), (RST: 1 << 19)]), (Adc24: [(PER: 1 << 17),               ]), (Adc579: [(PER: 16), (RST: 17)]), (Adc6810: [(PER: 15),          ])],
    HRTIM_TIMC: [(Adc13: [(PER: 1 << 23),               ]), (Adc24: [(PER: 1 << 21), (RST: 1 << 22)]), (Adc579: [(PER: 20),          ]), (Adc6810: [(PER: 18), (RST: 19)])],
    HRTIM_TIMD: [(Adc13: [(PER: 1 << 27),               ]), (Adc24: [(PER: 1 << 26), (RST: 1 << 27)]), (Adc579: [(PER: 23),          ]), (Adc6810: [(PER: 22), (RST: 23)])],
    HRTIM_TIME: [(Adc13: [(PER: 1 << 31),               ]), (Adc24: [                (RST: 1 << 31)]), (Adc579: [(PER: 26),          ]), (Adc6810: [                    ])],
    HRTIM_TIMF: [(Adc13: [(PER: 1 << 24), (RST: 1 << 28)]), (Adc24: [(PER: 1 << 24),               ]), (Adc579: [(PER: 30), (RST: 31)]), (Adc6810: [(PER: 31),          ])]
}

/// Master Timer Period event
impl<DST, PSCL, CPT1, CPT2> super::event::TimerResetEventSource<DST, PSCL>
    for HrTim<HRTIM_MASTER, PSCL, CPT1, CPT2, NoDacTrigger>
{
    const BITS: u32 = 1 << 4; // MSTPER
}

/// Master Timer Period event
impl<DST, PSCL, CPT1, CPT2> super::event::EventSource<DST, PSCL>
    for HrTim<HRTIM_MASTER, PSCL, CPT1, CPT2, NoDacTrigger>
{
    const BITS: u32 = 1 << 7; // MSTPER
}
