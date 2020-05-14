use core::marker::PhantomData;

use crate::syscon::clock_source::PeripheralClockSource;

/// A struct containing the clock configuration for a peripheral
pub struct Clock<Clock> {
    pub(crate) divval: u16,
    // The fields in the DLY register are ignored, since SSEL & EOF aren't used
    pub(crate) _clock: PhantomData<Clock>,
}

/// Implemented for SPI clock sources
pub trait ClockSource: PeripheralClockSource + private::Sealed {}

#[cfg(feature = "845")]
mod target {
    use crate::syscon::clock_source::PeripheralClockSource;

    use super::ClockSource;

    impl<T> super::private::Sealed for T where T: PeripheralClockSource {}
    impl<T> ClockSource for T where T: PeripheralClockSource {}
}

mod private {
    pub trait Sealed {}
}
