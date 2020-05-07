//! Clock configuration for the peripherals

#[cfg(feature = "82x")]
mod clocksource_82x;
#[cfg(feature = "845")]
mod clocksource_845;

#[cfg(feature = "82x")]
pub use clocksource_82x::*;
#[cfg(feature = "845")]
pub use clocksource_845::*;

use core::marker::PhantomData;

use crate::syscon;

/// Internal trait used configure clocking of peripheals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClock<PERIPH> {
    /// Selects the clock
    fn select_clock(&self, handle: &mut syscon::Handle);
}

/// Internal trait used for defining valid peripheal clock sources
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSource {
    /// The variant of FCLKSEL.SEL that selects this clock source
    ///
    /// This is not available (or required) on LPC82x.
    #[cfg(feature = "845")]
    const CLOCK: crate::pac::syscon::fclksel::SEL_A;
}

/// Internal trait used for defining the fclksel index for a peripheral
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSelector {
    /// The index of the FCLKSEL register
    ///
    /// This is not relevant on LPC82x.
    const REGISTER_NUM: usize;
}

/// A struct containing the clock configuration for the ADC peripheral
pub struct AdcClock {
    pub(crate) caldiv: u8,
    pub(crate) div: u8,
}

impl AdcClock {
    /// Create the clock config for the ADC peripheral
    ///
    /// The system clock is divided by `caldiv` during calibration or `div`
    /// during normal operation.
    /// During calibration the frequency of the ADC peripheral has to be 500 kHz
    /// and during normal operation it can't be higher than 30 MHz.
    pub unsafe fn new(caldiv: u8, div: u8) -> Self {
        Self { caldiv, div }
    }
    /// Create a new ADC clock config with the maximum sample rate
    ///
    /// Assumes the internal oscillator runs at 12 MHz
    pub fn new_default() -> Self {
        Self { caldiv: 24, div: 0 }
    }
}

/// A struct containing the clock configuration for a peripheral
pub struct I2cClock<PeriphClock> {
    pub(crate) divval: u16,
    pub(crate) mstsclhigh: u8,
    pub(crate) mstscllow: u8,
    _periphclock: PhantomData<PeriphClock>,
}
