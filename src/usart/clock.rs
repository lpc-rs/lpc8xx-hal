use core::marker::PhantomData;

use crate::syscon::{self, clock_source::PeripheralClockSelector};

/// Defines the clock configuration for a USART instance
pub struct Clock<Clock> {
    pub(crate) psc: u16,
    pub(crate) osrval: u8,
    pub(crate) _clock: PhantomData<Clock>,
}

impl<C> Clock<C>
where
    C: ClockSource,
{
    /// Create the clock config for the uart
    ///
    /// `osrval` has to be between 5-16
    pub fn new(_: &C, psc: u16, osrval: u8) -> Self {
        let osrval = osrval - 1;
        assert!(osrval > 3 && osrval < 0x10);

        Self {
            psc,
            osrval,
            _clock: PhantomData,
        }
    }
}

/// Implemented for USART clock sources
pub trait ClockSource: private::Sealed {
    /// Select the clock source
    ///
    /// This method is used by the USART API internally. It should not be
    /// relevant to most users.
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
    use crate::syscon::{self, UARTFRG};

    use super::ClockSource;

    impl super::private::Sealed for UARTFRG {}

    impl ClockSource for UARTFRG {
        fn select<S>(_: &S, _: &mut syscon::Handle) {
            // nothing to do; selected by default
        }
    }
}

#[cfg(feature = "845")]
mod target {
    use core::marker::PhantomData;

    use crate::syscon::{
        self,
        clock_source::{PeripheralClock, PeripheralClockSelector},
    };

    use super::{Clock, ClockSource};

    impl Clock<syscon::IOSC> {
        /// Create a new configuration with a specified baudrate
        ///
        /// Assumes the internal oscillator runs at 12 MHz
        pub fn new_with_baudrate(baudrate: u32) -> Self {
            // We want something with 5% tolerance
            let calc = baudrate * 20;
            let mut osrval = 5;
            for i in (5..=16).rev() {
                if calc * (i as u32) < 12_000_000 {
                    osrval = i;
                }
            }
            let psc = (12_000_000 / (baudrate * osrval as u32) - 1) as u16;
            let osrval = osrval - 1;
            Self {
                psc,
                osrval,
                _clock: PhantomData,
            }
        }
    }

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
