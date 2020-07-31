use core::{fmt, marker::PhantomData};

use embedded_hal::{
    blocking::serial::write::Default as BlockingWriteDefault, serial::Write,
};
use nb::block;
use void::Void;

use crate::{
    dma::{self, transfer::state::Ready},
    init_state,
    pac::dma0::channel::xfercfg::DSTINC_A,
};

use super::{
    flags::{Flag, Interrupts},
    instances::Instance,
    state::{Enabled, Word},
};

/// USART transmitter
///
/// # `embedded-hal` traits
/// - [`embedded_hal::serial::Write`] for asynchronous sending
/// - [`embedded_hal::blocking::serial::Write`] for synchronous receiving
///
/// [`embedded_hal::serial::Write`]: #impl-Write%3Cu8%3E
/// [`embedded_hal::blocking::serial::Write`]: #impl-Write
pub struct Tx<I, State> {
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

impl<I, W> Tx<I, Enabled<W>>
where
    I: Instance,
    W: Word,
{
    /// Query whether the provided flag is set
    ///
    /// Flags that need to be reset by software will be reset by this operation.
    pub fn is_flag_set(&self, flag: Flag) -> bool {
        flag.is_set::<I>()
    }

    /// Enable interrupts
    ///
    /// Enables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    pub fn enable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.enable::<I>();
    }

    /// Disable interrupts
    ///
    /// Disables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    pub fn disable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.disable::<I>();
    }
}

impl<I> Tx<I, Enabled<u8>>
where
    I: Instance,
{
    /// Writes the provided buffer using DMA
    ///
    /// # Panics
    ///
    /// Panics, if the length of `buffer` is 0 or larger than 1024.
    pub fn write_all(
        self,
        buffer: &'static [u8],
        channel: dma::Channel<I::TxChannel, init_state::Enabled>,
    ) -> dma::Transfer<Ready, I::TxChannel, &'static [u8], Self> {
        dma::Transfer::new(channel, buffer, self)
    }
}

impl<I, W> Write<W> for Tx<I, Enabled<W>>
where
    I: Instance,
    W: Word,
{
    type Error = Void;

    fn write(&mut self, word: W) -> nb::Result<(), Self::Error> {
        // Sound, as we're only reading from `stat`, and `txdat` is exclusively
        // accessed by this method.
        let usart = unsafe { &*I::REGISTERS };

        if usart.stat.read().txrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        usart.txdat.write(|w|
            // This is sound, as all `u8` values are valid here.
            unsafe { w.txdat().bits(word.into()) });

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

impl<I, W> BlockingWriteDefault<W> for Tx<I, Enabled<W>>
where
    I: Instance,
    W: Word,
{
}

impl<I> fmt::Write for Tx<I, Enabled<u8>>
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

impl<I> dma::Dest for Tx<I, Enabled<u8>>
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

    fn end_addr(&mut self) -> *mut u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).txdat }) as *const _ as *mut u8
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        self.flush()
    }
}
