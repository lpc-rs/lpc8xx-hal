use core::{fmt, marker::PhantomData};

use cortex_m::interrupt;
use embedded_hal::{
    blocking::serial::write::Default as BlockingWriteDefault, serial::Write,
};
use nb::block;
use void::Void;

use crate::{
    dma::{self, transfer::state::Ready},
    init_state,
    pac::dma0::channel::xfercfg::DSTINC_A,
    pins::{self, Pin},
    swm,
};

use super::{
    flags::{Flag, Interrupts},
    instances::Instance,
    state::{CtsThrottle, Enabled, NoThrottle, Word},
};

/// USART transmitter
///
/// Can be accessed through [`USART`].
///
/// # `embedded-hal` traits
/// - [`embedded_hal::serial::Write`] for non-blocking writes
/// - [`embedded_hal::blocking::serial::Write`] for blocking writes
///
/// [`USART`]: struct.USART.html
/// [`embedded_hal::serial::Write`]: #impl-Write<W>
/// [`embedded_hal::blocking::serial::Write`]: #impl-Write<Word>
pub struct Tx<I, State, Throttle> {
    instance: PhantomData<I>,
    state: PhantomData<State>,
    throttle: Throttle,
}

impl<I, State> Tx<I, State, NoThrottle>
where
    I: Instance,
{
    pub(super) fn new() -> Self {
        Self {
            instance: PhantomData,
            state: PhantomData,
            throttle: NoThrottle,
        }
    }
}

impl<I, W, Mode, Throttle> Tx<I, Enabled<W, Mode>, Throttle>
where
    I: Instance,
    W: Word,
{
    /// Enable RTS signal
    ///
    /// Configure the transmitter to assert the Request to Send (RTS) signal,
    /// when it is ready to send.
    ///
    /// This is a convenience method that ensures the correct RTS function for
    /// this peripheral instance is assigned to a pin. The same effect can be
    /// achieved by just assigning the function using the SWM API.
    pub fn enable_rts<P, S>(
        &mut self,
        function: swm::Function<I::Rts, swm::state::Unassigned>,
        pin: Pin<P, S>,
        swm: &mut swm::Handle,
    ) -> (
        swm::Function<I::Rts, swm::state::Assigned<P>>,
        <Pin<P, S> as swm::AssignFunction<
            I::Rts,
            <I::Rts as swm::FunctionTrait<P>>::Kind,
        >>::Assigned,
    )
    where
        P: pins::Trait,
        S: pins::State,
        Pin<P, S>: swm::AssignFunction<
            I::Rts,
            <I::Rts as swm::FunctionTrait<P>>::Kind,
        >,
        I::Rts: swm::FunctionTrait<P>,
    {
        function.assign(pin, swm)
    }

    /// Disable RTS signal
    ///
    /// Configure the transmitter to no longer assert the Request to Send (RTS)
    /// signal.
    ///
    /// This is a convenience method that ensures the correct RTS function for
    /// this peripheral instance is unassigned. The same effect can be achieved
    /// by just unassigning the function using the SWM API.
    pub fn disable_rts<P, S>(
        &mut self,
        function: swm::Function<I::Rts, swm::state::Assigned<P>>,
        pin: Pin<P, S>,
        swm: &mut swm::Handle,
    ) -> (
        swm::Function<I::Rts, swm::state::Unassigned>,
        <Pin<P, S> as swm::UnassignFunction<
            I::Rts,
            <I::Rts as swm::FunctionTrait<P>>::Kind,
        >>::Unassigned,
    )
    where
        P: pins::Trait,
        S: pins::State,
        Pin<P, S>: swm::UnassignFunction<
            I::Rts,
            <I::Rts as swm::FunctionTrait<P>>::Kind,
        >,
        I::Rts: swm::FunctionTrait<P>,
    {
        function.unassign(pin, swm)
    }

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

impl<I, W, Mode> Tx<I, Enabled<W, Mode>, NoThrottle>
where
    I: Instance,
    W: Word,
{
    /// Enable throttling via CTS signal
    ///
    /// Configure the transmitter to only transmit, while the CTS signal is
    /// asserted.
    pub fn enable_cts_throttling<P>(
        self,
        function: swm::Function<I::Cts, swm::state::Assigned<P>>,
    ) -> Tx<
        I,
        Enabled<W, Mode>,
        CtsThrottle<swm::Function<I::Cts, swm::state::Assigned<P>>>,
    > {
        interrupt::free(|_| {
            // Sound, as we're in a critical section that protects our read-
            // modify-write access.
            let usart = unsafe { &*I::REGISTERS };

            usart.cfg.modify(|_, w| w.ctsen().enabled());
        });

        Tx {
            instance: self.instance,
            state: self.state,
            throttle: CtsThrottle(function),
        }
    }
}

impl<I, W, Mode, Function> Tx<I, Enabled<W, Mode>, CtsThrottle<Function>>
where
    I: Instance,
    W: Word,
{
    /// Disable throttling via CTS signal
    ///
    /// Configure the transmitter to ignore the CTS signal. Returns the SWM
    /// function for the CTS signal, so it can be reused to enable CTS
    /// throttling again, or for something else.
    pub fn disable_cts_throttling(
        self,
    ) -> (Tx<I, Enabled<W, Mode>, NoThrottle>, Function) {
        interrupt::free(|_| {
            // Sound, as we're in a critical section that protects our read-
            // modify-write access.
            let usart = unsafe { &*I::REGISTERS };

            usart.cfg.modify(|_, w| w.ctsen().disabled());
        });

        (
            Tx {
                instance: self.instance,
                state: self.state,
                throttle: NoThrottle,
            },
            self.throttle.0,
        )
    }
}

impl<I, Mode, Throttle> Tx<I, Enabled<u8, Mode>, Throttle>
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

impl<I, W, Mode, Throttle> Write<W> for Tx<I, Enabled<W, Mode>, Throttle>
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

impl<I, W, Mode, Throttle> BlockingWriteDefault<W>
    for Tx<I, Enabled<W, Mode>, Throttle>
where
    I: Instance,
    W: Word,
{
}

impl<I, Mode, Throttle> fmt::Write for Tx<I, Enabled<u8, Mode>, Throttle>
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

impl<I, State, Throttle> crate::private::Sealed for Tx<I, State, Throttle> {}

impl<I, Mode, Throttle> dma::Dest for Tx<I, Enabled<u8, Mode>, Throttle>
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
