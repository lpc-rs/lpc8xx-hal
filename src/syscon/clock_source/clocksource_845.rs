use core::marker::PhantomData;

use crate::{
    i2c,
    pac::{self, syscon::fclksel::SEL_A},
    spi,
    syscon::{self, frg, IOSC},
};

use super::{PeripheralClock, PeripheralClockSelector, PeripheralClockSource};

macro_rules! periph_clock_selector {
    ($peripheral:ident, $num:expr) => {
        impl PeripheralClockSelector for pac::$peripheral {
            const REGISTER_NUM: usize = $num;
        }
    };
}

periph_clock_selector!(SPI0, 9);
periph_clock_selector!(SPI1, 10);

impl PeripheralClockSource for frg::FRG<frg::FRG0> {
    const CLOCK: SEL_A = SEL_A::FRG0CLK;
}

impl PeripheralClockSource for frg::FRG<frg::FRG1> {
    const CLOCK: SEL_A = SEL_A::FRG1CLK;
}

impl PeripheralClockSource for IOSC {
    const CLOCK: SEL_A = SEL_A::FRO;
}

impl<CLOCK: PeripheralClockSource> i2c::Clock<CLOCK> {
    /// Create the clock config for the i2c peripheral
    ///
    /// mstclhigh & mstcllow have to be between 2-9
    pub fn new(_: &CLOCK, divval: u16, mstsclhigh: u8, mstscllow: u8) -> Self {
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

impl i2c::Clock<IOSC> {
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

impl<PERIPH: i2c::Instance, CLOCK: PeripheralClockSource>
    PeripheralClock<PERIPH> for i2c::Clock<CLOCK>
{
    fn select_clock(&self, syscon: &mut syscon::Handle) {
        syscon.fclksel[PERIPH::REGISTER_NUM]
            .write(|w| w.sel().variant(CLOCK::CLOCK));
    }
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    spi::Clock<(PERIPH, CLOCK)>
{
    /// Create the clock config for the spi peripheral
    pub fn new(_: &CLOCK, divval: u16) -> Self {
        Self {
            divval,
            _clock: PhantomData,
        }
    }
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    PeripheralClock<PERIPH> for spi::Clock<(PERIPH, CLOCK)>
{
    fn select_clock(&self, syscon: &mut syscon::Handle) {
        syscon.fclksel[PERIPH::REGISTER_NUM]
            .write(|w| w.sel().variant(CLOCK::CLOCK));
    }
}
