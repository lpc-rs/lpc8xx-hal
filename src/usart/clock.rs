use core::marker::PhantomData;

use crate::syscon::{self, clock_source::PeripheralClockSelector};

/// Defines the clock configuration for a USART instance
///
/// This struct has two type arguments:
/// - `Clock` specifies the clock used to power the USART clock. This clock will
///   be selected when the USART instance is enabled.
/// - `Mode` specifies the USART mode. A distinction between synchronous and
///   asynchronous mode has to be made, as OSRVAL has no meaning in synchronous
///   mode.
pub struct Clock<Clock, Mode> {
    pub(super) psc: u16,
    pub(super) osrval: u8,
    pub(super) _clock: PhantomData<Clock>,
    pub(super) _mode: PhantomData<Mode>,
}

impl<C, Mode> Clock<C, Mode>
where
    C: ClockSource,
{
    /// Create the clock configuration for the USART
    ///
    /// The `osrval` argument has to be between 5-16. It will be ignored in
    /// synchronous mode.
    pub fn new(_: &C, psc: u16, osrval: u8) -> Self {
        let osrval = osrval - 1;
        assert!(osrval > 3 && osrval < 0x10);

        Self {
            psc,
            osrval,
            _clock: PhantomData,
            _mode: PhantomData,
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

    use crate::{
        syscon::{
            self,
            clock_source::{PeripheralClock, PeripheralClockSelector},
        },
        usart::state::AsyncMode,
    };

    use super::{Clock, ClockSource};

    impl Clock<syscon::IOSC, AsyncMode> {
        /// Create a new configuration with a specified baudrate
        ///
        /// Assumes the internal oscillator runs at 12 MHz.
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
                _mode: PhantomData,
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
