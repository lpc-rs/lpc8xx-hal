use crate::pac;
use crate::{
    pac::syscon::fclksel::SEL_A,
    syscon::{self, frg, PeripheralClock, IOSC},
};
use core::marker::PhantomData;

/// A struct containing the clock configuration for a peripheral
pub struct PeripheralClockConfig<PERIPH, CLOCK> {
    // UART, SPI & I2C peripherals all have an internal 16 bit clock dividerr
    psc: u16,
    _periphclock: PhantomData<(PERIPH, CLOCK)>,
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    PeripheralClockConfig<PERIPH, CLOCK>
{
    /// TODO
    pub fn new(_: &CLOCK, psc: u16) -> Self {
        Self {
            psc,
            _periphclock: PhantomData,
        }
    }
}

/// Internal trait used for defining the fclksel index for a peripheral
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSelector {
    /// The index
    const REGISTER_NUM: usize;
}

impl PeripheralClockSelector for pac::USART0 {
    const REGISTER_NUM: usize = 0;
}

impl PeripheralClockSelector for pac::USART1 {
    const REGISTER_NUM: usize = 1;
}

impl PeripheralClockSelector for pac::USART2 {
    const REGISTER_NUM: usize = 2;
}

impl PeripheralClockSelector for pac::USART3 {
    const REGISTER_NUM: usize = 3;
}

impl PeripheralClockSelector for pac::USART4 {
    const REGISTER_NUM: usize = 4;
}

/// Internal trait used for defining valid peripheal clock sources
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSource {
    /// The variant
    const CLOCK: SEL_A;
}

impl PeripheralClockSource for frg::FRG<frg::FRG0> {
    const CLOCK: SEL_A = SEL_A::FRG0CLK;
}

impl PeripheralClockSource for frg::FRG<frg::FRG1> {
    const CLOCK: SEL_A = SEL_A::FRG1CLK;
}

impl PeripheralClockSource for IOSC {
    const CLOCK: SEL_A = SEL_A::FRO;
}

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource> PeripheralClock<PERIPH>
    for PeripheralClockConfig<PERIPH, CLOCK>
{
    fn select_clock(&self, syscon: &mut syscon::Handle) {
        syscon.fclksel[PERIPH::REGISTER_NUM].write(|w| w.sel().variant(CLOCK::CLOCK));
    }
    fn get_psc(&self) -> u16 {
        self.psc
    }
}
