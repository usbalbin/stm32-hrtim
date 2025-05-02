use crate::{
    ext::{Chan, TimExt},
    pac::HRTIM_COMMON,
    timer::{Ch1, Ch2, ChExt, InstanceX},
};
use core::marker::PhantomData;

use super::event::EventSource;

impl<TIM: InstanceX, PSCL, CH: ChExt> HrOutput<TIM, PSCL> for HrOut<TIM, PSCL, CH> {
    fn enable(&mut self) {
        let common = unsafe { &*HRTIM_COMMON::ptr() };
        common.oenr().write(|w| match CH::CH {
            Chan::Ch1 => w.t1oen(TIM::T_X as _).set_bit(),
            Chan::Ch2 => w.t2oen(TIM::T_X as _).set_bit(),
        });
    }

    fn disable(&mut self) {
        let common = unsafe { &*HRTIM_COMMON::ptr() };
        common.odisr().write(|w| match CH::CH {
            Chan::Ch1 => w.t1odis(TIM::T_X as _).set_bit(),
            Chan::Ch2 => w.t2odis(TIM::T_X as _).set_bit(),
        });
    }

    fn enable_set_event<ES: EventSource<TIM, PSCL>>(&mut self, _set_event: &ES) {
        let tim = unsafe { &*TIM::ptr() };
        unsafe {
            tim.set_r(CH::CH).modify(|r, w| w.bits(r.bits() | ES::BITS));
        }
    }
    fn disable_set_event<ES: EventSource<TIM, PSCL>>(&mut self, _set_event: &ES) {
        let tim = unsafe { &*TIM::ptr() };
        unsafe {
            tim.set_r(CH::CH)
                .modify(|r, w| w.bits(r.bits() & !ES::BITS));
        }
    }

    fn enable_rst_event<ES: EventSource<TIM, PSCL>>(&mut self, _reset_event: &ES) {
        let tim = unsafe { &*TIM::ptr() };
        unsafe {
            tim.rst_r(CH::CH).modify(|r, w| w.bits(r.bits() | ES::BITS));
        }
    }
    fn disable_rst_event<ES: EventSource<TIM, PSCL>>(&mut self, _reset_event: &ES) {
        let tim = unsafe { &*TIM::ptr() };
        unsafe {
            tim.rst_r(CH::CH)
                .modify(|r, w| w.bits(r.bits() & !ES::BITS));
        }
    }

    fn get_state(&self) -> State {
        let ods;
        let oen;

        unsafe {
            let common = &*HRTIM_COMMON::ptr();
            match CH::CH {
                Chan::Ch1 => {
                    ods = common.odsr().read().t1ods(TIM::T_X as _).bit_is_set();
                    oen = common.oenr().read().t1oen(TIM::T_X as _).bit_is_set();
                }
                Chan::Ch2 => {
                    ods = common.odsr().read().t2ods(TIM::T_X as _).bit_is_set();
                    oen = common.oenr().read().t2oen(TIM::T_X as _).bit_is_set();
                }
            }
        }

        match (oen, ods) {
            (true, _) => State::Running,
            (false, false) => State::Idle,
            (false, true) => State::Fault,
        }
    }
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
pub unsafe trait ToHrOut<TIM> {
    type Out<PSCL>;
}

unsafe impl<TIM, PA, PB> ToHrOut<TIM> for (PA, PB)
where
    PA: ToHrOut<TIM>,
    PB: ToHrOut<TIM>,
{
    type Out<PSCL> = (PA::Out<PSCL>, PB::Out<PSCL>);
}

pub struct HrOut<TIM, PSCL, CH>(PhantomData<(TIM, PSCL, CH)>);
pub type HrOut1<TIM, PSCL> = HrOut<TIM, PSCL, Ch1>;
pub type HrOut2<TIM, PSCL> = HrOut<TIM, PSCL, Ch2>;

unsafe impl<T> ToHrOut<T> for () {
    type Out<PSCL> = ();
}

pub struct CH1<PSCL>(PhantomData<PSCL>);
pub struct CH2<PSCL>(PhantomData<PSCL>);
