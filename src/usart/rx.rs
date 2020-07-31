use core::marker::PhantomData;

use void::Void;

use crate::{
    dma::{self, transfer::state::Ready},
    embedded_hal::serial::Read,
    init_state,
    pac::dma0::channel::xfercfg::SRCINC_A,
};

use super::{
    flags::Flag,
    instances::Instance,
    state::{Enabled, Word},
};

/// USART receiver
///
/// # `embedded-hal` traits
/// - [`embedded_hal::serial::Read`] for asynchronous receiving
///
///
/// [`embedded_hal::serial::Read`]: #impl-Read%3Cu8%3E
pub struct Rx<I, State> {
    _instance: PhantomData<I>,
    _state: PhantomData<State>,
}

impl<I, State> Rx<I, State>
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

impl<I, W> Rx<I, Enabled<W>>
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

    /// Enable the RXRDY interrupt
    ///
    /// The interrupt will not actually work unless the interrupts for this
    /// peripheral have also been enabled in the NVIC. See
    /// [`USART::enable_in_nvic`].
    ///
    /// [`USART::enable_in_nvic`]: struct.USART.html#method.enable_in_nvic
    pub fn enable_rxrdy(&mut self) {
        // Sound, as we're only writing atomically to a stateless register.
        let usart = unsafe { &*I::REGISTERS };

        usart.intenset.write(|w| w.rxrdyen().set_bit());
    }

    /// Disable the RXRDY interrupt
    pub fn disable_rxrdy(&mut self) {
        // Sound, as we're only writing atomically to a stateless register.
        let usart = unsafe { &*I::REGISTERS };

        usart.intenclr.write(|w| w.rxrdyclr().set_bit());
    }
}

impl<I> Rx<I, Enabled<u8>>
where
    I: Instance,
{
    /// Reads until the provided buffer is full, using DMA
    ///
    /// # Panics
    ///
    /// Panics, if the length of `buffer` is 0 or larger than 1024.
    pub fn read_all(
        self,
        buffer: &'static mut [u8],
        channel: dma::Channel<I::RxChannel, init_state::Enabled>,
    ) -> dma::Transfer<Ready, I::RxChannel, Self, &'static mut [u8]> {
        dma::Transfer::new(channel, self, buffer)
    }
}

impl<I, W> Read<W> for Rx<I, Enabled<W>>
where
    I: Instance,
    W: Word,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<W, Self::Error> {
        // Sound, as we're only reading from `stat`, and `rxdatastat` is
        // exclusively accessed by this method.
        let usart = unsafe { &*I::REGISTERS };

        let stat = usart.stat.read();

        if stat.rxbrk().bit_is_set() {
            return Err(nb::Error::WouldBlock);
        }

        if stat.rxrdy().bit_is_set() {
            // It's important to read this register all at once, as reading
            // it changes the status flags.
            let rx_dat_stat = usart.rxdatstat.read();

            if stat.overrunint().bit_is_set() {
                Err(nb::Error::Other(Error::Overrun))
            } else if rx_dat_stat.framerr().bit_is_set() {
                Err(nb::Error::Other(Error::Framing))
            } else if rx_dat_stat.parityerr().bit_is_set() {
                Err(nb::Error::Other(Error::Parity))
            } else if rx_dat_stat.rxnoise().bit_is_set() {
                Err(nb::Error::Other(Error::Noise))
            } else {
                Ok(Word::from_u16(rx_dat_stat.rxdat().bits()))
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<I, State> crate::private::Sealed for Rx<I, State> {}

impl<I> dma::Source for Rx<I, Enabled<u8>>
where
    I: Instance,
{
    type Error = Void;

    fn is_valid(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn increment(&self) -> SRCINC_A {
        SRCINC_A::NO_INCREMENT
    }

    fn transfer_count(&self) -> Option<u16> {
        None
    }

    fn end_addr(&self) -> *const u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).rxdat }) as *const _ as *mut u8
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

/// A USART error
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Character received with a stop bit missing at the expected location
    Framing,

    /// Corrupted character received
    Noise,

    /// Character received, while receive buffer was still in use
    Overrun,

    /// Parity error detected in received character
    Parity,
}
