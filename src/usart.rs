//! API for the USART peripherals
//!
//! The USART peripheral is described in the user manual, chapter 13.
//!
//! Currently, only some UART functionality is implemented.
//!
//! # Examples
//!
//! ``` no_run
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//!
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::{
//!     GPIO,
//!     SYSCON,
//!     SWM,
//! };
//! use lpc82x_hal::usart::{
//!     BaudRate,
//!     USART,
//! };
//!
//! let mut peripherals = lpc82x::Peripherals::take().unwrap();
//!
//! let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
//! let     swm    = SWM::new(peripherals.SWM);
//! let     gpio   = GPIO::new(peripherals.GPIO_PORT);
//! let     usart0 = USART::new(peripherals.USART0);
//!
//! let mut swm_handle = swm.handle.enable(&mut syscon.handle);
//!
//! // Set baud rate to 115200 baud
//! //
//! // The common peripheral clock for all UART units, U_PCLK, needs to be set
//! // to 16 times the desired baud rate. This results in a frequency of
//! // 1843200 Hz for U_PLCK.
//! //
//! // We assume the main clock runs at 12 Mhz. To get close to the desired
//! // frequency for U_PLCK, we divide that by 6 using UARTCLKDIV, resulting in
//! // a frequency of 2 Mhz.
//! //
//! // To get to the desired 1843200 Hz, we need to further divide the frequency
//! // using the fractional baud rate generator. The fractional baud rate
//! // generator divides the frequency by `1 + MULT/DIV`.
//! //
//! // DIV must always be 256. To achieve this, we need to set the UARTFRGDIV to
//! // 0xff. MULT can then be fine-tuned to get as close as possible to the
//! // desired value. We choose the value 22, which we write into UARTFRGMULT.
//! //
//! // Finally, we can set an additional divider value for the UART unit by
//! // passing it as an argument to `BaudRate::new` (this will set the BRG
//! // register). As we are already close enough to the desired value, we pass
//! // 0, resulting in no further division.
//! //
//! // All of this is somewhat explained in the user manual, section 13.3.1.
//! syscon.uartfrg.set_clkdiv(6);
//! syscon.uartfrg.set_frgmult(22);
//! syscon.uartfrg.set_frgdiv(0xff);
//! let baud_rate = BaudRate::new(&syscon.uartfrg, 0);
//!
//! // Prepare PIO0_0 and PIO0_4. The `init` method we call below needs pins to
//! // assign the USART's movable function to. For that, the pins need to be
//! // unused. Since PIO0_0 and PIO0_4 are unused by default, we just have to
//! // promise the API that we didn't change the default state up till now.
//! let pio0_0 = unsafe { gpio.pins.pio0_0.affirm_default_state() };
//! let pio0_4 = unsafe { gpio.pins.pio0_4.affirm_default_state() };
//!
//! // We also need to provide USART0's movable functions. Those need to be
//! // unassigned, and since they are unassigned by default, we just need to
//! // promise the API that we didn't change them.
//! let u0_rxd = unsafe { swm.movable_functions.u0_rxd.affirm_default_state() };
//! let u0_txd = unsafe { swm.movable_functions.u0_txd.affirm_default_state() };
//!
//! // Initialize USART0. This should never fail, as the only reason `init`
//! // returns a `Result::Err` is when the transmitter is busy, which it
//! // shouldn't be right now.
//! let mut serial = usart0
//!     .enable(
//!         &baud_rate,
//!         &mut syscon.handle,
//!         pio0_0,
//!         pio0_4,
//!         u0_rxd,
//!         u0_txd,
//!         &mut swm_handle,
//!     )
//!     .expect("UART initialization shouldn't fail");
//!
//! // Write a string, blocking until it has finished writing
//! serial.bwrite_all(b"Hello, world!");
//! ```
//!
//! [`USART`]: struct.USART.html


use core::ops::Deref;

use embedded_hal::blocking::serial::write::Default as BlockingWriteDefault;
use embedded_hal::serial::{
    Read,
    Write,
};
use nb;

use gpio::{
    pin_state,
    Pin,
    PinTrait,
};
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
    movable_function,
    InputFunction,
    OutputFunction,
};
use syscon::{
    self,
    UARTFRG,
};


/// Interface to a USART peripheral
///
/// Please refer to the [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct USART<UsartX, State : InitState = init_state::Enabled> {
    usart : UsartX,
    _state: State,
}

impl<UsartX> USART<UsartX, init_state::Unknown>
    where UsartX: Peripheral,
{
    /// Create an instance of `USART`
    pub fn new(usart: UsartX) -> Self {
        USART {
            usart : usart,
            _state: init_state::Unknown,
        }
    }
}

impl<UsartX, State> USART<UsartX, State>
    where
        UsartX: Peripheral,
        State : init_state::NotEnabled
{
    /// Enable a USART peripheral
    ///
    /// This method is only available, if `USART` is not already in the
    /// [`Enabled`] state. Code that attempts to call this method when the USART
    /// is already enabled will not compile.
    ///
    /// Consumes this instance of `USART` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// # Limitations
    ///
    /// For USART to function correctly, the UARTFRG reset must be cleared. This
    /// is the default case, so unless you have messed with those settings, you
    /// should be good.
    ///
    /// # Examples
    ///
    /// Please refer to the [module documentation] for a full example.
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`BaudRate`]: struct.BaudRate.html
    /// [module documentation]: index.html
    pub fn enable<Rx: PinTrait, Tx: PinTrait>(mut self,
        baud_rate: &BaudRate,
        syscon   : &mut syscon::Handle,
        rx       : Pin<Rx, pin_state::Unused>,
        tx       : Pin<Tx, pin_state::Unused>,
        rxd      : UsartX::Rx,
        txd      : UsartX::Tx,
        swm      : &mut swm::Handle,
    )
        -> nb::Result<USART<UsartX, init_state::Enabled>, !>
        where
            UsartX::Rx: movable_function::Assign<Rx> + InputFunction,
            UsartX::Tx: movable_function::Assign<Tx> + OutputFunction,
    {
        syscon.enable_clock(&mut self.usart);
        syscon.clear_reset(&mut self.usart);

        rx
            .into_swm_pin()
            .assign_input_function(rxd, swm);
        tx
            .into_swm_pin()
            .assign_output_function(txd, swm);

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

impl<UsartX, State> USART<UsartX, State>
    where
        UsartX: Peripheral,
        State : init_state::NotDisabled
{
    /// Disable a USART peripheral
    ///
    /// This method is only available, if `USART` is not already in the
    /// [`Disabled`] state. Code that attempts to call this method when the
    /// USART is already disabled will not compile.
    ///
    /// Consumes this instance of `USART` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable<Rx: PinTrait, Tx: PinTrait>(mut self,
        syscon: &mut syscon::Handle,
    )
        -> USART<UsartX, init_state::Disabled>
    {
        syscon.disable_clock(&mut self.usart);

        USART {
            usart : self.usart,
            _state: init_state::Disabled,
        }
    }
}

impl<UsartX> USART<UsartX, init_state::Enabled>
    where UsartX: Peripheral,
{
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

impl<UsartX> Read<u8> for USART<UsartX>
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

impl<UsartX> Write<u8> for USART<UsartX>
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

impl<UsartX> BlockingWriteDefault<u8> for USART<UsartX>
    where UsartX: Peripheral,
{}


/// Internal trait for USART peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// The trait definition comes with some complexity that is caused by the fact
/// that the required [`Deref`] implementation is implemented for `Self`, while
/// the other traits required are implemented for `&Self`. This should be
/// resolved once we pick up some changes to upstream dependencies that are
/// currently coming down the pipe.
///
/// [`Deref`]: https://doc.rust-lang.org/std/ops/trait.Deref.html
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

    type Rx = swm::U0_RXD<movable_function::state::Unassigned>;
    type Tx = swm::U0_TXD<movable_function::state::Unassigned>;
}

impl Peripheral for raw::USART1 {
    const INTERRUPT: Interrupt = Interrupt::UART1;

    type Rx = swm::U1_RXD<movable_function::state::Unassigned>;
    type Tx = swm::U1_TXD<movable_function::state::Unassigned>;
}

impl Peripheral for raw::USART2 {
    const INTERRUPT: Interrupt = Interrupt::UART2;

    type Rx = swm::U2_RXD<movable_function::state::Unassigned>;
    type Tx = swm::U2_TXD<movable_function::state::Unassigned>;
}


/// Represents a UART baud rate
///
/// Can be passed to [`USART::init`] to configure the baud rate for a USART
/// peripheral.
///
/// [`USART::init`]: struct.USART.html#method.init
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
    ///
    /// [`UARTFRG`]: ../syscon/struct.UARTFRG.html
    pub fn new(uartfrg : &'frg UARTFRG<'frg>, brgval : u16) -> Self {
        Self {
            _uartfrg: uartfrg,
            brgval  : brgval,
        }
    }
}


/// USART error
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
