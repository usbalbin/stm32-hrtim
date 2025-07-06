use crate::pac::HRTIM_COMMON;
use crate::Polarity;

use super::control::HrTimCalibrated;

#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq)]
pub struct ExternalEventSource<const N: u8, const IS_FAST: bool>;

pub struct EevInputs {
    pub eev_input1: EevInput<1>,
    pub eev_input2: EevInput<2>,
    pub eev_input3: EevInput<3>,
    pub eev_input4: EevInput<4>,
    pub eev_input5: EevInput<5>,
    pub eev_input6: EevInput<6>,
    pub eev_input7: EevInput<7>,
    pub eev_input8: EevInput<8>,
    pub eev_input9: EevInput<9>,
    pub eev_input10: EevInput<10>,
}

impl EevInputs {
    pub(crate) unsafe fn new() -> Self {
        EevInputs {
            eev_input1: EevInput,
            eev_input2: EevInput,
            eev_input3: EevInput,
            eev_input4: EevInput,
            eev_input5: EevInput,
            eev_input6: EevInput,
            eev_input7: EevInput,
            eev_input8: EevInput,
            eev_input9: EevInput,
            eev_input10: EevInput,
        }
    }
}

#[non_exhaustive]
pub struct EevInput<const N: u8>;

/// This is implemented for types that can be used as inputs to the eev
/// # Safety
/// Only implement for types that can be used as sources to eev number `EEV_N` with src bits `SRC_BITS`
pub unsafe trait EevSrcBits<const EEV_N: u8>: Sized {
    const SRC_BITS: u8;
    fn cfg(self) {}
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EdgeOrPolarity {
    Edge(Edge),
    Polarity(Polarity),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Edge {
    Rising = 0b01,
    Falling = 0b10,
    Both = 0b11,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EevSamplingFilter {
    /// No filtering, fault acts asynchronously
    ///
    /// Note that this bypasses any f_eevs (FaultSamplingClkDiv)
    None = 0b0000,

    /// Sample directly at rate f_hrtim, with a count of 2
    ///
    /// Note that this bypasses: any f_eevs (FaultSamplingClkDiv)
    HrtimN2 = 0b0001,

    /// Sample directly at rate f_hrtim, with a count of 4
    ///
    /// Note that this bypasses any f_eevs (FaultSamplingClkDiv)
    HrtimN4 = 0b0010,

    /// Sample directly at rate f_hrtim, with a count of 8
    ///
    /// Note that this bypasses any f_eevs (FaultSamplingClkDiv)
    HrtimN8 = 0b0011,

    /// Sample at rate f_eevs / 2, with a count of 6
    EevsDiv2N6 = 0b0100,

    /// Sample at rate f_eevs / 2, with a count of 8
    EevsDiv2N8 = 0b0101,

    /// Sample at rate f_eevs / 4, with a count of 6
    EevsDiv4N6 = 0b0110,

    /// Sample at rate f_eevs / 4, with a count of 8
    EevsDiv4N8 = 0b0111,

    /// Sample at rate f_eevs / 8, with a count of 6
    EevsDiv8N6 = 0b1000,

    /// Sample at rate f_eevs / 8, with a count of 8
    EevsDiv8N8 = 0b1001,

    /// Sample at rate f_eevs / 16, with a count of 5
    EevsDiv16N5 = 0b1010,

    /// Sample at rate f_eevs / 16, with a count of 6
    EevsDiv16N6 = 0b1011,

    /// Sample at rate f_eevs / 16, with a count of 8
    EevsDiv16N8 = 0b1100,

    /// Sample at rate f_eevs / 32, with a count of 5
    EevsDiv32N5 = 0b1101,

    /// Sample at rate f_eevs / 32, with a count of 6
    EevsDiv32N6 = 0b1110,

    /// Sample at rate f_eevs / 32, with a count of 8
    EevsDiv32N8 = 0b1111,
}

pub trait ExternalEventBuilder1To5 {}
pub trait ExternalEventBuilder6To10 {}
pub struct SourceBuilder<const N: u8, const IS_FAST: bool> {
    /// EExSRC
    src_bits: u8,

    /// EExSNS
    edge_or_polarity_bits: u8,

    /// EExPOL
    polarity_bit: bool,

    /// EExF
    filter_bits: u8,
}

#[cfg(feature = "stm32g4")]
impl<const N: u8, const IS_FAST: bool> SourceBuilder<N, IS_FAST> {
    /// # Safety
    /// Caller needs to ensure that src_bits is a valid bit pattern
    /// for eeXsrc bits in eecr1/2 registers for the intended input
    pub unsafe fn new(src_bits: u8) -> Self {
        Self {
            src_bits,
            edge_or_polarity_bits: 0, // Level sensitive
            polarity_bit: false,      // Active high
            filter_bits: 0,           // No filter
        }
    }
}

impl<const N: u8> SourceBuilder<N, false> {
    pub fn edge_or_polarity(mut self, edge_or_polarity: EdgeOrPolarity) -> Self {
        (self.edge_or_polarity_bits, self.polarity_bit) = match edge_or_polarity {
            EdgeOrPolarity::Polarity(Polarity::ActiveHigh) => (0b00, false),
            EdgeOrPolarity::Polarity(Polarity::ActiveLow) => (0b00, true),
            EdgeOrPolarity::Edge(Edge::Rising) => (0b01, false),
            EdgeOrPolarity::Edge(Edge::Falling) => (0b10, false),
            EdgeOrPolarity::Edge(Edge::Both) => (0b11, false),
        };

        self
    }
}

impl<const N: u8> SourceBuilder<N, true> {
    /// Edge sensitivity not available in fast mode
    pub fn polarity(mut self, polarity: Polarity) -> Self {
        (self.edge_or_polarity_bits, self.polarity_bit) = match polarity {
            Polarity::ActiveHigh => (0b00, false),
            Polarity::ActiveLow => (0b00, true),
        };

        self
    }
}

impl<const N: u8> SourceBuilder<N, false>
where
    SourceBuilder<N, false>: ExternalEventBuilder6To10,
{
    pub fn filter(mut self, filter: EevSamplingFilter) -> Self {
        self.filter_bits = filter as _;
        self
    }
}

pub trait ToExternalEventSource<const N: u8, const IS_FAST: bool> {
    fn finalize(self, _calibrated: &mut HrTimCalibrated) -> ExternalEventSource<N, IS_FAST>;
}

macro_rules! impl_eev1_5_to_es {
    ($eev:ident, $N:literal, $eeXsrc:ident, $eeXpol:ident, $eeXsns:ident, $eeXfast:ident) => {
        impl<const IS_FAST: bool> ExternalEventBuilder1To5 for SourceBuilder<$N, IS_FAST> {}

        impl SourceBuilder<$N, false> {
            pub fn fast(self) -> SourceBuilder<$N, true> {
                let SourceBuilder {
                    src_bits,
                    edge_or_polarity_bits,
                    polarity_bit,
                    filter_bits,
                } = self;

                SourceBuilder {
                    src_bits,
                    edge_or_polarity_bits,
                    polarity_bit,
                    filter_bits,
                }
            }
        }

        impl<const IS_FAST: bool> ToExternalEventSource<$N, IS_FAST>
            for SourceBuilder<$N, IS_FAST>
        {
            fn finalize(
                self,
                _calibrated: &mut HrTimCalibrated,
            ) -> ExternalEventSource<$N, IS_FAST> {
                let SourceBuilder {
                    src_bits,
                    edge_or_polarity_bits,
                    polarity_bit,
                    filter_bits: _,
                } = self;

                let common = unsafe { &*HRTIM_COMMON::ptr() };

                // SAFETY: Thanks to, `HrTimCalibrated`, we know we have exclusive access to the register,
                //         we also know no timers are started.
                unsafe {
                    common.eecr1().modify(|_r, w| {
                        w.$eeXsrc()
                            .bits(src_bits)
                            .$eeXpol()
                            .bit(polarity_bit)
                            .$eeXsns()
                            .bits(edge_or_polarity_bits)
                            .$eeXfast()
                            .bit(IS_FAST)
                    });
                }

                ExternalEventSource
            }
        }

        /// EEV$1 event
        impl<const IS_FAST: bool, DST, PSCL> super::event::EventSource<DST, PSCL>
            for ExternalEventSource<$N, IS_FAST>
        {
            const BITS: u32 = 1 << ($N + 20); // EEV1 is at bit 21
        }
    };
}

macro_rules! impl_eev6_10_to_es {
    ($eev:ident, $N:literal, $eeXsrc:ident, $eeXpol:ident, $eeXsns:ident, $eeXf:ident) => {
        impl ExternalEventBuilder6To10 for SourceBuilder<$N, false> {}

        impl ToExternalEventSource<$N, false> for SourceBuilder<$N, false> {
            fn finalize(self, _calibrated: &mut HrTimCalibrated) -> ExternalEventSource<$N, false> {
                let SourceBuilder {
                    src_bits,
                    edge_or_polarity_bits,
                    polarity_bit,
                    filter_bits,
                } = self;

                let common = unsafe { &*HRTIM_COMMON::ptr() };

                unsafe {
                    common.eecr2().modify(|_r, w| {
                        w.$eeXsrc()
                            .bits(src_bits)
                            .$eeXpol()
                            .bit(polarity_bit)
                            .$eeXsns()
                            .bits(edge_or_polarity_bits)
                    });
                    common.eecr3().modify(|_r, w| w.$eeXf().bits(filter_bits));
                }

                ExternalEventSource
            }
        }

        /// EEV$1 event
        impl<DST, PSCL> super::event::EventSource<DST, PSCL> for ExternalEventSource<$N, false> {
            const BITS: u32 = 1 << ($N + 20); // EEV1 is at bit 21
        }
    };
}

impl_eev1_5_to_es!(Eevnt1, 1, ee1src, ee1pol, ee1sns, ee1fast);
impl_eev1_5_to_es!(Eevnt2, 2, ee2src, ee2pol, ee2sns, ee2fast);
impl_eev1_5_to_es!(Eevnt3, 3, ee3src, ee3pol, ee3sns, ee3fast);
impl_eev1_5_to_es!(Eevnt4, 4, ee4src, ee4pol, ee4sns, ee4fast);
impl_eev1_5_to_es!(Eevnt5, 5, ee5src, ee5pol, ee5sns, ee5fast);

impl_eev6_10_to_es!(Eevnt6, 6, ee6src, ee6pol, ee6sns, ee6f);
impl_eev6_10_to_es!(Eevnt7, 7, ee7src, ee7pol, ee7sns, ee7f);
impl_eev6_10_to_es!(Eevnt8, 8, ee8src, ee8pol, ee8sns, ee8f);
impl_eev6_10_to_es!(Eevnt9, 9, ee9src, ee9pol, ee9sns, ee9f);
impl_eev6_10_to_es!(Eevnt10, 10, ee10src, ee10pol, ee10sns, ee10f);

impl<const N: u8, const IS_FAST: bool, TIM, PSCL> super::capture::CaptureEvent<TIM, PSCL>
    for ExternalEventSource<N, IS_FAST>
{
    const BITS: u32 = 1 << (N + 1); // EEV1 is at bit #2 etc
}

impl<const N: u8, const IS_FAST: bool, DST, PSCL> super::event::TimerResetEventSource<DST, PSCL>
    for ExternalEventSource<N, IS_FAST>
{
    const BITS: u32 = 1 << (N + 8); // EEV1 is at bit 9
}
