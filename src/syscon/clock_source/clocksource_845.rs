use core::marker::PhantomData;

use crate::{
    i2c,
    pac::syscon::fclksel::SEL_A,
    spi,
    syscon::{
        self,
        frg::{FRG, FRG0, FRG1},
        IOSC,
    },
};

use super::{PeripheralClock, PeripheralClockSelector};

macro_rules! peripheral_clocks {
    (
        $(
            $clock:ty,
            $sel:ident;
        )*
     ) => {
        $(
            impl PeripheralClock for $clock {
                const CLOCK: SEL_A = SEL_A::$sel;

                fn select<S>(_: &S, syscon: &mut syscon::Handle)
                where
                    S: PeripheralClockSelector,
                {
                    syscon.fclksel[S::REGISTER_NUM]
                        .write(|w| w.sel().variant(Self::CLOCK));
                }
            }
        )*
    };
}

peripheral_clocks!(
    FRG<FRG0>, FRG0CLK;
    FRG<FRG1>, FRG1CLK;
    IOSC, FRO;
);

impl<CLOCK: i2c::ClockSource> i2c::Clock<CLOCK> {
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

impl<CLOCK: spi::ClockSource> spi::Clock<CLOCK> {
    /// Create the clock config for the spi peripheral
    pub fn new(_: &CLOCK, divval: u16) -> Self {
        Self {
            divval,
            _clock: PhantomData,
        }
    }
}
