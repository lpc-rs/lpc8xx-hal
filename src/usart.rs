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
//! let mut serial = p.USART0
//!     .enable(
//!         &baud_rate,
//!         &mut syscon.handle,
//!         u0_rxd,
//!         u0_txd,
//!     )
//!     .expect("UART initialization shouldn't fail");
//!
//! // Use a blocking method to write a string
//! serial.bwrite_all(b"Hello, world!");
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [examples in the repository]: https://github.com/braun-robotics/rust-lpc82x-hal/tree/master/examples


use core::fmt;
use core::ops::Deref;

use embedded_hal::blocking::serial::write::Default as BlockingWriteDefault;
use embedded_hal::serial::{
    Read,
    Write,
};
use nb;

use init_state::{
    self,
    InitState,
};
use raw::{
    self,
    Interrupt,
    NVIC,
};
use swm::{
    self,
    FunctionTrait,
    PinTrait,
};
use syscon::{
    self,
    UARTFRG,
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
pub struct USART<UsartX, State: InitState = init_state::Enabled> {
    usart : UsartX,
    _state: State,
}

impl<UsartX> USART<UsartX, init_state::Disabled> where UsartX: Peripheral {
    pub(crate) fn new(usart: UsartX) -> Self {
        USART {
            usart : usart,
            _state: init_state::Disabled,
        }
    }

    /// Enable the USART
    ///
    /// Enables the clock and clears the peripheral reset for the USART
    /// peripheral.
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
    pub fn enable<Rx, Tx>(mut self,
        baud_rate: &BaudRate,
        syscon   : &mut syscon::Handle,
        _        : swm::Function<UsartX::Rx, swm::state::Assigned<Rx>>,
        _        : swm::Function<UsartX::Tx, swm::state::Assigned<Tx>>,
    )
        -> nb::Result<USART<UsartX, init_state::Enabled>, !>
        where
            Rx        : PinTrait,
            Tx        : PinTrait,
            UsartX::Rx: FunctionTrait<Rx>,
            UsartX::Tx: FunctionTrait<Tx>,
    {
        syscon.enable_clock(&mut self.usart);
        syscon.clear_reset(&mut self.usart);

        self.usart.brg.write(|w| unsafe { w.brgval().bits(baud_rate.brgval) });

        // Disable USART peripheral before writing configuration. This is
        // required, according to the user manual, section 13.6.1.
        //
        // Also according to that section in the user manual, we should make
        // sure that the USART is neither sending nor receiving. Presumably we
        // have full control over the USART at this point, so the sending bit is
        // fine. How it could be possible to make sure we're not receiving
        // without introducing a race condition, I don't know.
        //
        // Even if the race condition weren't an issue, we can't just wait for
        // the receiver to become idle. If another piece of hardware is
        // continuously sending (maybe due to some malfunction), that would mean
        // this function could block forever. This is not an acceptable risk to
        // take.
        //
        // I have no idea what would happen, if we tried to disable the USART
        // while it is receiving. If you, the reader, ever find out, well, this
        // is the piece of code that may need fixing.
        if self.usart.stat.read().txidle().bit_is_clear() {
            // Since we should have full control over the USART at this point,
            // this presumably shouldn't take long and just waiting in a loop
            // should be okay. However, it's possible that some other piece of
            // code is working with the UART behind our back.
            //
            // If that were the case, we would want an explicit error, not a
            // random freeze.
            return Err(nb::Error::WouldBlock);
        }
        self.usart.cfg.write(|w| w.enable().disabled());

        self.usart.cfg.modify(|_, w|
            w
                .enable().enabled()
                .datalen()._8_bit_data_length()
                .paritysel().no_parity()
                .stoplen()._1_stop_bit()
                .ctsen().no_flow_control()
                .syncen().asynchronous_mode_is()
                .loop_().normal_operation()
                .autoaddr().disabled()
                .rxpol().not_changed()
                .txpol().not_changed()
        );

        self.usart.ctl.modify(|_, w|
            w
                .txbrken().normal_operation()
                .addrdet().disabled()
                .txdis().not_disabled()
                .autobaud().disabled()
        );

        Ok(USART {
            usart : self.usart,
            _state: init_state::Enabled,
        })
    }
}

impl<UsartX> USART<UsartX, init_state::Enabled> where UsartX: Peripheral {
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
    pub fn disable(mut self, syscon: &mut syscon::Handle)
        -> USART<UsartX, init_state::Disabled>
    {
        syscon.disable_clock(&mut self.usart);

        USART {
            usart : self.usart,
            _state: init_state::Disabled,
        }
    }

    /// Enable the USART interrupts
    ///
    /// Enable the interrupts for this USART peripheral. This only enables the
    /// interrupts via the NVIC. It doesn't enable any specific interrupt.
    pub fn enable_interrupts(&mut self, nvic: &mut NVIC) {
        nvic.enable(UsartX::INTERRUPT);
    }

    /// Enable the RXRDY interrupt
    ///
    /// The interrupt will not actually work unless the interrupts for this
    /// peripheral have also been enabled via the NVIC. See
    /// [`enable_interrupts`].
    ///
    /// [`enable_interrupts`]: #method.enable_interrupts
    pub fn enable_rxrdy_interrupt(&mut self) {
        self.usart.intenset.write(|w|
            w.rxrdyen().set_bit()
       );
    }

    /// Disable the RXRDY interrupt
    pub fn disable_rxrdy_interrupt(&mut self) {
        self.usart.intenclr.write(|w|
            w.rxrdyclr().set_bit()
        );
    }

    /// Enable the TXRDY interrupt
    ///
    /// The interrupt will not actually work unless the interrupts for this
    /// peripheral have also been enabled via the NVIC. See
    /// [`enable_interrupts`].
    ///
    /// [`enable_interrupts`]: #method.enable_interrupts
    pub fn enable_txrdy_interrupt(&mut self) {
        self.usart.intenset.write(|w|
            w.txrdyen().set_bit()
        );
    }

    /// Disable the TXRDY interrupt
    pub fn disable_txrdy_interrupt(&mut self) {
        self.usart.intenclr.write(|w|
            w.txrdyclr().set_bit()
        );
    }
}

impl<UsartX> Read<u8> for USART<UsartX, init_state::Enabled>
    where UsartX: Peripheral,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let ref uart = self.usart;

        let stat = uart.stat.read();

        if stat.rxbrk().bit_is_set() {
            return Err(nb::Error::WouldBlock);
        }
        if stat.overrunint().bit_is_set() {
            return Err(nb::Error::Other(Error::Overrun));
        }

        if stat.rxrdy().bit_is_set() {
            // It's important to read this register all at once, as reading
            // it changes the status flags.
            let rx_dat_stat = uart.rxdatstat.read();

            if rx_dat_stat.framerr().bit_is_set() {
                return Err(nb::Error::Other(Error::Framing));
            }
            if rx_dat_stat.parityerr().bit_is_set() {
                return Err(nb::Error::Other(Error::Parity));
            }
            if rx_dat_stat.rxnoise().bit_is_set() {
                return Err(nb::Error::Other(Error::Noise));
            }

            // `bits` returns `u16`, but at most 9 bits are used. We've
            // configured UART to use only 8 bits, so we can safely cast to
            // `u8`.
            return Ok(rx_dat_stat.rxdat().bits() as u8);
        }
        else {
            return Err(nb::Error::WouldBlock);
        }
    }
}

impl<UsartX> Write<u8> for USART<UsartX, init_state::Enabled>
    where UsartX: Peripheral,
{
    type Error = !;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        if self.usart.stat.read().txrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        unsafe {
            self.usart.txdat.write(|w| w.txdat().bits(word as u16));
        }

        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.usart.stat.read().txidle().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }
}

impl<UsartX> BlockingWriteDefault<u8> for USART<UsartX, init_state::Enabled>
    where UsartX: Peripheral,
{}

impl<UsartX> fmt::Write for USART<UsartX>
    where
        Self  : BlockingWriteDefault<u8>,
        UsartX: Peripheral,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use ::prelude::*;

        self.bwrite_all(s.as_bytes())
            .map_err(|_| fmt::Error)?;
        block!(self.flush())
            .map_err(|_| fmt::Error)?;

        Ok(())
    }
}

impl<UsartX, State> USART<UsartX, State> where State: InitState {
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
    /// [open an issue]: https://github.com/braun-robotics/rust-lpc82x-hal/issues
    pub fn free(self) -> UsartX {
        self.usart
    }
}


/// Internal trait for USART peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Peripheral:
    Deref<Target = raw::usart0::RegisterBlock>
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

impl Peripheral for raw::USART0 {
    const INTERRUPT: Interrupt = Interrupt::UART0;

    type Rx = swm::U0_RXD;
    type Tx = swm::U0_TXD;
}

impl Peripheral for raw::USART1 {
    const INTERRUPT: Interrupt = Interrupt::UART1;

    type Rx = swm::U1_RXD;
    type Tx = swm::U1_TXD;
}

impl Peripheral for raw::USART2 {
    const INTERRUPT: Interrupt = Interrupt::UART2;

    type Rx = swm::U2_RXD;
    type Tx = swm::U2_TXD;
}


/// Represents a UART baud rate
///
/// Can be passed to [`USART::enable`] to configure the baud rate for a USART
/// peripheral.
pub struct BaudRate<'frg> {
    _uartfrg: &'frg UARTFRG<'frg>,

    /// USART Baud Rate Generator divider value
    ///
    /// See user manual, section 13.6.9.
    brgval: u16,
}

impl<'frg> BaudRate<'frg> {
    /// Create a `BaudRate` instance
    ///
    /// Creates a `BaudRate` instance from two components: A reference to the
    /// [`UARTFRG`] and the BRGVAL.
    ///
    /// The [`UARTFRG`] controls U_PCLK, the clock that is shared by all USART
    /// peripherals. Please configure it before attempting to create a
    /// `BaudRate`. By keeping a reference to it, `BaudRate` ensures that U_PCLK
    /// cannot be changes as long as the `BaudRate` instance exists.
    ///
    /// BRGVAL is an additional divider value that divides the shared baud rate
    /// to allow individual USART peripherals to use different baud rates. A
    /// value of `0` means that U_PCLK is used directly, `1` means that U_PCLK
    /// is divided by 2 before using it, `2` means it's divided by 3, and so on.
    ///
    /// Please refer to the user manual, section 13.3.1, for further details.
    pub fn new(uartfrg : &'frg UARTFRG<'frg>, brgval : u16) -> Self {
        Self {
            _uartfrg: uartfrg,
            brgval  : brgval,
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

    /// Character received, while receiver buffer was still in use
    Overrun,

    /// Parity error detected in received character
    Parity,
}
