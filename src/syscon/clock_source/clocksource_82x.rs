use core::marker::PhantomData;

use crate::{
    i2c,
    syscon::{self, UARTFRG},
};

use super::{PeripheralClock, PeripheralClockSource, SpiClock};

impl PeripheralClockSource for UARTFRG {}

impl i2c::Clock<()> {
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
            _clock: PhantomData,
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
            _clock: PhantomData,
        }
    }
}

impl<PERIPH: crate::i2c::Instance> PeripheralClock<PERIPH> for i2c::Clock<()> {
    fn select_clock(&self, _: &mut syscon::Handle) {
        // NOOP, selected by default
    }
}

impl<PERIPH: crate::spi::Instance> SpiClock<PERIPH> {
    /// Create the clock config for the spi peripheral
    pub fn new(divval: u16) -> Self {
        Self {
            divval,
            _clock: PhantomData,
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
