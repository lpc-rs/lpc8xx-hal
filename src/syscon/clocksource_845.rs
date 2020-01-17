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
    /// Create the clock config for the uart
    ///
    /// Please be aware that the usart additionally divides the
    /// clock input by 16
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
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSelector {
    /// The index
    const REGISTER_NUM: usize;
}

macro_rules! periph_clock_selector {
    ($peripheral:ident, $num:expr) => {
        impl PeripheralClockSelector for pac::$peripheral {
            const REGISTER_NUM: usize = $num;
        }
    };
}
periph_clock_selector!(USART0, 0);
periph_clock_selector!(USART1, 1);
periph_clock_selector!(USART2, 2);
periph_clock_selector!(USART3, 3);
periph_clock_selector!(USART4, 4);
periph_clock_selector!(I2C0, 5);
periph_clock_selector!(I2C1, 6);
periph_clock_selector!(I2C2, 7);
periph_clock_selector!(I2C3, 8);

/// Internal trait used for defining valid peripheal clock sources
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
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

impl<PERIPH: PeripheralClockSelector, CLOCK: PeripheralClockSource>
    PeripheralClock<PERIPH> for PeripheralClockConfig<PERIPH, CLOCK>
{
    fn select_clock(&self, syscon: &mut syscon::Handle) {
        syscon.fclksel[PERIPH::REGISTER_NUM]
            .write(|w| w.sel().variant(CLOCK::CLOCK));
    }
    fn get_psc(&self) -> u16 {
        self.psc
    }
}
