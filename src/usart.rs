//! APIs for the USART peripherals
//!
//! See user manual, chapter 13.


use core::ops::Deref;

use embedded_hal::serial::Read;
use lpc82x::{
    self,
    Interrupt,
    NVIC,
};
use nb;

use Pin;
use init_state::{
    self,
    InitState,
};
use swm::{
    self,
    SWM,
};
use syscon::{
    self,
    UARTFRG,
};


/// Write half of a serial interface
///
/// # License Note
///
/// This trait is a modified version of the same trait from embedded-hal and is
/// subject to embedded-hal's license. It has been [proposed] for inclusion in
/// embedded-hal.
///
/// [proposed]: https://github.com/japaric/embedded-hal/pull/22
pub trait Write<Word> {
    /// Write error
    type Error;

    /// Writes a single word to the serial interface
    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error>;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error>;
}


/// Interface to the USART peripherals
///
/// Each instance of `USART` expects to have full ownership of one USART
/// peripheral. Don't use [`lpc82x::USART0`], [`lpc82x::USART1`], or
/// [`lpc82x::USART2`] directly, unless you know what you're doing.
///
/// [`lpc82x::USART0`]: ../../lpc82x/struct.USART0.html
/// [`lpc82x::USART1`]: ../../lpc82x/struct.USART1.html
/// [`lpc82x::USART2`]: ../../lpc82x/struct.USART2.html
pub struct USART<
    'usart,
    UsartX: 'usart,
    State : InitState = init_state::Initialized,
> {
    usart : &'usart UsartX,
    _state: State,
}

impl<'usart, UsartX> USART<'usart, UsartX, init_state::Unknown>
    where
        UsartX            : Peripheral,
        for<'a> &'a UsartX: syscon::ClockControl + syscon::ResetControl,
{
    pub(crate) fn new(usart: &'usart UsartX) -> Self {
        USART {
            usart : usart,
            _state: init_state::Unknown,
        }
    }

    /// Initializes a USART peripheral
    ///
    /// # Limitations
    ///
    /// When using multiple USARTs at the same time, you must take care that
    /// their baud rate settings don't interfere with each other. Please refer
    /// to the documentation of [`BaudRate`] for full details.
    ///
    /// [`BaudRate`]: struct.BaudRate.html
    pub fn init<Rx: Pin, Tx: Pin>(mut self,
        baud_rate: &BaudRate,
        syscon   : &mut syscon::Api,
        swm      : &mut SWM,
    ) -> nb::Result<USART<'usart, UsartX, init_state::Initialized>, !> {
        syscon.enable_clock(&mut self.usart);
        syscon.clear_reset(&mut self.usart);

        swm.assign_pin::<UsartX::Rx, Rx>();
        swm.assign_pin::<UsartX::Tx, Tx>();

        self.usart.brg.write(|w| unsafe { w.brgval().bits(baud_rate.brg_val) });

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
            _state: init_state::Initialized,
        })
    }
}

impl<'usart, UsartX> USART<'usart, UsartX>
    where
        UsartX            : Peripheral,
        for<'a> &'a UsartX: syscon::ClockControl + syscon::ResetControl,
{
    /// Enables the USART interrupts
    ///
    /// Enables the interrupts for this USART peripheral. This only enables
    /// interrupts in general. It doesn't enable any specific interrupt. Other
    /// methods must be used for this.
    pub fn enable_interrupts(&mut self, nvic: &NVIC) {
        nvic.enable(UsartX::INTERRUPT);
    }

    /// Enable RXRDY interrupt
    pub fn enable_rxrdy_interrupt(&mut self) {
        self.usart.intenset.write(|w|
            w.rxrdyen().set_bit()
       );
    }

    /// Disable RXRDY interrupt
    pub fn disable_rxrdy_interrupt(&mut self) {
        self.usart.intenclr.write(|w|
            w.rxrdyclr().set_bit()
        );
    }

    /// Enable TXRDY interrupt
    pub fn enable_txrdy_interrupt(&mut self) {
        self.usart.intenset.write(|w|
            w.txrdyen().set_bit()
        );
    }

    /// Disable TXRDY interrupt
    pub fn disable_txrdy_interrupt(&mut self) {
        self.usart.intenclr.write(|w|
            w.txrdyclr().set_bit()
        );
    }
}

impl<'usart, UsartX> Read<u8> for USART<'usart, UsartX>
    where
        UsartX            : Peripheral,
        for<'a> &'a UsartX: syscon::ClockControl + syscon::ResetControl,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let uart = self.usart;

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

impl<'usart, UsartX> Write<u8> for USART<'usart, UsartX>
    where
        UsartX            : Peripheral,
        for<'a> &'a UsartX: syscon::ClockControl + syscon::ResetControl,
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

impl<'usart, UsartX> blocking::Write<u8> for USART<'usart, UsartX>
    where
        UsartX            : Peripheral,
        for<'a> &'a UsartX: syscon::ClockControl + syscon::ResetControl,
{
    type Error = !;

    fn write_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        blocking::write_all(self, buffer)
    }
}


/// Implemented for all USART peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// The trait definition comes with some complexity that is caused by the fact
/// that the required `Deref` implementation is implemented for `Self`, while
/// the other traits required are implemented for `&Self`. This should be
/// resolved once we pick up some changes to upstream dependencies that are
/// currently coming down the pipe.
pub trait Peripheral:
    Deref<Target = lpc82x::usart0::RegisterBlock>
    where
        for<'a> &'a Self: syscon::ClockControl,
        for<'a> &'a Self: syscon::ResetControl,
{
    /// The interrupt that is triggered for this USART peripheral
    const INTERRUPT: Interrupt;

    /// The movable function that needs to be assigned to this USART's RX pin
    type Rx: swm::MovableFunction;

    /// The movable function that needs to be assigned to this USART's TX pin
    type Tx: swm::MovableFunction;
}

impl Peripheral for lpc82x::USART0 {
    const INTERRUPT: Interrupt = Interrupt::UART0;

    type Rx = swm::U0_RXD;
    type Tx = swm::U0_TXD;
}

impl Peripheral for lpc82x::USART1 {
    const INTERRUPT: Interrupt = Interrupt::UART1;

    type Rx = swm::U1_RXD;
    type Tx = swm::U1_TXD;
}

impl Peripheral for lpc82x::USART2 {
    const INTERRUPT: Interrupt = Interrupt::UART2;

    type Rx = swm::U2_RXD;
    type Tx = swm::U2_TXD;
}


/// Baud rate for a UART unit
///
/// Can be passed to [`USART::init`] to configure the baud rate for the USART
/// peripheral.
///
/// # Limitations
///
/// The fields [`clk_div`], [`frg_mult`], and [`frg_div`] represent global
/// configuration values that are shared between all USART peripherals. Only
/// [`brg_val`] can be set independently for each USART.
///
/// When running multiple USARTs, it is the user's responsibility to make sure
/// that all the `BaudRate`s passed to them have identical values for `clk_div`,
/// `frg_mult`, and `frg_div`.
///
/// [`USART::init`]: struct.USART.html#method.init
/// [`clk_div`]: #structfield.clk_div
/// [`frg_mult`]: #structfield.frg_mult
/// [`frg_div`]: #structfield.frg_div
/// [`brg_val`]: #structfield.brg_val
pub struct BaudRate<'frg> {
    _uartfrg: &'frg mut UARTFRG<'frg>,

    /// USART Baud Rate Generator divider value
    ///
    /// See user manual, section 13.6.9.
    brg_val: u16,
}

impl<'frg> BaudRate<'frg> {
    /// Create a `BaudRate` instance by providing the register values
    pub fn new(uartfrg : &'frg mut UARTFRG<'frg>, brg_val : u16) -> Self {
        Self {
            _uartfrg: uartfrg,
            brg_val : brg_val,
        }
    }

    /// Returns a `BaudRate` instance for 115200 baud
    ///
    /// # Limitations
    ///
    /// This function assumes the default configuration for the main clock,
    /// namely that the IRC running at 12 MHz is used. If you have made any
    /// changes to the main clock configuration, please don't use this function
    /// and create a `BaudRate` manually instead.
    pub fn baud_115200(uartfrg: &'frg mut UARTFRG<'frg>) -> Self {
        // The common peripheral clock for all UART units, U_PCLK, needs to be
        // set to 16 times the desired baud rate. This results in a frequency of
        // 1843200 Hz for U_PLCK.
        //
        // We assume the main clock runs at 12 Mhz. To get close to the desired
        // frequency for U_PLCK, we divide that by 6 using UARTCLKDIV, resulting
        // in a frequency of 2 Mhz.
        //
        // To get to the desired 1843200 Hz, we need to further divide the
        // frequency using the fractional baud rate generator. The fractional
        // baud rate generator divides the frequency by `1 + MULT/DIV`.
        //
        // DIV must always be 256. To achieve this, we need to set the
        // UARTFRGDIV to 0xff. MULT can then be fine-tuned to get as close as
        // possible to the desired value. We choose the value 22, which we write
        // into UARTFRGMULT.
        //
        // Finally, we can set an additional divider value for the UART unit by
        // writing to the BRG register. As we are already close enough to the
        // desired value, we write 0, resulting in no further division.
        //
        // All of this is somewhat explained in the user manual, section 13.3.1.
        uartfrg.set_clkdiv(6);
        uartfrg.set_frgmult(22);
        uartfrg.set_frgdiv(0xff);

        BaudRate::new(uartfrg, 0)
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


/// Platform-independent, blocking code
///
/// This is based on [a proposal] for embedded-hal.
///
/// [a proposal]: https://github.com/japaric/embedded-hal/issues/18
pub mod blocking {
    use usart as serial;


    /// Write half of a serial interface (blocking implementation)
    pub trait Write<Word> {
        /// The type or error that can occur when writing
        type Error;

        /// Writes a slice, blocking until everything has been written
        fn write_all(&mut self, buffer: &[Word]) -> Result<(), Self::Error>;
    }


    /// Implements a blocking write over a non-blocking [`Write`]
    ///
    /// Can be used by HAL implementations as a default implementation for
    /// [`blocking::Write`].
    ///
    /// [`Write`]: ../trait.Write.html
    /// [`blocking::Write`]: trait.Write.html
    pub fn write_all<S, Word>(serial: &mut S, buffer: &[Word])
        -> Result<(), S::Error>
        where
            S   : serial::Write<Word>,
            Word: Copy,
    {
        for &word in buffer {
            block!(serial.write(word))?;
        }
        block!(serial.flush())?;

        Ok(())
    }
}
