use super::timer::{self, Ch1, Ch2, ChExt, InstanceX};
use crate::ext::{CptcrW, MasterExt, TimExt};
use core::marker::PhantomData;

pub struct Dma;
pub struct NoDma;

/// Type alias for the default capture for channel 1
pub type HrCaptCh1<TIM, PSCL> = HrCapt<TIM, PSCL, Ch1, NoDma>;

/// Type alias for the default capture for channel 2
pub type HrCaptCh2<TIM, PSCL> = HrCapt<TIM, PSCL, Ch2, NoDma>;

pub struct HrCapt<TIM, PSCL, CH, DMA> {
    _x: PhantomData<(TIM, PSCL, CH, DMA)>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub enum CountingDirection {
    Up = 0,
    #[cfg(feature = "hrtim_v2")]
    Down = 1,
}

/// Implemented for
/// * TIM's update event
/// * EEVT1-10
///
/// TODO:
/// * All neighbor timers CMP1, CPM2, OUT1_RST and OUT1_SET events
pub trait CaptureEvent<TIM, PSCL> {
    const BITS: u32;
}

/// Trait for capture channels used for capturing edges
///
/// ```
/// let capture: HrCapt<_, _, _> = todo!();
/// if capture.is_pending() {
///     let (value, dir) = capture.get_last();
///     capture.clear_interrupt();
///     defmt::info!("Edge captured at counter value: {}, with: {}", value, dir);
/// }
/// ```
///
/// or alternatively
///
/// ```
/// let capture: HrCapt<_, _, _> = todo!();
/// if let Some((value, dir)) = capture.get() {
///     defmt::info!("Edge captured at counter value: {}, with: {}", value, dir);
/// }
/// ```
pub trait HrCapture {
    /// Try to get the capture value
    ///
    /// Returns none if edge has been captured since last time
    ///
    /// NOTE: This function will use [`Self::is_pending`] to chech if there is a value available and
    /// [`Self::clear_interrupt`] to clear it.
    fn get(&mut self) -> Option<(u16, CountingDirection)> {
        if self.is_pending() {
            let value = self.get_last();
            self.clear_interrupt();
            Some(value)
        } else {
            None
        }
    }

    /// Get number of ticks relative to beginning of upcounting
    ///
    /// where captures during down counting count as negative (before the upcount)
    ///
    /// ```text
    ///              Counter
    /// ----------------------------------   <--- period
    /// \               ^               /
    ///    \            |            /
    ///       \         |         /
    ///          \      |      /
    /// Down count  \   |   /   Up count
    ///                \|/
    /// <-------------- 0 --------------> t
    /// Negative result | positive result
    /// ```
    ///
    /// NOTE: This function will use [`Self::is_pending`] to chech if there is a value available and
    /// [`Self::clear_interrupt`] to clear it.
    fn get_signed(&mut self, period: u16) -> Option<i32> {
        if self.is_pending() {
            let value = self.get_last_signed(period);
            self.clear_interrupt();
            Some(value)
        } else {
            None
        }
    }

    fn get_last(&self) -> (u16, CountingDirection);

    /// Get number of ticks relative to beginning of upcounting
    ///
    /// where captures during down counting count as negative (before the upcount)
    ///
    /// ```text
    ///              Counter
    /// ----------------------------------   <--- period
    /// \               ^               /
    ///    \            |            /
    ///       \         |         /
    ///          \      |      /
    /// Down count  \   |   /   Up count
    ///                \|/
    /// <-------------- 0 --------------> t
    /// Negative result | positive result
    /// ```
    fn get_last_signed(&self, #[allow(unused_variables)] period: u16) -> i32 {
        let (value, dir) = self.get_last();

        // The capture counter always counts up and restarts at period
        match dir {
            CountingDirection::Up => i32::from(value),
            #[cfg(feature = "hrtim_v2")]
            CountingDirection::Down => i32::from(value) - i32::from(period),
        }
    }

    fn clear_interrupt(&mut self);

    fn is_pending(&self) -> bool;
}

pub fn dma_value_to_dir_and_value(x: u32) -> (u16, CountingDirection) {
    let value = (x & 0xFFFF) as u16;
    #[cfg(feature = "hrtim_v2")]
    match x & (1 << 16) != 0 {
        true => (value, CountingDirection::Down),
        false => (value, CountingDirection::Up),
    }

    #[cfg(any(feature = "hrtim_v1", feature = "hrtim_v1_1"))]
    (value, CountingDirection::Up)
}

pub fn dma_value_to_signed(x: u32, #[allow(unused_variables)] period: u16) -> i32 {
    let (value, dir) = dma_value_to_dir_and_value(x);

    // The capture counter always counts up and restarts at period
    match dir {
        CountingDirection::Up => i32::from(value),
        #[cfg(feature = "hrtim_v2")]
        CountingDirection::Down => i32::from(value) - i32::from(period),
    }
}

impl<TIM: InstanceX, CH: ChExt, PSCL> HrCapt<TIM, PSCL, CH, NoDma> {
    /// Add event to capture
    ///
    /// If multiple events are added, they will be ORed together meaning
    /// that a capture will be trigger if any one of the events triggers
    pub fn add_event<E: CaptureEvent<TIM, PSCL>>(&mut self, _event: &E) {
        let tim = unsafe { &*TIM::ptr() };

        // SAFETY: We are the only one with access to cptXYcr
        unsafe {
            tim.cptcr(CH::CH).modify(|r, w| w.bits(r.bits() | E::BITS));
        }
    }

    /// Remove event to capture
    pub fn remove_event<E: CaptureEvent<TIM, PSCL>>(&mut self, _event: &E) {
        let tim = unsafe { &*TIM::ptr() };

        // SAFETY: We are the only one with access to cptXYcr
        unsafe {
            tim.cptcr(CH::CH).modify(|r, w| w.bits(r.bits() & !E::BITS));
        }
    }

    /// Force capture trigger now
    pub fn trigger_now(&mut self) {
        // SAFETY: We are the only one with access to cptXYcr
        let tim = unsafe { &*TIM::ptr() };

        tim.cptcr(CH::CH).modify(|_, w| w.set_swcpt());
    }

    // TODO: It would be sufficient to instead of hr_control only require exclusive access to the owning timer
    // however that would be hard to do since typically the capture device is a field of that same timer.
    // Would it make more sense to have this method direcly on HrTim instead?
    pub fn enable_interrupt(&mut self, enable: bool, _hr_control: &mut super::HrPwmControl) {
        let tim = unsafe { &*TIM::ptr() };

        tim.dier().modify(|_r, w| w.cptie(CH::CH as _).bit(enable));
    }

    pub fn enable_dma(self, _ch: timer::DmaChannel<TIM>) -> HrCapt<TIM, PSCL, CH, Dma> {
        // SAFETY: We own the only insance of this timers dma channel, no one else can do this
        let tim = unsafe { &*TIM::ptr() };
        tim.dier().modify(|_r, w| w.cptde(CH::CH as _).set_bit());
        HrCapt { _x: PhantomData }
    }
}

impl<TIM: InstanceX, CH: ChExt, PSCL, DMA> HrCapture for HrCapt<TIM, PSCL, CH, DMA> {
    fn get_last(&self) -> (u16, CountingDirection) {
        let tim = unsafe { &*TIM::ptr() };
        let data = tim.cptr(CH::CH).read();

        #[cfg(feature = "hrtim_v2")]
        let dir = match data.dir().bit() {
            true => CountingDirection::Down,
            false => CountingDirection::Up,
        };
        #[cfg(any(feature = "hrtim_v1", feature = "hrtim_v1_1"))]
        let dir = CountingDirection::Up;

        let value = data.cpt().bits();

        (value, dir)
    }

    fn clear_interrupt(&mut self) {
        let tim = unsafe { &*TIM::ptr() };

        // No need for exclusive access since this is a write only register
        tim.icr().write(|w| w.cptc(CH::CH as _).clear());
    }

    fn is_pending(&self) -> bool {
        let tim = unsafe { &*TIM::ptr() };

        // No need for exclusive access since this is a read only register
        tim.isr().read().cpt(CH::CH as _).bit()
    }
}
