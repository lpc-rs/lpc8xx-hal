use core::fmt;

use embedded_hal::{
    blocking::serial::write::Default as BlockingWriteDefault,
    serial::{Read, Write},
};
use void::Void;

use crate::{
    init_state::{Disabled, Enabled},
    pac::NVIC,
    pins,
    swm::{self, FunctionTrait},
    syscon,
};

use super::{
    clock::{Clock, ClockSource},
    instances::Instance,
    rx::{Error, Rx},
    settings::Settings,
    tx::Tx,
};

/// Interface to a USART peripheral
///
/// Controls the USART.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// You can either use this struct as-is, if you need to send and receive in the
/// same place, or you can move the `rx` and `tx` fields out of this struct, to
/// use the sender and receiver from different contexts.
///
/// Please refer to the [module documentation] for more information.
///
/// # `embedded-hal` traits
/// - [`embedded_hal::serial::Read`] for asynchronous receiving
/// - [`embedded_hal::serial::Write`] for asynchronous sending
/// - [`embedded_hal::blocking::serial::Write`] for synchronous sending
///
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
/// [`embedded_hal::serial::Read`]: #impl-Read%3Cu8%3E
/// [`embedded_hal::serial::Write`]: #impl-Write%3Cu8%3E
/// [`embedded_hal::blocking::serial::Write`]: #impl-Write
pub struct USART<I, State> {
    /// The USART Receiver
    pub rx: Rx<I, State>,

    /// The USART Transmitter
    pub tx: Tx<I, State>,

    usart: I,
}

impl<I> USART<I, Disabled>
where
    I: Instance,
{
    pub(crate) fn new(usart: I) -> Self {
        USART {
            rx: Rx::new(),
            tx: Tx::new(),

            usart,
        }
    }

    /// Enable the USART
    ///
    /// This method is only available, if `USART` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `USART` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// # Limitations
    ///
    /// For USART to function correctly, the UARTFRG reset must be cleared. This
    /// is the default, so unless you have messed with those settings, you
    /// should be good.
    ///
    /// # Examples
    ///
    /// Please refer to the [module documentation] for a full example.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`BaudRate`]: struct.BaudRate.html
    /// [module documentation]: index.html
    pub fn enable<RxPin, TxPin, CLOCK>(
        self,
        clock: &Clock<CLOCK>,
        syscon: &mut syscon::Handle,
        _: swm::Function<I::Rx, swm::state::Assigned<RxPin>>,
        _: swm::Function<I::Tx, swm::state::Assigned<TxPin>>,
        settings: Settings,
    ) -> USART<I, Enabled>
    where
        RxPin: pins::Trait,
        TxPin: pins::Trait,
        I::Rx: FunctionTrait<RxPin>,
        I::Tx: FunctionTrait<TxPin>,
        CLOCK: ClockSource,
    {
        syscon.enable_clock(&self.usart);

        CLOCK::select(&self.usart, syscon);
        self.usart
            .brg
            .write(|w| unsafe { w.brgval().bits(clock.psc) });
        self.usart
            .osr
            .write(|w| unsafe { w.osrval().bits(clock.osrval) });

        // According to the user manual, section 13.6.1, we need to make sure
        // that the USART is not sending or receiving data before writing to
        // CFG, and that it is disabled. We statically know that it is disabled
        // at this point, so there isn't anything to do here to ensure it.

        self.usart.cfg.modify(|_, w| {
            w.enable().enabled();
            w.datalen().bit_8();
            w.ctsen().disabled();
            w.syncen().asynchronous_mode();
            w.loop_().normal();
            w.autoaddr().disabled();
            settings.apply(w);
            w
        });

        self.usart.ctl.modify(|_, w| {
            w.txbrken().normal();
            w.addrdet().disabled();
            w.txdis().enabled();
            w.autobaud().disabled()
        });

        USART {
            rx: Rx::new(), // can't use `self.rx`, due to state
            tx: Tx::new(), // can't use `self.tx`, due to state
            usart: self.usart,
        }
    }
}

impl<I> USART<I, Enabled>
where
    I: Instance,
{
    /// Disable the USART
    ///
    /// This method is only available, if `USART` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `USART` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(self, syscon: &mut syscon::Handle) -> USART<I, Disabled> {
        syscon.disable_clock(&self.usart);

        USART {
            rx: Rx::new(), // can't use `self.rx`, due to state
            tx: Tx::new(), // can't use `self.tx`, due to state
            usart: self.usart,
        }
    }

    /// Enable interrupts for this instance in the NVIC
    ///
    /// This only enables the interrupts in the NVIC. It doesn't enable any
    /// specific interrupt in this USART instance.
    pub fn enable_in_nvic(&mut self) {
        // Safe, because there's no critical section here that this could
        // interfere with.
        unsafe { NVIC::unmask(I::INTERRUPT) };
    }

    /// Disable interrupts for this instance in the NVIC
    ///
    /// This only disables the interrupts in the NVIC. It doesn't change
    /// anything about the interrupt configuration within this USART instance.
    pub fn disable_in_nvic(&mut self) {
        NVIC::mask(I::INTERRUPT);
    }

    /// Clear's this instance's interrupt pending flag in the NVIC
    ///
    /// This only clears the interrupt's pending flag in the NVIC. It does not
    /// affect any of the interrupt-related flags in the peripheral.
    pub fn clear_nvic_pending(&mut self) {
        NVIC::unpend(I::INTERRUPT);
    }

    /// Enable the RXRDY interrupt
    ///
    /// See [`Rx::enable_rxrdy`].
    ///
    /// [`Rx::enable_rxrdy`]: struct.Rx.html#method.enable_rxrdy
    pub fn enable_rxrdy(&mut self) {
        self.rx.enable_rxrdy()
    }

    /// Disable the RXRDY interrupt
    ///
    /// See [`Rx::disable_rxrdy`].
    ///
    /// [`Rx::disable_rxrdy`]: struct.Rx.html#method.disable_rxrdy
    pub fn disable_rxrdy(&mut self) {
        self.rx.disable_rxrdy()
    }

    /// Enable the TXRDY interrupt
    ///
    /// See [`Tx::enable_txrdy`].
    ///
    /// [`Tx::enable_txrdy`]: struct.Tx.html#method.enable_txrdy
    pub fn enable_txrdy(&mut self) {
        self.tx.enable_txrdy()
    }

    /// Disable the TXRDY interrupt
    ///
    /// See [`Tx::disable_txrdy`].
    ///
    /// [`Tx::disable_txrdy`]: struct.Tx.html#method.disable_txrdy
    pub fn disable_txrdy(&mut self) {
        self.tx.disable_txrdy()
    }
}

impl<I, State> USART<I, State>
where
    I: Instance,
{
    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> I {
        self.usart
    }
}

impl<I> Read<u8> for USART<I, Enabled>
where
    I: Instance,
{
    type Error = Error;

    /// Reads a single word from the serial interface
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.read()
    }
}

impl<I> Write<u8> for USART<I, Enabled>
where
    I: Instance,
{
    type Error = Void;

    /// Writes a single word to the serial interface
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(word)
    }

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<I> BlockingWriteDefault<u8> for USART<I, Enabled> where I: Instance {}

impl<I> fmt::Write for USART<I, Enabled>
where
    Self: BlockingWriteDefault<u8>,
    I: Instance,
{
    /// Writes a string slice into this writer, returning whether the write succeeded.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.tx.write_str(s)
    }
}
