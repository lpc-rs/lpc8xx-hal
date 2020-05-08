use core::marker::PhantomData;

use crate::syscon::clock_source::PeripheralClockSource;

/// Defines the clock configuration for a USART instance
pub struct Clock<Clock> {
    pub(crate) psc: u16,
    pub(crate) osrval: u8,
    pub(crate) _clock: PhantomData<Clock>,
}

impl<C> Clock<C>
where
    C: PeripheralClockSource,
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

#[cfg(feature = "82x")]
mod target {
    use crate::{
        syscon::{self, clock_source::PeripheralClock},
        usart::Instance,
    };

    use super::Clock;

    impl<I> PeripheralClock<I> for Clock<syscon::UARTFRG>
    where
        I: Instance,
    {
        fn select_clock(&self, _: &mut syscon::Handle) {
            // NOOP, selected by default
        }
    }
}

#[cfg(feature = "845")]
mod target {
    use core::marker::PhantomData;

    use crate::{
        syscon::{
            self,
            clock_source::{PeripheralClock, PeripheralClockSource},
        },
        usart::Instance,
    };

    use super::Clock;

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

    impl<I, C> PeripheralClock<I> for Clock<C>
    where
        I: Instance,
        C: PeripheralClockSource,
    {
        fn select_clock(&self, syscon: &mut syscon::Handle) {
            syscon.fclksel[I::REGISTER_NUM]
                .write(|w| w.sel().variant(C::CLOCK));
        }
    }
}
