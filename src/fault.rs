use crate::pac::HRTIM_COMMON;

use super::control::HrPwmCtrl;

/// Allows a FaultMonitor to monitor faults
pub trait FaultMonitor {
    fn enable_interrupt(&mut self, hr_control: &mut HrPwmCtrl);

    /// Returns true if a fault is preventing PWM output
    fn is_fault_active(&self) -> bool;

    /// Clear the fault interrupt flag
    ///
    /// This will *NOT* resume normal PWM operation. The affected outputs need to be re-enabled to resume operation;
    /// This will do nothing if the fault is still active.
    fn clear_fault(&mut self);
}

pub enum FaultAction {
    /// Output never enters fault mode
    None = 0b00,

    /// Output forced to `active` level on fault
    ForceActive = 0b01,

    /// Output forced to `inactive` level on fault
    ForceInactive = 0b10,

    /// The output is floating/tri stated on fault
    Floating = 0b11,
}

/// # Safety
/// Only implement for actual fault sources with correct `ENABLE_BITS`
pub unsafe trait FaultSource: Copy {
    const ENABLE_BITS: u8;
}

#[cfg(feature = "stm32g4")]
pub struct SourceBuilder<I> {
    _input: I,
    src_bits: u8,

    /// FLTxP
    is_active_high: bool,

    /// FLTxF[3:0]
    filter_bits: u8,
}

#[cfg(feature = "stm32g4")]
impl<I> SourceBuilder<I> {
    pub unsafe fn new(input: I, src_bits: u8) -> Self {
        SourceBuilder {
            _input: input,
            src_bits,
            is_active_high: false,
            filter_bits: 0b0000,
        }
    }
}

#[non_exhaustive]
pub struct FaultInput1;
#[non_exhaustive]
pub struct FaultInput2;
#[non_exhaustive]
pub struct FaultInput3;
#[non_exhaustive]
pub struct FaultInput4;
#[non_exhaustive]
pub struct FaultInput5;
#[non_exhaustive]
pub struct FaultInput6;

pub struct FaultInputs {
    #[cfg(feature = "stm32g4")]
    pub fault_input1: FaultInput1,
    #[cfg(feature = "stm32g4")]
    pub fault_input2: FaultInput2,
    #[cfg(feature = "stm32g4")]
    pub fault_input3: FaultInput3,
    #[cfg(feature = "stm32g4")]
    pub fault_input4: FaultInput4,
    #[cfg(feature = "stm32g4")]
    pub fault_input5: FaultInput5,
    #[cfg(feature = "stm32g4")]
    pub fault_input6: FaultInput6,
}

impl FaultInputs {
    pub(crate) unsafe fn new() -> Self {
        FaultInputs {
            #[cfg(feature = "stm32g4")]
            fault_input1: FaultInput1,
            #[cfg(feature = "stm32g4")]
            fault_input2: FaultInput2,
            #[cfg(feature = "stm32g4")]
            fault_input3: FaultInput3,
            #[cfg(feature = "stm32g4")]
            fault_input4: FaultInput4,
            #[cfg(feature = "stm32g4")]
            fault_input5: FaultInput5,
            #[cfg(feature = "stm32g4")]
            fault_input6: FaultInput6,
        }
    }
}

pub enum FaultSamplingFilter {
    /// No filtering, fault acts asynchronously
    ///
    /// Note that this bypasses any f_flts (SamplingClkDiv)
    None = 0b0000,

    /// Sample directly at rate f_hrtim, with a count of 2
    ///
    /// Note that this bypasses: any f_flts (SamplingClkDiv)
    HrtimN2 = 0b0001,

    /// Sample directly at rate f_hrtim, with a count of 4
    ///
    /// Note that this bypasses any f_flts (SamplingClkDiv)
    HrtimN4 = 0b0010,

    /// Sample directly at rate f_hrtim, with a count of 8
    ///
    /// Note that this bypasses any f_flts (SamplingClkDiv)
    HrtimN8 = 0b0011,

    /// Sample at rate f_flts / 2, with a count of 6
    FltsDiv2N6 = 0b0100,

    /// Sample at rate f_flts / 2, with a count of 8
    FltsDiv2N8 = 0b0101,

    /// Sample at rate f_flts / 4, with a count of 6
    FltsDiv4N6 = 0b0110,

    /// Sample at rate f_flts / 4, with a count of 8
    FltsDiv4N8 = 0b0111,

    /// Sample at rate f_flts / 8, with a count of 6
    FltsDiv8N6 = 0b1000,

    /// Sample at rate f_flts / 8, with a count of 8
    FltsDiv8N8 = 0b1001,

    /// Sample at rate f_flts / 16, with a count of 5
    FltsDiv16N5 = 0b1010,

    /// Sample at rate f_flts / 16, with a count of 6
    FltsDiv16N6 = 0b1011,

    /// Sample at rate f_flts / 16, with a count of 8
    FltsDiv16N8 = 0b1100,

    /// Sample at rate f_flts / 32, with a count of 5
    FltsDiv32N5 = 0b1101,

    /// Sample at rate f_flts / 32, with a count of 6
    FltsDiv32N6 = 0b1110,

    /// Sample at rate f_flts / 32, with a count of 8
    FltsDiv32N8 = 0b1111,
}

macro_rules! impl_flt_monitor {
    ($($t:ident: ($fltx:ident, $fltxc:ident, $fltxie:ident),)+) => {$(
        #[non_exhaustive]
        pub struct $t;

        impl FaultMonitor for $t {
            fn enable_interrupt(&mut self, _hr_control: &mut HrPwmCtrl) {
                let common = unsafe { &*HRTIM_COMMON::ptr() };
                common.ier().modify(|_r, w| w.$fltxie().set_bit());
            }

            fn is_fault_active(&self) -> bool {
                let common = unsafe { &*HRTIM_COMMON::ptr() };
                common.isr().read().$fltx().bit()
            }

            fn clear_fault(&mut self) {
                let common = unsafe { &*HRTIM_COMMON::ptr() };
                common.icr().write(|w| w.$fltxc().clear());
            }
        }
    )+};
}

impl_flt_monitor!(
    FltMonitorSys: (sysflt, sysfltc, sysfltie),
    FltMonitor1: (flt1, flt1c, flt1ie),
    FltMonitor2: (flt2, flt2c, flt2ie),
    FltMonitor3: (flt3, flt3c, flt3ie),
    FltMonitor4: (flt4, flt4c, flt4ie),
    FltMonitor5: (flt5, flt5c, flt5ie),
);

#[cfg(feature = "hrtim_v2")]
impl_flt_monitor!(
    FltMonitor6: (flt6, flt6c, flt6ie),
);
