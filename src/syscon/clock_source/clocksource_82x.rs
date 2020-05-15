use core::marker::PhantomData;

use crate::{i2c, spi};

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

impl spi::Clock<()> {
    /// Create the clock config for the spi peripheral
    pub fn new(divval: u16) -> Self {
        Self {
            divval,
            _clock: PhantomData,
        }
    }
}
