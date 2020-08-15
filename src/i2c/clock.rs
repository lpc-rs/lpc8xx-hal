use core::marker::PhantomData;

use crate::syscon::{self, clock_source::PeripheralClockSelector};

/// Contains the clock configuration for an I2C instance
pub struct Clock<Clock> {
    pub(crate) divval: u16,
    pub(crate) mstsclhigh: u8,
    pub(crate) mstscllow: u8,
    pub(crate) _clock: PhantomData<Clock>,
}

impl<C> Clock<C>
where
    C: ClockSource,
{
    /// Create the clock config for the I2C peripheral
    ///
    /// `mstclhigh` and `mstcllow` have to be between 2-9.
    pub fn new(_: &C, divval: u16, mstsclhigh: u8, mstscllow: u8) -> Self {
        assert!(mstsclhigh > 1 && mstsclhigh < 10);
        assert!(mstscllow > 1 && mstscllow < 10);
        Self {
            divval,
            mstsclhigh: mstsclhigh - 2,
            mstscllow: mstscllow - 2,
            _clock: PhantomData,
        }
    }
}

/// Implemented for I2C clock sources
pub trait ClockSource: private::Sealed {
    /// Select the clock source
    ///
    /// This method is used by the I2C API internally. It should not be relevant
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
    use core::marker::PhantomData;

    use crate::syscon;

    use super::{Clock, ClockSource};

    impl super::private::Sealed for () {}

    impl ClockSource for () {
        fn select<S>(_: &S, _: &mut syscon::Handle) {
            // nothing to do; `()` represents the clock that is selected by
            // default
        }
    }

    impl Clock<()> {
        /// Create a new I2C clock configuration for 400 kHz
        ///
        /// Assumes the internal oscillator runs at 12 MHz.
        pub fn new_400khz() -> Self {
            Self {
                divval: 5,
                mstsclhigh: 0,
                mstscllow: 1,
                _clock: PhantomData,
            }
        }
    }
}

#[cfg(feature = "845")]
mod target {
    use core::marker::PhantomData;

    use crate::syscon::{
        self,
        clock_source::{PeripheralClock, PeripheralClockSelector},
        IOSC,
    };

    use super::{Clock, ClockSource};

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

    impl Clock<IOSC> {
        /// Create a new I2C clock configuration for 400 kHz
        ///
        /// Assumes the internal oscillator runs at 12 MHz.
        pub fn new_400khz() -> Self {
            Self {
                divval: 5,
                mstsclhigh: 0,
                mstscllow: 1,
                _clock: PhantomData,
            }
        }
    }
}

mod private {
    pub trait Sealed {}
}
