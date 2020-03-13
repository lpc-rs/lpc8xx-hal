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

/// Defines the clock configuration for a usart
pub struct UsartClock<PeriphClock> {
    pub(crate) psc: u16,
    pub(crate) osrval: u8,
    _periphclock: PhantomData<PeriphClock>,
}

// `impl` blocks are defined in the target-specific sub-modules.
