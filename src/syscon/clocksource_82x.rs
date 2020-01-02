use crate::syscon::{self, PeripheralClock, UARTFRG};
use core::marker::PhantomData;

/// A struct containing the clock configuration for a peripheral
pub struct PeripheralClockConfig<PERIPH> {
    // UART, SPI & I2C peripherals all have an internal 16 bit clock dividerr
    psc: u16,
    _periph: PhantomData<PERIPH>,
}

impl<USART: crate::usart::Peripheral> PeripheralClockConfig<USART> {
    /// TODO
    pub fn new(_: &UARTFRG, psc: u16) -> Self {
        Self {
            psc,
            _periph: PhantomData,
        }
    }
}

impl<USART: crate::usart::Peripheral> PeripheralClock<USART> for PeripheralClockConfig<USART> {
    fn select_clock(&self, _: &mut syscon::Handle) {
        // NOOP, selected by default
    }

    fn get_psc(&self) -> u16 {
        self.psc
    }
}
