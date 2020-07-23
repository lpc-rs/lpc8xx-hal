use core::marker::PhantomData;

use crate::{embedded_hal::serial::Read, init_state::Enabled};

use super::instances::Instance;

/// USART receiver
///
/// # `embedded-hal` traits
/// - [`embedded_hal::serial::Read`] for asynchronous receiving
///
///
/// [`embedded_hal::serial::Read`]: #impl-Read%3Cu8%3E
pub struct Rx<I, State = Enabled> {
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

impl<I> Rx<I, Enabled>
where
    I: Instance,
{
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

impl<I> Read<u8> for Rx<I, Enabled>
where
    I: Instance,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
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
                // `bits` returns `u16`, but at most 9 bits are used. We've
                // configured UART to use only 8 bits, so we can safely cast to
                // `u8`.
                Ok(rx_dat_stat.rxdat().bits() as u8)
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
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
