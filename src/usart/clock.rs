use core::marker::PhantomData;

use crate::syscon::{self, clock_source::PeripheralClockSelector};

/// Defines the clock configuration for a USART instance
///
/// This struct has two type arguments:
/// - `T` specifies the clock used to power the USART clock. This clock will be
///   selected when the USART instance is enabled.
/// - `Mode` specifies the USART mode. A distinction between synchronous and
///   asynchronous mode has to be made, as OSRVAL has no meaning in synchronous
///   mode.
#[derive(Debug)]
pub struct Clock<T, Mode> {
    pub(super) brgval: u16,
    pub(super) osrval: u8,
    pub(super) _clock: PhantomData<T>,
    pub(super) _mode: PhantomData<Mode>,
}

impl<T, Mode> Clock<T, Mode>
where
    T: ClockSource,
{
    /// Create the clock configuration for the USART
    ///
    /// The `osrval` argument has to be between 5-16. It will be ignored in
    /// synchronous mode.
    pub fn new(_: &T, brgval: u16, osrval: u8) -> Self {
        let osrval = osrval - 1;
        assert!(osrval > 3 && osrval < 0x10);

        Self {
            brgval,
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
        /// Searches for configuration values that lead to a baud rate that is
        /// within 5% accuracy of the desired baudrate. Panics, if it can't find
        /// such parameters.
        ///
        /// Chooses the highest possibly oversampling value that will still give
        /// the desired accuracy. Please note that if the oversampling value
        /// gets too low, this can result in framing and noise errors when
        /// receiving data.
        ///
        /// Due to the aforementioned limitations, and because this methods is
        /// relatively computationally expensive, it is recommended to only use
        /// it during initialization, with known baud rates. If you need more
        /// control, please use [`Clock::new`] in combination with an FRG.
        ///
        /// Assumes the internal oscillator runs at 12 MHz.
        pub fn new_with_baudrate(baudrate: u32) -> Self {
            fn calculate_brgval(
                desired_baudrate: u32,
                osrval: u8,
            ) -> (u16, u8) {
                let iosc_frequency = 12_000_000;

                let brgval = iosc_frequency
                    / (desired_baudrate * (osrval + 1) as u32)
                    - 1;
                let resulting_baudrate =
                    iosc_frequency / (brgval + 1) / (osrval as u32 + 1);

                // This subtraction should never overflow. Due to rounding, the
                // resulting baud rate is always going to be higher than the
                // desired one.
                let deviation_percent = (resulting_baudrate - desired_baudrate)
                    * 100
                    / desired_baudrate;

                (brgval as u16, deviation_percent as u8)
            }
            fn search_parameters(baudrate: u32) -> (u16, u8) {
                // Look for the highest `osrval` that will give us an accuracy
                // within 5%.
                for osrval in (0x4..=0xf).rev() {
                    let (brgval, deviation_percent) =
                        calculate_brgval(baudrate, osrval);
                    if deviation_percent < 5 {
                        return (brgval, osrval);
                    }
                }

                panic!("Could not find parameters that are accurate within 5%");
            }

            let (brgval, osrval) = search_parameters(baudrate);

            Self {
                brgval,
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
