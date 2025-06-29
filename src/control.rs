#[cfg(feature = "hrtim_v2")]
use crate::adc_trigger;
#[cfg(feature = "hrtim_v2")]
use crate::fault::FltMonitor6;
use crate::fault::{
    FltMonitor1, FltMonitor2, FltMonitor3, FltMonitor4, FltMonitor5, FltMonitorSys,
};

use crate::timer::{self, HrTimer};
use crate::{pac, pac::HRTIM_COMMON};

use super::{external_event::EevInputs, fault::FaultInputs};

impl HrTimOngoingCalibration {
    /// Look in the hal for an corresponding extension trait for `HRTIM_COMMON`.
    ///
    /// ..unless you are the one implementing the hal
    ///
    /// # Safety
    /// The user is expected to have setup and enabled rcc clock to the peripheral
    pub unsafe fn hr_control() -> HrTimOngoingCalibration {
        #[allow(unused_variables)]
        let common = unsafe { &*HRTIM_COMMON::ptr() };

        // Start calibration procedure
        #[cfg(not(feature = "stm32h7"))]
        common
            .dllcr()
            .write(|w| w.cal().set_bit().calen().clear_bit());

        HrTimOngoingCalibration {
            #[cfg(feature = "stm32g4")]
            adc_trigger1_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger2_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger3_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger4_postscaler: AdcTriggerPostscaler::None,

            #[cfg(feature = "stm32g4")]
            adc_trigger5_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger6_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger7_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger8_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger9_postscaler: AdcTriggerPostscaler::None,
            #[cfg(feature = "stm32g4")]
            adc_trigger10_postscaler: AdcTriggerPostscaler::None,

            flt_divider: SamplingClkDiv::None,
            eev_divider: SamplingClkDiv::None,
        }
    }
}

pub struct HrTimOngoingCalibration {
    #[cfg(feature = "stm32g4")]
    adc_trigger1_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger2_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger3_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger4_postscaler: AdcTriggerPostscaler,

    #[cfg(feature = "stm32g4")]
    adc_trigger5_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger6_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger7_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger8_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger9_postscaler: AdcTriggerPostscaler,
    #[cfg(feature = "stm32g4")]
    adc_trigger10_postscaler: AdcTriggerPostscaler,

    flt_divider: SamplingClkDiv,
    eev_divider: SamplingClkDiv,
}

impl HrTimOngoingCalibration {
    /// SAFETY: Calibration needs to be done before calling this
    unsafe fn init(self) {
        let common = unsafe { &*HRTIM_COMMON::ptr() };

        let Self {
            #[cfg(feature = "stm32g4")]
            adc_trigger1_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger2_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger3_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger4_postscaler,

            #[cfg(feature = "stm32g4")]
            adc_trigger5_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger6_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger7_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger8_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger9_postscaler,
            #[cfg(feature = "stm32g4")]
            adc_trigger10_postscaler,

            flt_divider,
            eev_divider,
        } = self;

        unsafe {
            // Enable periodic calibration
            // with f_hrtim at 170MHz, these settings leads to
            // a period of about 6.2ms
            #[cfg(not(feature = "stm32h7"))]
            common
                .dllcr()
                .modify(|_r, w| w.calrte().bits(0b00).cal().set_bit().calen().clear_bit());
            common
                .fltinr2()
                .write(|w| w.fltsd().bits(flt_divider as u8));

            common.eecr3().write(|w| w.eevsd().bits(eev_divider as u8));

            #[cfg(feature = "stm32g4")]
            common.adcps1().write(|w| {
                w.adc1psc()
                    .bits(adc_trigger1_postscaler as u8)
                    .adc2psc()
                    .bits(adc_trigger2_postscaler as u8)
                    .adc3psc()
                    .bits(adc_trigger3_postscaler as u8)
                    .adc4psc()
                    .bits(adc_trigger4_postscaler as u8)
                    .adc5psc()
                    .bits(adc_trigger5_postscaler as u8)
            });

            #[cfg(feature = "stm32g4")]
            common.adcps2().write(|w| {
                w.adc6psc()
                    .bits(adc_trigger6_postscaler as u8)
                    .adc7psc()
                    .bits(adc_trigger7_postscaler as u8)
                    .adc8psc()
                    .bits(adc_trigger8_postscaler as u8)
                    .adc9psc()
                    .bits(adc_trigger9_postscaler as u8)
                    .adc10psc()
                    .bits(adc_trigger10_postscaler as u8)
            });

            // TODO: Adc trigger 5-10
        }
    }

    pub fn wait_for_calibration(self) -> (HrTimCalibrated, FaultInputs, EevInputs) {
        #[cfg(not(feature = "stm32h7"))]
        {
            let common = unsafe { &*HRTIM_COMMON::ptr() };
            while common.isr().read().dllrdy().bit_is_clear() {
                // Wait until ready
            }
        }

        // Calibration is now done, it is safe to continue
        unsafe { self.init() };

        (HrTimCalibrated, unsafe { FaultInputs::new() }, unsafe {
            EevInputs::new()
        })
    }

    #[cfg(feature = "stm32g4")]
    pub fn set_adc1_trigger_psc(mut self, post_scaler: AdcTriggerPostscaler) -> Self {
        self.adc_trigger1_postscaler = post_scaler;
        self
    }

    #[cfg(feature = "stm32g4")]
    pub fn set_adc2_trigger_psc(mut self, post_scaler: AdcTriggerPostscaler) -> Self {
        self.adc_trigger2_postscaler = post_scaler;
        self
    }

    #[cfg(feature = "stm32g4")]
    pub fn set_adc3_trigger_psc(mut self, post_scaler: AdcTriggerPostscaler) -> Self {
        self.adc_trigger3_postscaler = post_scaler;
        self
    }

    #[cfg(feature = "stm32g4")]
    pub fn set_adc4_trigger_psc(mut self, post_scaler: AdcTriggerPostscaler) -> Self {
        self.adc_trigger4_postscaler = post_scaler;
        self
    }

    pub fn set_fault_sampling_division(mut self, divider: SamplingClkDiv) -> Self {
        self.flt_divider = divider;
        self
    }

    pub fn set_eev_sampling_division(mut self, divider: SamplingClkDiv) -> Self {
        self.eev_divider = divider;
        self
    }
}

/// This object may be used for things that needs to be done before any timers have been started but after the calibration has been completed. Its existence is proof that no timers have started.
///
/// Once done with setup, use the `constrain` to get a `HrPwmControl` which can be used to start the timers.
#[non_exhaustive]
pub struct HrTimCalibrated;

impl HrTimCalibrated {
    pub fn constrain(self) -> HrPwmControl {
        HrPwmControl {
            control: HrPwmCtrl,
            fault_sys: FltMonitorSys,
            fault_1: FltMonitor1,
            fault_2: FltMonitor2,
            fault_3: FltMonitor3,
            fault_4: FltMonitor4,
            fault_5: FltMonitor5,
            #[cfg(feature = "hrtim_v2")]
            fault_6: FltMonitor6,

            #[cfg(feature = "stm32g4")]
            adc_trigger1: adc_trigger::AdcTrigger1,
            #[cfg(feature = "stm32g4")]
            adc_trigger2: adc_trigger::AdcTrigger2,
            #[cfg(feature = "stm32g4")]
            adc_trigger3: adc_trigger::AdcTrigger3,
            #[cfg(feature = "stm32g4")]
            adc_trigger4: adc_trigger::AdcTrigger4,
            #[cfg(feature = "stm32g4")]
            adc_trigger5: adc_trigger::AdcTrigger5,
            #[cfg(feature = "stm32g4")]
            adc_trigger6: adc_trigger::AdcTrigger6,
            #[cfg(feature = "stm32g4")]
            adc_trigger7: adc_trigger::AdcTrigger7,
            #[cfg(feature = "stm32g4")]
            adc_trigger8: adc_trigger::AdcTrigger8,
            #[cfg(feature = "stm32g4")]
            adc_trigger9: adc_trigger::AdcTrigger9,
            #[cfg(feature = "stm32g4")]
            adc_trigger10: adc_trigger::AdcTrigger10,
        }
    }
}

impl<'a> From<&'a mut HrPwmControl> for &'a mut HrPwmCtrl {
    fn from(val: &'a mut HrPwmControl) -> Self {
        &mut val.control
    }
}

/// Used as a token to guarantee unique access to resources common to multiple timers
///
/// An instance of this object can be obtained from [`HrPwmControl`].control
#[non_exhaustive]
pub struct HrPwmCtrl;

pub struct Foo<'a>(&'a mut pac::hrtim_master::cr::W);

impl<'a> Foo<'a> {
    pub fn start<T: HrTimer>(self, _t: &mut T) -> Self {
        use crate::timer::Instance;

        let w = self.0;
        Foo(match T::Timer::TIMX {
            timer::Timer::Master => w.mcen().set_bit(),
            timer::Timer::Tim(v) => w.tcen(v as _).set_bit(),
        })
    }
    pub fn stop<T: HrTimer>(self, _t: &mut T) -> Self {
        use crate::timer::Instance;

        let w = self.0;
        Foo(match T::Timer::TIMX {
            timer::Timer::Master => w.mcen().clear_bit(),
            timer::Timer::Tim(v) => w.tcen(v as _).clear_bit(),
        })
    }
}

impl HrPwmCtrl {
    /// Start/stop multiple timers at the exact same time
    ///
    /// ```
    /// let mut timer_a = ...;
    /// let mut timer_b = ...;
    /// let mut timer_c = ...;
    /// hr_control.start_stop_timers(|w| w
    ///     .start(&mut timer_a)
    ///     .start(&mut timer_b)
    ///     .stop(&mut timer_c)
    /// );
    /// ```
    pub fn start_stop_timers(&mut self, p: impl FnOnce(Foo) -> Foo) {
        let master = unsafe { pac::HRTIM_MASTER::steal() };
        master.cr().modify(|_, w| p(Foo(w)).0);
    }
}

/// Used as a token to guarantee unique access to resources common to multiple timers
#[non_exhaustive]
pub struct HrPwmControl {
    pub control: HrPwmCtrl,

    pub fault_sys: FltMonitorSys,
    pub fault_1: FltMonitor1,
    pub fault_2: FltMonitor2,
    pub fault_3: FltMonitor3,
    pub fault_4: FltMonitor4,
    pub fault_5: FltMonitor5,
    #[cfg(feature = "stm32g4")]
    pub fault_6: FltMonitor6,

    #[cfg(feature = "stm32g4")]
    pub adc_trigger1: adc_trigger::AdcTrigger1,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger2: adc_trigger::AdcTrigger2,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger3: adc_trigger::AdcTrigger3,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger4: adc_trigger::AdcTrigger4,

    #[cfg(feature = "stm32g4")]
    pub adc_trigger5: adc_trigger::AdcTrigger5,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger6: adc_trigger::AdcTrigger6,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger7: adc_trigger::AdcTrigger7,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger8: adc_trigger::AdcTrigger8,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger9: adc_trigger::AdcTrigger9,
    #[cfg(feature = "stm32g4")]
    pub adc_trigger10: adc_trigger::AdcTrigger10,
}

#[cfg(feature = "stm32g4")]
pub enum AdcTriggerPostscaler {
    None = 0,
    Div2 = 1,
    Div3 = 2,
    Div4 = 3,
    Div5 = 4,
    Div6 = 5,
    Div7 = 6,
    Div8 = 7,
    Div9 = 8,
    Div10 = 9,
    Div11 = 10,
    Div12 = 11,
    Div13 = 12,
    Div14 = 13,
    Div15 = 14,
    Div16 = 15,
    Div17 = 16,
    Div18 = 17,
    Div19 = 18,
    Div20 = 19,
    Div21 = 20,
    Div22 = 21,
    Div23 = 22,
    Div24 = 23,
    Div25 = 24,
    Div26 = 25,
    Div27 = 26,
    Div28 = 27,
    Div29 = 28,
    Div30 = 29,
    Div31 = 30,
    Div32 = 31,
}

/// The divsion ratio between f_hrtim and the fault signal sampling clock for digital filters
pub enum SamplingClkDiv {
    /// No division
    ///
    /// fault signal sampling clock f_flts = f_hrtim
    None = 0b00,

    /// 1/2
    ///
    /// fault signal sampling clock f_flts = f_hrtim / 2
    Two = 0b01,

    /// 1/4
    ///
    /// fault signal sampling clock f_flts = f_hrtim / 4
    Four = 0b10,

    /// 1/8
    ///
    /// fault signal sampling clock f_flts = f_hrtim / 8
    Eight = 0b11,
}
