use core::marker::PhantomData;

use crate::syscon::{self, clock_source::PeripheralClockSelector};

/// A struct containing the clock configuration for a peripheral
pub struct Clock<Clock> {
    pub(crate) divval: u16,
    // The fields in the DLY register are ignored, since SSEL & EOF aren't used
    pub(crate) _clock: PhantomData<Clock>,
}

impl<C> Clock<C>
where
    C: ClockSource,
{
    /// Create the clock config for the SPI peripheral
    pub fn new(_: &C, divval: u16) -> Self {
        Self {
            divval,
            _clock: PhantomData,
        }
    }
}

/// Implemented for SPI clock sources
pub trait ClockSource: private::Sealed {
    /// Select the clock source
    ///
    /// This method is used by the SPI API internally. It should not be relevant
    /// to most users.
    ///
    /// The `selector` argument should not be required to implement this trait,
    /// but it makes sure that the caller has access to the peripheral they are
    /// selecting the clock for.
    fn select<S>(selector: &S, handle: &mut syscon::Handle)
    where
        S: PeripheralClockSelector;
}

#[cfg(feature = "82x")]
mod target {
    use crate::syscon;

    use super::ClockSource;

    impl super::private::Sealed for () {}

    impl ClockSource for () {
        fn select<S>(_: &S, _: &mut syscon::Handle) {
            // nothing to do; `()` represents the clock that is selected by
            // default
        }
    }
}

#[cfg(feature = "845")]
mod target {
    use crate::syscon::{
        self,
        clock_source::{PeripheralClock, PeripheralClockSelector},
    };

    use super::ClockSource;

    impl<T> super::private::Sealed for T where T: PeripheralClock {}
    impl<T> ClockSource for T
    where
        T: PeripheralClock,
    {
        fn select<S>(selector: &S, handle: &mut syscon::Handle)
        where
            S: PeripheralClockSelector,
        {
            T::select(selector, handle);
        }
    }
}

mod private {
    pub trait Sealed {}
}
