use core::marker::PhantomData;

use crate::{
    syscon::{self, PeripheralClock, UARTFRG},
    usart,
};

impl<PERIPH: crate::usart::Instance> usart::Clock<PERIPH> {
    /// Create the clock config for the uart
    ///
    /// `osrval` has to be between 5-16
    pub fn new(_: &UARTFRG, psc: u16, osrval: u8) -> Self {
        let osrval = osrval - 1;
        assert!(osrval > 3 && osrval < 0x10);

        Self {
            psc,
            osrval,
            _clock: PhantomData,
        }
    }
}

impl<USART: crate::usart::Instance> PeripheralClock<USART>
    for usart::Clock<USART>
{
    fn select_clock(&self, _: &mut syscon::Handle) {
        // NOOP, selected by default
    }
}

/// A struct containing the clock configuration for a peripheral
pub struct I2cClock<PeriphClock> {
    pub(crate) divval: u16,
    pub(crate) mstsclhigh: u8,
    pub(crate) mstscllow: u8,
    _periphclock: PhantomData<PeriphClock>,
}

impl<PERIPH: crate::i2c::Instance> I2cClock<PERIPH> {
    /// Create the clock config for the i2c peripheral
    ///
    /// mstclhigh & mstcllow have to be between 2-9
    pub fn new(divval: u16, mstsclhigh: u8, mstscllow: u8) -> Self {
        assert!(mstsclhigh > 1 && mstsclhigh < 10);
        assert!(mstscllow > 1 && mstscllow < 10);
        Self {
            divval,
            mstsclhigh: mstsclhigh - 2,
            mstscllow: mstscllow - 2,
            _periphclock: PhantomData,
        }
    }

    /// Create a new i2c clock config for 400 kHz
    ///
    /// Assumes the internal oscillator runs at 12 MHz
    pub fn new_400khz() -> Self {
        Self {
            divval: 5,
            mstsclhigh: 0,
            mstscllow: 1,
            _periphclock: PhantomData,
        }
    }
}

impl<PERIPH: crate::i2c::Instance> PeripheralClock<PERIPH>
    for I2cClock<PERIPH>
{
    fn select_clock(&self, _: &mut syscon::Handle) {
        // NOOP, selected by default
    }
}

/// A struct containing the clock configuration for a peripheral
pub struct SpiClock<PERIPH> {
    pub(crate) divval: u16,
    // The fields in the DLY register are ignored, since SSEL & EOF aren't used
    _periphclock: PhantomData<PERIPH>,
}

impl<PERIPH: crate::spi::Instance> SpiClock<PERIPH> {
    /// Create the clock config for the spi peripheral
    pub fn new(divval: u16) -> Self {
        Self {
            divval,
            _periphclock: PhantomData,
        }
    }
}

impl<PERIPH: crate::spi::Instance> PeripheralClock<PERIPH>
    for SpiClock<PERIPH>
{
    fn select_clock(&self, _: &mut syscon::Handle) {
        // NOOP, selected by default
    }
}
