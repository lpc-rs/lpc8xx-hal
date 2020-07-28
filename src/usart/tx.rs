use core::{fmt, marker::PhantomData};

use embedded_hal::{
    blocking::serial::write::Default as BlockingWriteDefault, serial::Write,
};
use nb::block;
use void::Void;

use crate::{
    dma::{self, transfer::state::Started},
    init_state::Enabled,
    pac::dma0::channel::xfercfg::DSTINC_A,
};

use super::instances::Instance;

/// USART transmitter
///
/// # `embedded-hal` traits
/// - [`embedded_hal::serial::Write`] for asynchronous sending
/// - [`embedded_hal::blocking::serial::Write`] for synchronous receiving
///
/// [`embedded_hal::serial::Write`]: #impl-Write%3Cu8%3E
/// [`embedded_hal::blocking::serial::Write`]: #impl-Write
pub struct Tx<I, State = Enabled> {
    _instance: PhantomData<I>,
    _state: PhantomData<State>,
}

impl<I, State> Tx<I, State>
where
    I: Instance,
{
    pub(super) fn new() -> Self {
        Self {
            _instance: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<I> Tx<I, Enabled>
where
    I: Instance,
{
    /// Enable the TXRDY interrupt
    ///
    /// The interrupt will not actually work unless the interrupts for this
    /// peripheral have also been enabled in the NVIC. See
    /// [`USART::enable_in_nvic`].
    ///
    /// [`USART::enable_in_nvic`]: struct.USART.html#method.enable_in_nvic
    pub fn enable_txrdy(&mut self) {
        // Sound, as we're only writing atomically to a stateless register.
        let usart = unsafe { &*I::REGISTERS };

        usart.intenset.write(|w| w.txrdyen().set_bit());
    }

    /// Disable the TXRDY interrupt
    pub fn disable_txrdy(&mut self) {
        // Sound, as we're only writing atomically to a stateless register.
        let usart = unsafe { &*I::REGISTERS };

        usart.intenclr.write(|w| w.txrdyclr().set_bit());
    }

    /// Writes the provided buffer using DMA
    ///
    /// # Panics
    ///
    /// Panics, if the length of `buffer` is 0 or larger than 1024.
    pub fn write_all(
        self,
        buffer: &'static [u8],
        channel: dma::Channel<I::TxChannel, Enabled>,
    ) -> dma::Transfer<Started, I::TxChannel, &'static [u8], Self> {
        dma::Transfer::start(channel, buffer, self)
    }
}

impl<I> Write<u8> for Tx<I, Enabled>
where
    I: Instance,
{
    type Error = Void;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        // Sound, as we're only reading from `stat`, and `txdat` is exclusively
        // accessed by this method.
        let usart = unsafe { &*I::REGISTERS };

        if usart.stat.read().txrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        usart.txdat.write(|w|
            // This is sound, as all `u8` values are valid here.
            unsafe { w.txdat().bits(word as u16) });

        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // Sound, as we're only reading from a register.
        let usart = unsafe { &*I::REGISTERS };

        if usart.stat.read().txidle().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }
}

impl<I> BlockingWriteDefault<u8> for Tx<I, Enabled> where I: Instance {}

impl<I> fmt::Write for Tx<I, Enabled>
where
    Self: BlockingWriteDefault<u8>,
    I: Instance,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use crate::prelude::*;

        self.bwrite_all(s.as_bytes()).map_err(|_| fmt::Error)?;
        block!(self.flush()).map_err(|_| fmt::Error)?;

        Ok(())
    }
}

impl<I, State> crate::private::Sealed for Tx<I, State> {}

impl<I> dma::Dest for Tx<I, Enabled>
where
    I: Instance,
{
    type Error = Void;

    fn is_valid(&self) -> bool {
        true
    }

    fn is_full(&self) -> bool {
        false
    }

    fn increment(&self) -> DSTINC_A {
        DSTINC_A::NO_INCREMENT
    }

    fn transfer_count(&self) -> Option<u16> {
        None
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.flush()
    }

    fn end_addr(&mut self) -> *mut u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).txdat }) as *const _ as *mut u8
    }
}
