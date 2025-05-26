#![no_std]

#[cfg(not(any(
    feature = "stm32f334",
    feature = "stm32h742",
    feature = "stm32h743",
    //feature = "stm32h745",
    feature = "stm32h747cm7",
    feature = "stm32h750",
    feature = "stm32h753",
    //feature = "stm32h755",
    //feature = "stm32h757",
    feature = "stm32g474",
    feature = "stm32g484",
)))]
compile_error!(
    "This crate requires one of the following features enabled:
    stm32f334

    stm32h742
    stm32h743
    #stm32h745
    stm32h747cm7
    stm32h750
    stm32h753
    #stm32h755
    #stm32h757

    stm32g474
    stm32g484"
);

pub mod adc_trigger;
pub mod capture;
pub mod compare_register;
pub mod control;
pub mod deadtime;
pub mod event;
pub mod external_event;
pub mod fault;
pub mod output;
pub mod timer;
pub mod timer_eev_cfg;

#[cfg(feature = "stm32f334")]
pub use stm32f3::stm32f3x4 as pac;

#[cfg(feature = "stm32h742")]
pub use stm32h7::stm32h742 as pac;

#[cfg(feature = "stm32h743")]
pub use stm32h7::stm32h743 as pac;

//#[cfg(feature = "stm32h745")]
//pub use stm32h7::stm32h745 as pac;

#[cfg(feature = "stm32h747cm7")]
pub use stm32h7::stm32h747cm7 as pac;

#[cfg(feature = "stm32h750")]
pub use stm32h7::stm32h750 as pac;

#[cfg(feature = "stm32h753")]
pub use stm32h7::stm32h753 as pac;

//#[cfg(feature = "stm32h755")]
//pub use stm32h7::stm32h755 as pac;

//#[cfg(feature = "stm32h757")]
//pub use stm32h7::stm32h757 as pac;

#[cfg(feature = "stm32g474")]
pub use stm32g4::stm32g474 as pac;

#[cfg(feature = "stm32g484")]
pub use stm32g4::stm32g484 as pac;

use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::compare_register::{HrCr1, HrCr2, HrCr3, HrCr4};
use crate::fault::{FaultAction, FaultSource};
use crate::timer::HrTim;
#[cfg(feature = "hrtim_v2")]
use pac::HRTIM_TIMF;
use pac::{HRTIM_COMMON, HRTIM_MASTER, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME};

use capture::{HrCaptCh1, HrCaptCh2};

use self::control::HrPwmControl;

use self::deadtime::DeadtimeConfig;
use self::output::ToHrOut;
use self::timer_eev_cfg::EevCfgs;

/// Internal enum that keeps track of the count settings before PWM is finalized
enum CountSettings {
    //Frequency(Hertz),
    Period(u16),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum HrTimerMode {
    SingleShotNonRetriggerable,
    SingleShotRetriggerable,
    Continuous,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum HrCountingDirection {
    /// Asymetrical up counting mode
    ///
    ///
    ///                   *                  *
    ///  Counting up   *  |               *  |
    ///             *                  *
    ///          *        |         *        |
    ///       *                  *           
    ///    *              |   *              |
    /// *                  *
    /// --------------------------------------
    ///
    /// ```txt
    /// |         *-------*                  *------
    ///           |       |                  |
    /// |         |       |                  |
    ///           |       |                  |
    /// ----------*       *------------------*
    /// ```
    ///
    /// This is the most common mode with least amount of quirks
    Up,

    #[cfg(feature = "hrtim_v2")]
    /// Symmetrical up-down counting mode
    ///
    ///
    /// ```txt
    /// Period-->                  *                      Counting     *
    ///           Counting up   *  |  *     Counting        Up      *  |
    ///                      *           *     down              *
    ///                   *        |        *                 *        |
    ///                *                       *           *
    ///             *              |              *     *              |
    /// 0     -->*                                   *                  
    /// ---------------------------------------------------------------------------
    ///          |         *---------------*         |         *---------------*
    ///                    |       |       |                   |       |       |
    ///          |         |               |         |         |               |
    ///                    |       |       |                   |       |       |
    ///          ----------*               *-------------------*               *---
    /// ```
    ///
    /// NOTE: This is incompatible with
    /// * Auto-delay
    /// * Balanded Idle
    /// * Triggered-half mode
    ///
    /// There is also differences in (including but not limited to) the following areas:
    /// * Counter roll over event
    /// * The events registered with `enable_set_event` will work as normal wen counting up, however when counting down, they will work as rst events.
    /// * The events registered with `enable_rst_event` will work as normal wen counting up, however when counting down, they will work as set events.
    UpDown,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InterleavedMode {
    Disabled,

    /// Dual interleaved or Half mode
    ///
    /// Automatically force
    /// * Cr1 to PERIOD / 2 (not visable through `get_duty`).
    ///
    /// Automatically updates when changing period
    ///
    /// NOTE: Affects Cr1
    Dual,

    #[cfg(feature = "hrtim_v2")]
    /// Triple interleaved mode
    ///
    /// Automatically force
    /// * Cr1 to 1 * PERIOD / 3 and
    /// * Cr2 to 2 * PERIOD / 3
    ///
    /// (not visable through `get_duty`). Automatically updates when changing period.
    ///
    /// NOTE: Must not be used simultaneously with other modes
    /// using CMP2 (dual channel dac trigger and triggered-half modes).
    Triple,

    #[cfg(feature = "hrtim_v2")]
    /// Quad interleaved mode
    ///
    /// Automatically force
    /// * Cr1 to 1 * PERIOD / 4,
    /// * Cr2 to 2 * PERIOD / 4 and
    /// * Cr3 to 3 * PERIOD / 4
    ///
    /// (not visable through `get_duty`). Automatically updates when changing period.
    ///
    /// NOTE: Must not be used simultaneously with other modes
    /// using CMP2 (dual channel dac trigger and triggered-half modes).
    Quad,
}

pub trait HrPwmAdvExt: Sized {
    type PreloadSource;

    fn pwm_advanced<PINS>(
        self,
        _pins: PINS,
    ) -> HrPwmBuilder<Self, PsclDefault, Self::PreloadSource, PINS>
    where
        PINS: ToHrOut<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Polarity {
    ActiveHigh,
    ActiveLow,
}

pub trait DacStepTrigger {
    const IS_CR2: bool;
    const IS_OUT1_RST: bool;
    const DCDS_BIT: Option<bool>;
}

pub trait DacResetTrigger {
    const IS_TIM_RST: bool;
    const IS_OUT1_SET: bool;
    const DCDR_BIT: Option<bool>;
}

pub struct NoDacTrigger;

impl DacStepTrigger for NoDacTrigger {
    const IS_CR2: bool = false;
    const IS_OUT1_RST: bool = false;

    const DCDS_BIT: Option<bool> = None;
}

impl DacResetTrigger for NoDacTrigger {
    const IS_TIM_RST: bool = false;
    const IS_OUT1_SET: bool = false;

    const DCDR_BIT: Option<bool> = None;
}

/// The trigger is generated on counter reset or roll-over event
pub struct DacResetOnCounterReset;
impl DacResetTrigger for DacResetOnCounterReset {
    const IS_TIM_RST: bool = true;
    const IS_OUT1_SET: bool = false;
    const DCDR_BIT: Option<bool> = Some(false);
}

/// The trigger is generated on output 1 set event
pub struct DacResetOnOut1Set;
impl DacResetTrigger for DacResetOnOut1Set {
    const IS_TIM_RST: bool = false;
    const IS_OUT1_SET: bool = true;
    const DCDR_BIT: Option<bool> = Some(true);
}

/// The trigger is generated on compare 2 event repeatedly
///
/// The compare 2 has a particular operating mode when using `OnCmp2`. The active
/// comparison value is automatically updated as soon as a compare match
/// has occured, so that the trigger can be repeated periodically with a period
/// equal to the CMP2 value.
///
/// NOTE:
/// The dual channel DAC trigger with `OnCmp2` must not be
/// used simultaneously with modes using CMP2 (triple / quad interleaved
/// and triggered-half modes).
///
/// Example:
/// Let’s consider a counter period = 8192. Dividing 8192 by 6 yields 1365.33.
/// – Round down value: 1365: 7 triggers are generated, the 6th and 7th being very
/// close (respectively for counter = 8190 and 8192)
/// – Round up value:1366: 6 triggers are generated. The 6th trigger on dac_step_trg
/// (for counter = 8192) is aborted by the counter roll-over from 8192 to 0.
pub struct DacStepOnCmp2;
impl DacStepTrigger for DacStepOnCmp2 {
    const IS_CR2: bool = true;
    const IS_OUT1_RST: bool = false;
    const DCDS_BIT: Option<bool> = Some(false);
}

/// The trigger is generated on output 1 rst event
pub struct DacStepOnOut1Rst;
impl DacStepTrigger for DacStepOnOut1Rst {
    const IS_CR2: bool = false;
    const IS_OUT1_RST: bool = true;
    const DCDS_BIT: Option<bool> = Some(true);
}

/// HrPwmBuilder is used to configure advanced HrTim PWM features
pub struct HrPwmBuilder<
    TIM,
    PSCL,
    PS,
    PINS,
    DacRst: DacResetTrigger = NoDacTrigger,
    DacStp: DacStepTrigger = NoDacTrigger,
> {
    _tim: PhantomData<TIM>,
    _prescaler: PhantomData<PSCL>,
    pub pins: PINS,
    timer_mode: HrTimerMode,
    counting_direction: HrCountingDirection,
    //base_freq: HertzU64,
    count: CountSettings,
    preload_source: Option<PS>,
    fault_enable_bits: u8,
    fault1_bits: u8,
    fault2_bits: u8,
    enable_push_pull: bool,
    interleaved_mode: InterleavedMode, // Also includes half mode
    repetition_counter: u8,
    deadtime: Option<DeadtimeConfig>,
    enable_repetition_interrupt: bool,
    eev_cfg: EevCfgs<TIM>,
    // TODO Add DAC triggers for stm32f334 (RM0364 21.3.19) and stm32h7 if applicable
    dac_rst_trigger: PhantomData<DacRst>,
    dac_stp_trigger: PhantomData<DacStp>,
    out1_polarity: Polarity,
    out2_polarity: Polarity,
}

pub struct HrParts<
    TIM,
    PSCL,
    OUT,
    DacRst: DacResetTrigger = NoDacTrigger,
    DacStp: DacStepTrigger = NoDacTrigger,
> {
    pub timer: HrTim<TIM, PSCL, HrCaptCh1<TIM, PSCL>, HrCaptCh2<TIM, PSCL>, DacRst>,

    pub cr1: HrCr1<TIM, PSCL>,
    pub cr2: HrCr2<TIM, PSCL, DacStp>,
    pub cr3: HrCr3<TIM, PSCL>,
    pub cr4: HrCr4<TIM, PSCL>,

    pub out: OUT,
    pub dma_channel: timer::DmaChannel<TIM>,
}

pub enum PreloadSource {
    /// Preloaded registers are updated on counter roll over or counter reset
    OnCounterReset,

    /// Preloaded registers are updated by master timer update
    OnMasterTimerUpdate,

    /// Prealoaded registers are updaten when the counter rolls over and the repetition counter is 0
    OnRepetitionUpdate,
}

pub enum MasterPreloadSource {
    /// Prealoaded registers are updaten when the master counter rolls over and the master repetition counter is 0
    OnMasterRepetitionUpdate,
}

macro_rules! hrtim_finalize_body {
    ($this:expr, $PreloadSource:ident, $TIMX:ident, [$($out:ident)*]) => {{
        let tim = unsafe { &*$TIMX::ptr() };
        let (period, prescaler_bits) = match $this.count {
            CountSettings::Period(period) => (period as u32, PSCL::BITS as u16),
        };

        let (half, _intlvd) = match $this.interleaved_mode {
            InterleavedMode::Disabled => (false, 0b00),
            InterleavedMode::Dual => (true, 0b00),
            #[cfg(feature = "hrtim_v2")]
            InterleavedMode::Triple => (false, 0b01),
            #[cfg(feature = "hrtim_v2")]
            InterleavedMode::Quad => (false, 0b10),
        };

        // Write prescaler and any special modes
        tim.cr().modify(|_r, w| unsafe {
            w
                // Enable Continuous mode
                .cont().bit($this.timer_mode == HrTimerMode::Continuous)
                .retrig().bit($this.timer_mode == HrTimerMode::SingleShotRetriggerable)

                // TODO: add support for more modes

                // half/double interleaved mode
                .half().bit(half)

                // Set prescaler
                .ckpsc().bits(prescaler_bits as u8)
        });

        #[cfg(feature = "hrtim_v2")]
        tim.cr().modify(|_r, w| unsafe {
            // Interleaved mode
            w.intlvd().bits(_intlvd)
        });

        $(
            // Only available for timers with outputs(not HRTIM_MASTER)
            #[allow(unused)]
            let $out = ();

            // TODO Add DAC triggers for stm32f334 (RM0364 21.3.19) and stm32h7 if applicable
            #[cfg(feature = "hrtim_v2")]
            tim.cr2().modify(|_r, w| {
                // Set counting direction
                w.udm().bit($this.counting_direction == HrCountingDirection::UpDown);
                assert!(DacRst::DCDR_BIT.is_some() == DacStp::DCDS_BIT.is_some());

                if let (Some(rst), Some(stp)) = (DacRst::DCDR_BIT, DacStp::DCDS_BIT) {
                    w
                        .dcde().set_bit()
                        .dcds().bit(stp as u8 != 0)
                        .dcdr().bit(rst as u8 != 0);
                }

                w
            });

            tim.cr().modify(|_r, w|
                // Push-Pull mode
                w.pshpll().bit($this.enable_push_pull)
            );
        )*

        // Write period
        tim.perr().write(|w| unsafe { w.per().bits(period as u16) });

        // Enable fault sources and lock configuration
        $(unsafe {
            // Only available for timers with outputs(not HRTIM_MASTER)
            #[allow(unused)]
            let $out = ();

            // Enable fault sources
            let fault_enable_bits = $this.fault_enable_bits as u32;
            tim.fltr().write(|w| w
                .flt1en().bit(fault_enable_bits & (1 << 0) != 0)
                .flt2en().bit(fault_enable_bits & (1 << 1) != 0)
                .flt3en().bit(fault_enable_bits & (1 << 2) != 0)
                .flt4en().bit(fault_enable_bits & (1 << 3) != 0)
                .flt5en().bit(fault_enable_bits & (1 << 4) != 0)
            );
            #[cfg(feature = "hrtim_v2")]
            tim.fltr().modify(|_, w| w.flt6en().bit(fault_enable_bits & (1 << 5) != 0));

            // ... and lock configuration
            tim.fltr().modify(|_r, w| w.fltlck().set_bit());

            tim.outr().modify(|_r, w| w
                // Set actions on fault for both outputs
                .fault1().bits($this.fault1_bits)
                .fault2().bits($this.fault2_bits)

                // Set output polarity for both outputs
                .pol1().bit(matches!($this.out1_polarity, Polarity::ActiveLow))
                .pol2().bit(matches!($this.out2_polarity, Polarity::ActiveLow))
            );
            if let Some(deadtime) = $this.deadtime {
                let DeadtimeConfig {
                    prescaler,
                    deadtime_rising_value,
                    deadtime_rising_sign,
                    deadtime_falling_value,
                    deadtime_falling_sign,
                } = deadtime;

                // SAFETY: DeadtimeConfig makes sure rising and falling values are valid
                // and DeadtimePrescaler has its own garantuee
                tim.dtr().modify(|_r, w| w
                    .dtprsc().bits(prescaler as u8)
                    .dtr().bits(deadtime_rising_value)
                    .sdtr().bit(deadtime_rising_sign)
                    .dtf().bits(deadtime_falling_value)
                    .sdtf().bit(deadtime_falling_sign)

                    // Lock configuration
                    .dtflk().set_bit()
                    .dtfslk().set_bit()
                    .dtrlk().set_bit()
                    .dtrslk().set_bit()
                );
                tim.outr().modify(|_r, w| w.dten().set_bit());
            }

            // External event configs
            let eev_cfg = $this.eev_cfg.clone();
            tim.eefr1().write(|w| w
                .ee1ltch().bit(eev_cfg.eev1.latch_bit).ee1fltr().bits(eev_cfg.eev1.filter_bits)
                .ee2ltch().bit(eev_cfg.eev2.latch_bit).ee2fltr().bits(eev_cfg.eev2.filter_bits)
                .ee3ltch().bit(eev_cfg.eev3.latch_bit).ee3fltr().bits(eev_cfg.eev3.filter_bits)
                .ee4ltch().bit(eev_cfg.eev4.latch_bit).ee4fltr().bits(eev_cfg.eev4.filter_bits)
                .ee5ltch().bit(eev_cfg.eev5.latch_bit).ee5fltr().bits(eev_cfg.eev5.filter_bits)
            );
            tim.eefr2().write(|w| w
                .ee6ltch().bit(eev_cfg.eev6.latch_bit).ee6fltr().bits(eev_cfg.eev6.filter_bits)
                .ee7ltch().bit(eev_cfg.eev7.latch_bit).ee7fltr().bits(eev_cfg.eev7.filter_bits)
                .ee8ltch().bit(eev_cfg.eev8.latch_bit).ee8fltr().bits(eev_cfg.eev8.filter_bits)
                .ee9ltch().bit(eev_cfg.eev9.latch_bit).ee9fltr().bits(eev_cfg.eev9.filter_bits)
                .ee10ltch().bit(eev_cfg.eev10.latch_bit).ee10fltr().bits(eev_cfg.eev10.filter_bits)
            );
            #[cfg(feature = "hrtim_v2")]
            tim.eefr3().write(|w| w
                .eevace().bit(eev_cfg.event_counter_enable_bit)
                // External Event A Counter Reset"]
                //.eevacres().bit()
                .eevarstm().bit(eev_cfg.event_counter_reset_mode_bit)
                .eevasel().bits(eev_cfg.event_counter_source_bits)
                .eevacnt().bits(eev_cfg.event_counter_threshold_bits)
            );
        })*


        hrtim_finalize_body!($PreloadSource, $this, tim);

        // Set repetition counter
        unsafe { tim.repr().write(|w| w.rep().bits($this.repetition_counter)); }

        // Enable interrupts
        tim.dier().modify(|_r, w| w.repie().bit($this.enable_repetition_interrupt));

        // Start timer
        //let master = unsafe { &*HRTIM_MASTER::ptr() };
        //master.mcr.modify(|_r, w| { w.$tXcen().set_bit() });
    }};

    (PreloadSource, $this:expr, $tim:expr) => {{
        match $this.preload_source {
            Some(PreloadSource::OnCounterReset) => {
                $tim.cr().modify(|_r, w| w
                    .trstu().set_bit()
                    .preen().set_bit()
                );
            },
            Some(PreloadSource::OnMasterTimerUpdate) => {
                $tim.cr().modify(|_r, w| w
                    .mstu().set_bit()
                    .preen().set_bit()
                );
            }
            Some(PreloadSource::OnRepetitionUpdate) => {
                $tim.cr().modify(|_r, w| w
                    .trepu().set_bit()
                    .preen().set_bit()
                );
            }
            None => ()
        }
    }};

    (MasterPreloadSource, $this:expr, $tim:expr) => {{
        match $this.preload_source {
            Some(MasterPreloadSource::OnMasterRepetitionUpdate) => {
                $tim.cr().modify(|_r, w| w
                    .mrepu().set_bit()
                    .preen().set_bit()
                );
            }
            None => ()
        }
    }};
}

macro_rules! hrtim_common_methods {
    ($TIMX:ident, $PS:ident) => {
        /// Set the prescaler; PWM count runs at base_frequency/(prescaler+1)
        pub fn prescaler<P>(
            self,
            _prescaler: P,
        ) -> HrPwmBuilder<$TIMX, P, $PS, PINS, DacRst, DacStp>
        where
            P: HrtimPrescaler,
        {
            let HrPwmBuilder {
                _tim,
                _prescaler: _,
                pins,
                timer_mode,
                fault_enable_bits,
                fault1_bits,
                fault2_bits,
                enable_push_pull,
                interleaved_mode,
                counting_direction,
                //base_freq,
                count,
                preload_source,
                repetition_counter,
                deadtime,
                enable_repetition_interrupt,
                eev_cfg,
                dac_rst_trigger,
                dac_stp_trigger,
                out1_polarity,
                out2_polarity,
            } = self;

            let period = match count {
                CountSettings::Period(period) => period,
            };

            let count = CountSettings::Period(period);

            HrPwmBuilder {
                _tim,
                _prescaler: PhantomData,
                pins,
                timer_mode,
                fault_enable_bits,
                fault1_bits,
                fault2_bits,
                enable_push_pull,
                interleaved_mode,
                counting_direction,
                //base_freq,
                count,
                preload_source,
                repetition_counter,
                deadtime,
                enable_repetition_interrupt,
                eev_cfg,
                dac_rst_trigger,
                dac_stp_trigger,
                out1_polarity,
                out2_polarity,
            }
        }

        pub fn timer_mode(mut self, timer_mode: HrTimerMode) -> Self {
            self.timer_mode = timer_mode;
            self
        }

        // TODO: Allow setting multiple?
        pub fn preload(mut self, preload_source: $PS) -> Self {
            self.preload_source = Some(preload_source);
            self
        }

        /// Set the period; PWM count runs from 0 to period, repeating every (period+1) counts
        pub fn period(mut self, period: u16) -> Self {
            self.count = CountSettings::Period(period);
            self
        }

        /// Set repetition counter, useful to reduce interrupts generated
        /// from timer by a factor (repetition_counter + 1)
        pub fn repetition_counter(mut self, repetition_counter: u8) -> Self {
            self.repetition_counter = repetition_counter;
            self
        }

        pub fn enable_repetition_interrupt(mut self) -> Self {
            self.enable_repetition_interrupt = true;
            self
        }

        pub fn eev_cfg(mut self, eev_cfg: EevCfgs<$TIMX>) -> Self {
            self.eev_cfg = eev_cfg;
            self
        }

        #[cfg(feature = "hrtim_v2")]
        /// Enable dac trigger with provided settings
        ///
        /// ### Edge-aligned slope compensation
        ///
        /// The DAC’s sawtooth starts on PWM period beginning and
        /// multiple triggers are generated during the timer period
        /// with a trigger interval equal to the CMP2 value.
        ///
        /// NOTE:
        /// Must not be used simultaneously with modes using
        /// CMP2 (triple / quad interleaved and triggered-half modes).
        /// reset_trigger: DacRstTrg::OnCounterReset,
        /// step_trigger: DacStpTrg::OnCmp2,
        ///
        /// ### Center-aligned slope compensation
        ///
        /// The DAC’s sawtooth starts on the output 1 set event and
        /// multiple triggers are generated during the timer period
        /// with a trigger interval equal to the CMP2 value.
        ///
        /// NOTE:
        /// Must not be used simultaneously with modes using
        /// CMP2 (triple / quad interleaved and triggered-half modes).
        ///
        /// NOTE:
        /// In centered-pattern mode, it is mandatory to have an even
        /// number of triggers per switching period, so as to avoid
        /// unevenly spaced triggers around counter’s peak value.
        ///
        /// reset_trigger: DacRstTrg::OnOut1Set,
        /// step_trigger: DacStpTrg::OnCmp2,
        ///
        /// ### Hysteretic controller - Reset on CounterReset
        ///
        /// 2 triggers are generated per PWM period.
        /// In edge-aligned mode the triggers are generated on counter
        /// reset or rollover and the output is reset
        ///
        /// reset_trigger: [`DacResetOnCounterReset,
        /// step_trigger: [`DacStepOnOut1Rst,
        ///
        /// ### Hysteretic controller - Reset on Out1Set
        ///
        /// 2 triggers are generated per PWM period.
        /// In center-aligned mode the triggers are generated when the output is
        /// set and when it is reset.
        ///
        /// reset_trigger: [`DacResetOnOut1Set`],
        /// step_trigger: [`DacStepOnOut1Rst`],
        pub fn dac_trigger_cfg<R: DacResetTrigger, S: DacStepTrigger>(
            self,
            _rst: R,
            _step: S,
        ) -> HrPwmBuilder<$TIMX, PSCL, $PS, PINS, R, S> {
            let HrPwmBuilder {
                _tim,
                _prescaler: _,
                pins,
                timer_mode,
                fault_enable_bits,
                fault1_bits,
                fault2_bits,
                enable_push_pull,
                interleaved_mode,
                counting_direction,
                //base_freq,
                count,
                preload_source,
                repetition_counter,
                deadtime,
                enable_repetition_interrupt,
                eev_cfg,
                dac_rst_trigger: _,
                dac_stp_trigger: _,
                out1_polarity,
                out2_polarity,
            } = self;

            HrPwmBuilder {
                _tim,
                _prescaler: PhantomData,
                pins,
                timer_mode,
                fault_enable_bits,
                fault1_bits,
                fault2_bits,
                enable_push_pull,
                interleaved_mode,
                counting_direction,
                //base_freq,
                count,
                preload_source,
                repetition_counter,
                deadtime,
                enable_repetition_interrupt,
                eev_cfg,
                dac_rst_trigger: PhantomData,
                dac_stp_trigger: PhantomData,
                out1_polarity,
                out2_polarity,
            }
        }
    };
}

// Implement PWM configuration for timer
macro_rules! hrtim_hal {
    ($($TIMX:ident: $($out:ident)*,)+) => {
        $(
            impl HrPwmAdvExt for $TIMX {
                type PreloadSource = PreloadSource;

                fn pwm_advanced<PINS>(
                    self,
                    pins: PINS,
                ) -> HrPwmBuilder<Self, PsclDefault, Self::PreloadSource, PINS>
                where
                    PINS: ToHrOut<$TIMX>,
                {
                    // TODO: That 32x factor... Is that included below, or should we
                    // do that? Also that will likely risk overflowing u32 since
                    // 170MHz * 32 = 5.44GHz > u32::MAX.Hz()
                    //let clk = HertzU64::from(HRTIM_COMMON::get_timer_frequency(&rcc.clocks)) * 32;

                    HrPwmBuilder {
                        _tim: PhantomData,
                        _prescaler: PhantomData,
                        pins,
                        timer_mode: HrTimerMode::Continuous,
                        fault_enable_bits: 0b000000,
                        fault1_bits: 0b00,
                        fault2_bits: 0b00,
                        counting_direction: HrCountingDirection::Up,
                        //base_freq: clk,
                        count: CountSettings::Period(u16::MAX),
                        preload_source: None,
                        enable_push_pull: false,
                        interleaved_mode: InterleavedMode::Disabled,
                        repetition_counter: 0,
                        deadtime: None,
                        enable_repetition_interrupt: false,
                        eev_cfg: EevCfgs::default(),
                        dac_rst_trigger: PhantomData,
                        dac_stp_trigger: PhantomData,
                        out1_polarity: Polarity::ActiveHigh,
                        out2_polarity: Polarity::ActiveHigh,
                    }
                }
            }

            impl<PSCL, PINS, DacRst, DacStp>
                HrPwmBuilder<$TIMX, PSCL, PreloadSource, PINS, DacRst, DacStp>
            where
                DacRst: DacResetTrigger,
                DacStp: DacStepTrigger,
                PSCL: HrtimPrescaler,
                PINS: ToHrOut<$TIMX, DacRst, DacStp>,
            {
                // For HAL writers:
                // Make sure to connect gpios after calling this function and then it should be safe to
                // conjure an instance of HrParts<$TIMX, PSCL, PINS::Out<PSCL>>
                pub fn _init(self, _control: &mut HrPwmControl) -> PINS {
                    hrtim_finalize_body!(self, PreloadSource, $TIMX, [$($out)*]);
                    self.pins
                }

                hrtim_common_methods!($TIMX, PreloadSource);

                pub fn with_fault_source<FS>(mut self, _fault_source: FS) -> Self
                    where FS: FaultSource
                {
                    self.fault_enable_bits |= FS::ENABLE_BITS;

                    self
                }

                pub fn fault_action1(mut self, fault_action1: FaultAction) -> Self {
                    self.fault1_bits = fault_action1 as _;

                    self
                }

                pub fn fault_action2(mut self, fault_action2: FaultAction) -> Self {
                    self.fault2_bits = fault_action2 as _;

                    self
                }

                pub fn out1_polarity(mut self, polarity: Polarity) -> Self {
                    self.out1_polarity = polarity;

                    self
                }

                pub fn out2_polarity(mut self, polarity: Polarity) -> Self {
                    self.out2_polarity = polarity;

                    self
                }

                /// Enable or disable Push-Pull mode
                ///
                /// Enabling Push-Pull mode will make output 1 and 2
                /// alternate every period with one being
                /// inactive and the other getting to output its wave form
                /// as normal
                ///
                ///         ----           .                ----
                ///out1    |    |          .               |    |
                ///        |    |          .               |    |
                /// --------    ----------------------------    --------------------
                ///        .                ------         .                ------
                ///out2    .               |      |        .               |      |
                ///        .               |      |        .               |      |
                /// ------------------------    ----------------------------      --
                ///
                /// NOTE: setting this will overide any 'Swap Mode' set
                pub fn push_pull_mode(mut self, enable: bool) -> Self {
                    // TODO: add check for incompatible modes
                    self.enable_push_pull = enable;

                    self
                }

                /// Set counting direction
                ///
                /// See [`HrCountingDirection`]
                pub fn counting_direction(mut self, counting_direction: HrCountingDirection) -> Self {
                    self.counting_direction = counting_direction;

                    self
                }

                /// Set interleaved or half modes
                ///
                /// NOTE: Check [`InterleavedMode`] for more info about special cases
                pub fn interleaved_mode(mut self, mode: InterleavedMode) -> Self {
                    self.interleaved_mode = mode;

                    self
                }

                pub fn deadtime(mut self, deadtime: DeadtimeConfig) -> Self {
                    self.deadtime = Some(deadtime);

                    self
                }

                //pub fn swap_mode(mut self, enable: bool) -> Self
            }
        )+
    };
}

impl HrPwmAdvExt for HRTIM_MASTER {
    type PreloadSource = MasterPreloadSource;

    fn pwm_advanced<PINS>(
        self,
        pins: PINS,
    ) -> HrPwmBuilder<Self, PsclDefault, Self::PreloadSource, PINS>
    where
        PINS: ToHrOut<HRTIM_MASTER>,
    {
        // TODO: That 32x factor... Is that included below, or should we
        // do that? Also that will likely risk overflowing u32 since
        // 170MHz * 32 = 5.44GHz > u32::MAX.Hz()
        //let clk = HertzU64::from(HRTIM_COMMON::get_timer_frequency(&rcc.clocks)) * 32;

        HrPwmBuilder {
            _tim: PhantomData,
            _prescaler: PhantomData,
            pins,
            timer_mode: HrTimerMode::Continuous,
            fault_enable_bits: 0b000000,
            fault1_bits: 0b00,
            fault2_bits: 0b00,
            counting_direction: HrCountingDirection::Up,
            //base_freq: clk,
            count: CountSettings::Period(u16::MAX),
            preload_source: None,
            enable_push_pull: false,
            interleaved_mode: InterleavedMode::Disabled,
            repetition_counter: 0,
            deadtime: None,
            enable_repetition_interrupt: false,
            eev_cfg: EevCfgs::default(),
            dac_rst_trigger: PhantomData,
            dac_stp_trigger: PhantomData,
            out1_polarity: Polarity::ActiveHigh,
            out2_polarity: Polarity::ActiveHigh,
        }
    }
}

impl<PSCL, PINS, DacRst, DacStp>
    HrPwmBuilder<HRTIM_MASTER, PSCL, MasterPreloadSource, PINS, DacRst, DacStp>
where
    DacRst: DacResetTrigger,
    DacStp: DacStepTrigger,
    PSCL: HrtimPrescaler,
    PINS: ToHrOut<HRTIM_MASTER>,
{
    pub fn finalize(self, _control: &mut HrPwmControl) -> HrParts<HRTIM_MASTER, PSCL, PINS> {
        hrtim_finalize_body!(self, MasterPreloadSource, HRTIM_MASTER, []);

        unsafe { MaybeUninit::uninit().assume_init() }
    }

    hrtim_common_methods!(HRTIM_MASTER, MasterPreloadSource);
}

hrtim_hal! {
    HRTIM_TIMA: out,
    HRTIM_TIMB: out,
    HRTIM_TIMC: out,
    HRTIM_TIMD: out,
    HRTIM_TIME: out,
}

#[cfg(feature = "hrtim_v2")]
hrtim_hal! {
    HRTIM_TIMF: out,
}

/// # Safety
/// Only implement for valid prescalers with correct values
pub unsafe trait HrtimPrescaler: Default {
    const BITS: u8;
    const VALUE: u8;

    /// Minimum allowed value for compare registers used with the timer with this prescaler
    ///
    /// NOTE: That for CR1 and CR3, 0 is also allowed
    const MIN_CR: u16;

    /// Maximum allowed value for compare registers used with the timer with this prescaler
    const MAX_CR: u16;
}

macro_rules! impl_pscl {
    ($($t:ident => $b:literal, $v:literal, $min:literal, $max:literal)+) => {$(
        #[derive(Copy, Clone, Default)]
        pub struct $t;
        unsafe impl HrtimPrescaler for $t {
            const BITS: u8 = $b;
            const VALUE: u8 = $v;
            const MIN_CR: u16 = $min;
            const MAX_CR: u16 = $max;
        }
    )+};
}

#[cfg(any(feature = "stm32f3", feature = "stm32g4"))]
pub type PsclDefault = Pscl128;

#[cfg(feature = "stm32h7")]
pub type PsclDefault = Pscl4;

#[cfg(any(feature = "stm32f3", feature = "stm32g4"))]
impl_pscl! {
    Pscl1   => 0b000,   1, 0x0060, 0xFFDF
    Pscl2   => 0b001,   2, 0x0030, 0xFFEF
    Pscl4   => 0b010,   4, 0x0018, 0xFFF7
    Pscl8   => 0b011,   8, 0x000C, 0xFFFB
    Pscl16  => 0b100,  16, 0x0006, 0xFFFD
    Pscl32  => 0b101,  32, 0x0003, 0xFFFD
    Pscl64  => 0b110,  64, 0x0003, 0xFFFD
    Pscl128 => 0b111, 128, 0x0003, 0xFFFD
}

#[cfg(feature = "stm32h7")]
impl_pscl! {
    Pscl1 => 0b101, 1, 0x0003, 0xFFFD
    Pscl2 => 0b110, 2, 0x0003, 0xFFFD
    Pscl4 => 0b111, 4, 0x0003, 0xFFFD
}

/*
/// HrTim timer
struct TimerHrTim<PSC>(PhantomData<PSC>);

impl<PSC: HrtimPrescaler> pwm::TimerType for TimerHrTim<PSC> {
    // Period calculator for 16-bit hrtimers
    //
    // NOTE: This function will panic if the calculated period can not fit into 16 bits
    fn calculate_frequency(base_freq: HertzU64, freq: Hertz, alignment: Alignment) -> (u32, u16) {
        let ideal_period = pwm::Timer32Bit::calculate_frequency(base_freq, freq, alignment).0 + 1;

        let prescale = u32::from(PSC::VALUE);

        // Round to the nearest period
        let period = (ideal_period + (prescale >> 1)) / prescale - 1;

        // It IS possible to fail this assert
        assert!(period <= 0xFFFF);

        (period, PSC::BITS.into())
    }
}
*/
