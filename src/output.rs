#[cfg(feature = "hrtim_v2")]
use crate::pac::HRTIM_TIMF;
use crate::{
    pac::{HRTIM_COMMON, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME},
    DacResetTrigger, DacStepTrigger, NoDacTrigger,
};
use core::marker::PhantomData;

use super::event::EventSource;

macro_rules! hrtim_out {
    ($($TIMX:ident: $out_type:ident: $tXYoen:ident, $tXYodis:ident, $tXYods:ident, $setXYr:ident, $rstXYr:ident,)+) => {$(
        impl<PSCL, R: DacResetTrigger, S: DacStepTrigger> HrOutput<$TIMX, PSCL> for $out_type<$TIMX, PSCL, R, S> {
            fn enable(&mut self) {
                let common = unsafe { &*HRTIM_COMMON::ptr() };
                common.oenr().write(|w| { w.$tXYoen().set_bit() });
            }

            fn disable(&mut self) {
                let common = unsafe { &*HRTIM_COMMON::ptr() };
                common.odisr().write(|w| { w.$tXYodis().set_bit() });
            }

            fn enable_set_event<ES: EventSource<$TIMX, PSCL>>(&mut self, _set_event: &ES) {
                let tim = unsafe { &*$TIMX::ptr() };
                unsafe { tim.$setXYr().modify(|r, w| w.bits(r.bits() | ES::BITS)); }
            }
            fn disable_set_event<ES: EventSource<$TIMX, PSCL>>(&mut self, _set_event: &ES) {
                let tim = unsafe { &*$TIMX::ptr() };
                unsafe { tim.$setXYr().modify(|r, w| w.bits(r.bits() & !ES::BITS)); }
            }

            fn enable_rst_event<ES: EventSource<$TIMX, PSCL>>(&mut self, _reset_event: &ES) {
                let tim = unsafe { &*$TIMX::ptr() };
                unsafe { tim.$rstXYr().modify(|r, w| w.bits(r.bits() | ES::BITS)); }
            }
            fn disable_rst_event<ES: EventSource<$TIMX, PSCL>>(&mut self, _reset_event: &ES) {
                let tim = unsafe { &*$TIMX::ptr() };
                unsafe { tim.$rstXYr().modify(|r, w| w.bits(r.bits() & !ES::BITS)); }
            }

            fn get_state(&self) -> State {
                let ods;
                let oen;

                unsafe {
                    let common = &*HRTIM_COMMON::ptr();
                    ods = common.odsr().read().$tXYods().bit_is_set();
                    oen = common.oenr().read().$tXYoen().bit_is_set();
                }

                match (oen, ods) {
                    (true, _) => State::Running,
                    (false, false) => State::Idle,
                    (false, true) => State::Fault
                }
            }
        }
    )+};
}

hrtim_out! {
    HRTIM_TIMA: HrOut1: ta1oen, ta1odis, ta1ods, set1r, rst1r,
    HRTIM_TIMA: HrOut2: ta2oen, ta2odis, ta2ods, set2r, rst2r,

    HRTIM_TIMB: HrOut1: tb1oen, tb1odis, tb1ods, set1r, rst1r,
    HRTIM_TIMB: HrOut2: tb2oen, tb2odis, tb2ods, set2r, rst2r,

    HRTIM_TIMC: HrOut1: tc1oen, tc1odis, tc1ods, set1r, rst1r,
    HRTIM_TIMC: HrOut2: tc2oen, tc2odis, tc2ods, set2r, rst2r,

    HRTIM_TIMD: HrOut1: td1oen, td1odis, td1ods, set1r, rst1r,
    HRTIM_TIMD: HrOut2: td2oen, td2odis, td2ods, set2r, rst2r,

    HRTIM_TIME: HrOut1: te1oen, te1odis, te1ods, set1r, rst1r,
    HRTIM_TIME: HrOut2: te2oen, te2odis, te2ods, set2r, rst2r,
}

#[cfg(feature = "hrtim_v2")]
hrtim_out! {
    HRTIM_TIMF: HrOut1: tf1oen, tf1odis, tf1ods, set1r, rst1r,
    HRTIM_TIMF: HrOut2: tf2oen, tf2odis, tf2ods, set2r, rst2r,
}

pub trait HrOutput<TIM, PSCL> {
    /// Enable this output
    fn enable(&mut self);

    /// Disable this output
    fn disable(&mut self);

    /// Set this output to active every time the specified event occurs
    ///
    /// NOTE: Enabling the same event for both SET and RESET
    /// will make that event TOGGLE the output
    fn enable_set_event<ES: EventSource<TIM, PSCL>>(&mut self, set_event: &ES);

    /// Stop listening to the specified event
    fn disable_set_event<ES: EventSource<TIM, PSCL>>(&mut self, set_event: &ES);

    /// Set this output to *not* active every time the specified event occurs
    ///
    /// NOTE: Enabling the same event for both SET and RESET
    /// will make that event TOGGLE the output
    fn enable_rst_event<ES: EventSource<TIM, PSCL>>(&mut self, reset_event: &ES);

    /// Stop listening to the specified event
    fn disable_rst_event<ES: EventSource<TIM, PSCL>>(&mut self, reset_event: &ES);

    /// Get current state of the output
    fn get_state(&self) -> State;
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Idle,
    Running,
    Fault,
}

impl State {
    pub fn is_idle(self) -> bool {
        matches!(self, State::Idle)
    }

    pub fn is_running(self) -> bool {
        matches!(self, State::Running)
    }

    pub fn is_fault(self) -> bool {
        matches!(self, State::Fault)
    }
}

/// # Safety
/// Caller needs to ensure that this is only implemented
/// for types that represent pin that can act as an output
/// for the specified timer `TIM`
pub unsafe trait ToHrOut<TIM, DacRst = NoDacTrigger, DacStp = NoDacTrigger>
where
    DacRst: DacResetTrigger,
    DacStp: DacStepTrigger,
{
    type Out<PSCL>;
}

unsafe impl<TIM, PA, PB, DacRst, DacStp> ToHrOut<TIM, DacRst, DacStp> for (PA, PB)
where
    PA: ToHrOut<TIM>,
    PB: ToHrOut<TIM>,
    DacRst: DacResetTrigger,
    DacStp: DacStepTrigger,
{
    type Out<PSCL> = (PA::Out<PSCL>, PB::Out<PSCL>);
}

// NOTE: Only HrOut1 can actually be used as a dac trigger

pub struct HrOut1<
    TIM,
    PSCL,
    DacRst: DacResetTrigger = NoDacTrigger,
    DacStp: DacStepTrigger = NoDacTrigger,
>(PhantomData<(TIM, PSCL, DacRst, DacStp)>);
pub struct HrOut2<
    TIM,
    PSCL,
    DacRst: DacResetTrigger = NoDacTrigger,
    DacStp: DacStepTrigger = NoDacTrigger,
>(PhantomData<(TIM, PSCL, DacRst, DacStp)>);

unsafe impl<T> ToHrOut<T> for () {
    type Out<PSCL> = ();
}
