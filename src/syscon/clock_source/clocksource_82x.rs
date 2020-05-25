use core::marker::PhantomData;

use crate::i2c;

impl i2c::Clock<()> {
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
