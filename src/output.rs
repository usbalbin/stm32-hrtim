#[cfg(feature = "hrtim_v2")]
use crate::stm32::HRTIM_TIMF;
use crate::{
    hal,
    mcu::GpioInputMode,
    stm32::{HRTIM_COMMON, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME},
};
use core::marker::PhantomData;

use super::event::EventSource;

use hal::gpio::{
    gpioa::{PA10, PA11, PA8, PA9},
    gpioc::PC8,
};

#[cfg(any(feature = "stm32f3", feature = "stm32g4"))]
use hal::gpio::{
    gpiob::{PB12, PB13, PB14, PB15},
    gpioc::PC9,
};

#[cfg(feature = "stm32h7")]
use hal::gpio::{
    gpioa::PA12,
    gpioc::{PC6, PC7},
    gpiog::{PG6, PG7},
};

#[cfg(feature = "stm32g4")]
use hal::gpio::gpioc::{PC6, PC7};

mod sealed {
    pub trait Sealed<T> {}
}

macro_rules! hrtim_out {
    ($($TIMX:ident: $out_type:ident: $tXYoen:ident, $tXYodis:ident, $tXYods:ident, $setXYr:ident, $rstXYr:ident,)+) => {$(
        impl<PSCL> HrOutput<$TIMX, PSCL> for $out_type<$TIMX, PSCL> {
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

pub trait ToHrOut<TIM>: sealed::Sealed<TIM> {
    type Out<PSCL>;

    #[cfg(feature = "stm32f3")]
    type GpioX: hal::gpio::marker::GpioStatic;
    #[cfg(feature = "stm32f3")]
    type Afr;

    fn connect_to_hrtim(
        self,
        #[cfg(feature = "stm32f3")]
        moder: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::MODER,
        #[cfg(feature = "stm32f3")]
        otyper: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::OTYPER,
        #[cfg(feature = "stm32f3")] afr: &mut Self::Afr,
    );
}

impl<TIM, PA, PB> sealed::Sealed<TIM> for (PA, PB)
where
    PA: ToHrOut<TIM>,
    PB: ToHrOut<TIM>,
{
}

#[cfg(feature = "stm32f3")]
impl<TIM, PA, PB> ToHrOut<TIM> for (PA, PB)
where
    PA: ToHrOut<TIM>,
    PB: ToHrOut<TIM, GpioX = PA::GpioX, Afr = PA::Afr>,
{
    type Out<PSCL> = (PA::Out<PSCL>, PB::Out<PSCL>);
    type GpioX = PA::GpioX;
    type Afr = PA::Afr;

    fn connect_to_hrtim(
        self,

        moder: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::MODER,

        otyper: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::OTYPER,

        afr: &mut Self::Afr,
    ) {
        self.0.connect_to_hrtim(moder, otyper, afr);
        self.1.connect_to_hrtim(moder, otyper, afr);
    }
}

#[cfg(any(feature = "stm32g4", feature = "stm32h7"))]
impl<TIM, PA, PB> ToHrOut<TIM> for (PA, PB)
where
    PA: ToHrOut<TIM>,
    PB: ToHrOut<TIM>,
{
    type Out<PSCL> = (PA::Out<PSCL>, PB::Out<PSCL>);

    fn connect_to_hrtim(self) {
        self.0.connect_to_hrtim();
        self.1.connect_to_hrtim();
    }
}

pub struct HrOut1<TIM, PSCL>(PhantomData<(TIM, PSCL)>);
pub struct HrOut2<TIM, PSCL>(PhantomData<(TIM, PSCL)>);

macro_rules! pins_helper {
    ($TIMX:ty, $HrOutY:ident, $CHY:ident<$CHY_AF:literal>, $GpioX:ident) => {
        impl sealed::Sealed<$TIMX> for $CHY<GpioInputMode> {}

        impl ToHrOut<$TIMX> for $CHY<GpioInputMode> {
            type Out<PSCL> = $HrOutY<$TIMX, PSCL>;

            #[cfg(feature = "stm32f3")]
            type GpioX = hal::gpio::$GpioX;
            #[cfg(feature = "stm32f3")]
            type Afr = <Self as hal::gpio::marker::IntoAf<{ $CHY_AF }>>::AFR;

            // Pin<Gpio, Index, Alternate<PushPull, AF>>
            fn connect_to_hrtim(
                self,
                #[cfg(feature = "stm32f3")]
                moder: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::MODER,
                #[cfg(feature = "stm32f3")]
                otyper: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::OTYPER,
                #[cfg(feature = "stm32f3")] afr: &mut Self::Afr,
            ) {
                #[allow(non_snake_case, unused_variables)]
                let $GpioX = ();

                #[cfg(feature = "stm32f3")]
                let _: $CHY<hal::gpio::Alternate<hal::gpio::PushPull, $CHY_AF>> =
                    self.into_af_push_pull(moder, otyper, afr);

                #[cfg(any(feature = "stm32g4", feature = "stm32h7"))]
                let _: $CHY<hal::gpio::Alternate<{ $CHY_AF }>> = self.into_alternate();
            }
        }
    };
}

// $GpioX is only used for f3x4
macro_rules! pins {
    ($($TIMX:ty: CH1: $CH1:ident<$CH1_AF:literal>, CH2: $CH2:ident<$CH2_AF:literal>, $GpioX:ident)+) => {$(
        pins_helper!($TIMX, HrOut1, $CH1<$CH1_AF>, $GpioX);
        pins_helper!($TIMX, HrOut2, $CH2<$CH2_AF>, $GpioX);
    )+};
}

#[cfg(any(feature = "stm32g4", feature = "stm32f3"))]
pins! {
    HRTIM_TIMA: CH1: PA8<13>,  CH2: PA9<13>,  Gpioa
    HRTIM_TIMB: CH1: PA10<13>, CH2: PA11<13>, Gpioa
    HRTIM_TIMC: CH1: PB12<13>, CH2: PB13<13>, Gpiob
    HRTIM_TIMD: CH1: PB14<13>, CH2: PB15<13>, Gpiob
    HRTIM_TIME: CH1: PC8<3>,   CH2: PC9<3>,   Gpioc
}

#[cfg(feature = "stm32g4")]
pins! {
    HRTIM_TIMF: CH1: PC6<13>, CH2: PC7<13>, Gpioc
}

// TODO: H7 does not start in input mode, it starts in Analog
#[cfg(feature = "stm32h7")] // RM0433
pins! {
    HRTIM_TIMA: CH1: PC6<1>,  CH2: PC7<1>,  Gpioc
    HRTIM_TIMB: CH1: PC8<1>, CH2: PA8<2>, GpioC // This type is not used for in this config so it's ok
    HRTIM_TIMC: CH1: PA9<2>, CH2: PA10<2>, GpioA
    HRTIM_TIMD: CH1: PA11<2>, CH2: PA12<2>, GpioA
    HRTIM_TIME: CH1: PG6<2>,  CH2: PG7<2>,   GpioG
}

impl<T> sealed::Sealed<T> for () {}
impl<T> ToHrOut<T> for () {
    type Out<PSCL> = ();
    #[cfg(feature = "stm32f3")]
    type GpioX = hal::gpio::Gpioa;
    #[cfg(feature = "stm32f3")]
    type Afr = ();

    fn connect_to_hrtim(
        self,
        #[cfg(feature = "stm32f3")]
        _moder: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::MODER,
        #[cfg(feature = "stm32f3")]
        _otyper: &mut <Self::GpioX as hal::gpio::marker::GpioStatic>::OTYPER,
        #[cfg(feature = "stm32f3")] _afr: &mut Self::Afr,
    ) {
    }
}

pub struct CH1<PSCL>(PhantomData<PSCL>);
pub struct CH2<PSCL>(PhantomData<PSCL>);
