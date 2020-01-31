//! API for USART
//!
//! The entry point to this API is [`USART`]. Currently, only some limited UART
//! functionality is implemented.
//!
//! The USART peripheral is described in the user manual, chapter 13.
//!
//! # Examples
//!
//! ``` no_run
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::Peripherals;
//! use lpc82x_hal::usart::{
//!     BaudRate,
//!     USART,
//! };
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut syscon = p.SYSCON.split();
//! let mut swm    = p.SWM.split();
//!
//! // Set baud rate to 115200 baud
//! // Please refer to the USART example in the repository for a full
//! // explanation of this value.
//! syscon.uartfrg.set_clkdiv(6);
//! syscon.uartfrg.set_frgmult(22);
//! syscon.uartfrg.set_frgdiv(0xff);
//! let baud_rate = BaudRate::new(&syscon.uartfrg, 0);
//!
//! let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
//!     swm.pins.pio0_0.into_swm_pin(),
//!     &mut swm.handle,
//! );
//! let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
//!     swm.pins.pio0_4.into_swm_pin(),
//!     &mut swm.handle,
//! );
//!
//! // Initialize USART0. This should never fail, as the only reason `init`
//! // returns a `Result::Err` is when the transmitter is busy, which it
//! // shouldn't be right now.
//! let mut serial = p.USART0.enable(
//!     &baud_rate,
//!     &mut syscon.handle,
//!     u0_rxd,
//!     u0_txd,
//! );
//!
//! // Use a blocking method to write a string
//! serial.tx().bwrite_all(b"Hello, world!");
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/examples

use core::fmt;
use core::ops::Deref;

use embedded_hal::blocking::serial::write::Default as BlockingWriteDefault;
use embedded_hal::serial::{Read, Write};
use nb::{self, block};
use void::Void;

use crate::{
    dma, init_state,
    pac::{self, usart0::TXDAT, Interrupt, NVIC},
    swm::{self, FunctionTrait, PinTrait},
    syscon::{self, clocksource::UsartClock, PeripheralClock},
};

/// Interface to a USART peripheral
///
/// Controls the USART.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct USART<I, State = init_state::Enabled> {
    usart: I,
    _state: State,
}

impl<I> USART<I, init_state::Disabled>
where
    I: Instance,
{
    pub(crate) fn new(usart: I) -> Self {
        USART {
            usart,
            _state: init_state::Disabled,
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
        clock: &UsartClock<CLOCK>,
        syscon: &mut syscon::Handle,
        _: swm::Function<I::Rx, swm::state::Assigned<RxPin>>,
        _: swm::Function<I::Tx, swm::state::Assigned<TxPin>>,
    ) -> USART<I, init_state::Enabled>
    where
        RxPin: PinTrait,
        TxPin: PinTrait,
        I::Rx: FunctionTrait<RxPin>,
        I::Tx: FunctionTrait<TxPin>,
        UsartClock<CLOCK>: PeripheralClock<I>,
    {
        syscon.enable_clock(&self.usart);

        clock.select_clock(syscon);
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
            w.paritysel().no_parity();
            w.stoplen().bit_1();
            w.ctsen().disabled();
            w.syncen().asynchronous_mode();
            w.loop_().normal();
            w.autoaddr().disabled();
            w.rxpol().standard();
            w.txpol().standard()
        });

        self.usart.ctl.modify(|_, w| {
            w.txbrken().normal();
            w.addrdet().disabled();
            w.txdis().enabled();
            w.autobaud().disabled()
        });

        USART {
            usart: self.usart,
            _state: init_state::Enabled(()),
        }
    }
}

impl<I> USART<I, init_state::Enabled>
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
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> USART<I, init_state::Disabled> {
        syscon.disable_clock(&self.usart);

        USART {
            usart: self.usart,
            _state: init_state::Disabled,
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

    /// Enable the RXRDY interrupt
    ///
    /// See [`Rx::enable_rxrdy`].
    ///
    /// [`Rx::enable_rxrdy`]: struct.Rx.html#method.enable_rxrdy
    pub fn enable_rxrdy(&mut self) {
        self.rx().enable_rxrdy()
    }

    /// Disable the RXRDY interrupt
    ///
    /// See [`Rx::disable_rxrdy`].
    ///
    /// [`Rx::disable_rxrdy`]: struct.Rx.html#method.disable_rxrdy
    pub fn disable_rxrdy(&mut self) {
        self.rx().disable_rxrdy()
    }

    /// Enable the TXRDY interrupt
    ///
    /// See [`Tx::enable_txrdy`].
    ///
    /// [`Tx::enable_txrdy`]: struct.Tx.html#method.enable_txrdy
    pub fn enable_txrdy(&mut self) {
        self.tx().enable_txrdy()
    }

    /// Disable the TXRDY interrupt
    ///
    /// See [`Tx::disable_txrdy`].
    ///
    /// [`Tx::disable_txrdy`]: struct.Tx.html#method.disable_txrdy
    pub fn disable_txrdy(&mut self) {
        self.tx().disable_txrdy()
    }

    /// Return USART receiver
    pub fn rx(&self) -> Rx<I> {
        Rx(self)
    }

    /// Return USART transmitter
    pub fn tx(&self) -> Tx<I> {
        Tx(self)
    }
}

impl<I, State> USART<I, State> {
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

/// USART receiver
pub struct Rx<'usart, I: 'usart>(&'usart USART<I>);

impl<'usart, I> Rx<'usart, I>
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
        self.0.usart.intenset.write(|w| w.rxrdyen().set_bit());
    }

    /// Disable the RXRDY interrupt
    pub fn disable_rxrdy(&mut self) {
        self.0.usart.intenclr.write(|w| w.rxrdyclr().set_bit());
    }
}

impl<'usart, I> Read<u8> for Rx<'usart, I>
where
    I: Instance,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let stat = self.0.usart.stat.read();

        if stat.rxbrk().bit_is_set() {
            return Err(nb::Error::WouldBlock);
        }

        if stat.rxrdy().bit_is_set() {
            // It's important to read this register all at once, as reading
            // it changes the status flags.
            let rx_dat_stat = self.0.usart.rxdatstat.read();

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

/// USART transmitter
pub struct Tx<'usart, I: 'usart>(&'usart USART<I>);

impl<'usart, I> Tx<'usart, I>
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
        self.0.usart.intenset.write(|w| w.txrdyen().set_bit());
    }

    /// Disable the TXRDY interrupt
    pub fn disable_txrdy(&mut self) {
        self.0.usart.intenclr.write(|w| w.txrdyclr().set_bit());
    }
}

impl<'usart, I> Write<u8> for Tx<'usart, I>
where
    I: Instance,
{
    type Error = Void;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        if self.0.usart.stat.read().txrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        unsafe {
            self.0.usart.txdat.write(|w| w.txdat().bits(word as u16));
        }

        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.0.usart.stat.read().txidle().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }
}

impl<'usart, I> BlockingWriteDefault<u8> for Tx<'usart, I> where I: Instance {}

impl<'usart, I> fmt::Write for Tx<'usart, I>
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

impl<'usart, I> dma::Dest for Tx<'usart, I>
where
    I: Instance,
{
    type Error = Void;

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.flush()
    }

    fn end_addr(&mut self) -> *mut u8 {
        &self.0.usart.txdat as *const _ as *mut TXDAT as *mut u8
    }
}

/// Internal trait for USART peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Instance:
    Deref<Target = pac::usart0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
{
    /// The interrupt that is triggered for this USART peripheral
    const INTERRUPT: Interrupt;

    /// The movable function that needs to be assigned to this USART's RX pin
    type Rx;

    /// The movable function that needs to be assigned to this USART's TX pin
    type Tx;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $interrupt:ident,
            $rx:ident,
            $tx:ident;
        )*
    ) => {
        $(
            impl Instance for pac::$instance {
                const INTERRUPT: Interrupt = Interrupt::$interrupt;

                type Rx = swm::$rx;
                type Tx = swm::$tx;
            }
        )*
    };
}

instances!(
    USART0, USART0, U0_RXD, U0_TXD;
    USART1, USART1, U1_RXD, U1_TXD;
    USART2, USART2, U2_RXD, U2_TXD;
);

#[cfg(feature = "845")]
instances!(
    USART3, PIN_INT6_USART3, U3_RXD, U3_TXD;
    USART4, PIN_INT7_USART4, U4_RXD, U4_TXD;
);

/// A USART error
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Character received with a stop bit missing at the expected location
    Framing,

    /// Corrupted character received
    Noise,

    /// Character received, while receiver buffer was still in use
    Overrun,

    /// Parity error detected in received character
    Parity,
}
