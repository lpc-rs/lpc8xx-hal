use core::fmt;

use embedded_hal::{
    blocking::serial::write::Default as BlockingWriteDefault,
    serial::{Read, Write},
};
use void::Void;

use crate::{
    init_state::Disabled,
    pac::NVIC,
    pins,
    swm::{self, FunctionTrait},
    syscon,
};

use super::{
    clock::{Clock, ClockSource},
    flags::{Flag, Interrupts},
    instances::Instance,
    rx::{Error, Rx},
    settings::Settings,
    state::{AsyncMode, Enabled, NoThrottle, Word},
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
    pub tx: Tx<I, State, NoThrottle>,

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

    /// Enable the USART in asynchronous mode
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
    pub fn enable_async<RxPin, TxPin, CLOCK, W>(
        self,
        clock: &Clock<CLOCK>,
        syscon: &mut syscon::Handle,
        _: swm::Function<I::Rx, swm::state::Assigned<RxPin>>,
        _: swm::Function<I::Tx, swm::state::Assigned<TxPin>>,
        settings: Settings<W>,
    ) -> USART<I, Enabled<W, AsyncMode>>
    where
        RxPin: pins::Trait,
        TxPin: pins::Trait,
        I::Rx: FunctionTrait<RxPin>,
        I::Tx: FunctionTrait<TxPin>,
        CLOCK: ClockSource,
        W: Word,
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

impl<I, W, Mode> USART<I, Enabled<W, Mode>>
where
    I: Instance,
    W: Word,
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

    /// Query whether the provided flag is set
    ///
    /// Flags that need to be reset by software will be reset by this operation.
    pub fn is_flag_set(&self, flag: Flag) -> bool {
        flag.is_set::<I>()
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

    /// Enable interrupts
    ///
    /// Enables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc8xx_hal::usart;
    ///
    /// # use lpc8xx_hal::Peripherals;
    /// #
    /// # let mut p = Peripherals::take().unwrap();
    /// #
    /// # let mut syscon = p.SYSCON.split();
    /// # let mut swm    = p.SWM.split();
    /// #
    /// # #[cfg(feature = "82x")]
    /// # let mut swm_handle = swm.handle;
    /// # #[cfg(feature = "845")]
    /// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
    /// #
    /// # #[cfg(feature = "82x")]
    /// # let clock_config = {
    /// #     syscon.uartfrg.set_clkdiv(6);
    /// #     syscon.uartfrg.set_frgmult(22);
    /// #     syscon.uartfrg.set_frgdiv(0xff);
    /// #     usart::Clock::new(&syscon.uartfrg, 0, 16)
    /// # };
    /// # #[cfg(feature = "845")]
    /// # let clock_config = usart::Clock::new_with_baudrate(115200);
    /// #
    /// # let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
    /// #     p.pins.pio0_0.into_swm_pin(),
    /// #     &mut swm_handle,
    /// # );
    /// # let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
    /// #     p.pins.pio0_4.into_swm_pin(),
    /// #     &mut swm_handle,
    /// # );
    /// #
    /// # let mut usart = p.USART0.enable_async(
    /// #     &clock_config,
    /// #     &mut syscon.handle,
    /// #     u0_rxd,
    /// #     u0_txd,
    /// #     usart::Settings::default(),
    /// # );
    /// #
    /// // Enable only RXRDY and TXRDY, leave other interrupts untouched.
    /// usart.enable_interrupts(usart::Interrupts {
    ///     RXRDY: true,
    ///     TXRDY: true,
    ///     .. usart::Interrupts::default()
    /// });
    /// ```
    pub fn enable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.enable::<I>();
    }

    /// Disable interrupts
    ///
    /// Disables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc8xx_hal::usart;
    ///
    /// # use lpc8xx_hal::Peripherals;
    /// #
    /// # let mut p = Peripherals::take().unwrap();
    /// #
    /// # let mut syscon = p.SYSCON.split();
    /// # let mut swm    = p.SWM.split();
    /// #
    /// # #[cfg(feature = "82x")]
    /// # let mut swm_handle = swm.handle;
    /// # #[cfg(feature = "845")]
    /// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
    /// #
    /// # #[cfg(feature = "82x")]
    /// # let clock_config = {
    /// #     syscon.uartfrg.set_clkdiv(6);
    /// #     syscon.uartfrg.set_frgmult(22);
    /// #     syscon.uartfrg.set_frgdiv(0xff);
    /// #     usart::Clock::new(&syscon.uartfrg, 0, 16)
    /// # };
    /// # #[cfg(feature = "845")]
    /// # let clock_config = usart::Clock::new_with_baudrate(115200);
    /// #
    /// # let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
    /// #     p.pins.pio0_0.into_swm_pin(),
    /// #     &mut swm_handle,
    /// # );
    /// # let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
    /// #     p.pins.pio0_4.into_swm_pin(),
    /// #     &mut swm_handle,
    /// # );
    /// #
    /// # let mut usart = p.USART0.enable_async(
    /// #     &clock_config,
    /// #     &mut syscon.handle,
    /// #     u0_rxd,
    /// #     u0_txd,
    /// #     usart::Settings::default(),
    /// # );
    /// #
    /// // Disable only RXRDY and TXRDY, leave other interrupts untouched.
    /// usart.disable_interrupts(usart::Interrupts {
    ///     RXRDY: true,
    ///     TXRDY: true,
    ///     .. usart::Interrupts::default()
    /// });
    /// ```
    pub fn disable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.disable::<I>();
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

impl<I, W, Mode> Read<W> for USART<I, Enabled<W, Mode>>
where
    I: Instance,
    W: Word,
{
    type Error = Error;

    /// Reads a single word from the serial interface
    fn read(&mut self) -> nb::Result<W, Self::Error> {
        self.rx.read()
    }
}

impl<I, W, Mode> Write<W> for USART<I, Enabled<W, Mode>>
where
    I: Instance,
    W: Word,
{
    type Error = Void;

    /// Writes a single word to the serial interface
    fn write(&mut self, word: W) -> nb::Result<(), Self::Error> {
        self.tx.write(word)
    }

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<I, W, Mode> BlockingWriteDefault<W> for USART<I, Enabled<W, Mode>>
where
    I: Instance,
    W: Word,
{
}

impl<I, Mode> fmt::Write for USART<I, Enabled<u8, Mode>>
where
    Self: BlockingWriteDefault<u8>,
    I: Instance,
{
    /// Writes a string slice into this writer, returning whether the write succeeded.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.tx.write_str(s)
    }
}
