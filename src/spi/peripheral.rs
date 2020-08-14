use core::convert::Infallible;

use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};

use crate::{
    dma::{self, transfer::state::Ready},
    init_state::{Disabled, Enabled},
    pac::spi0::cfg::MASTER_A,
    swm, syscon,
};

use super::{Clock, ClockSource, Instance, Interrupts, SlaveSelect, Transfer};

/// Interface to a SPI peripheral
///
/// Controls the SPI. Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// # `embedded-hal` traits
///
/// - [`embedded_hal::spi::FullDuplex`] for asynchronous transfers
/// - [`embedded_hal::blocking::spi::Transfer`] for synchronous transfers
/// - [`embedded_hal::blocking::spi::Write`] for synchronous writes
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
/// [`embedded_hal::spi::FullDuplex`]: #impl-FullDuplex%3Cu8%3E
/// [`embedded_hal::blocking::spi::Transfer`]: #impl-Transfer%3CW%3E
/// [`embedded_hal::blocking::spi::Write`]: #impl-Write%3CW%3E
pub struct SPI<I, State> {
    spi: I,
    _state: State,
}

impl<I> SPI<I, Disabled>
where
    I: Instance,
{
    pub(crate) fn new(spi: I) -> Self {
        Self {
            spi,
            _state: Disabled,
        }
    }

    /// Enable the SPI peripheral in master mode
    ///
    /// This method is only available, if `SPI` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `SPI` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// # Examples
    ///
    /// Please refer to the [module documentation] for a full example.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [module documentation]: index.html
    pub fn enable_as_master<SckPin, MosiPin, MisoPin, CLOCK>(
        self,
        clock: &Clock<CLOCK>,
        syscon: &mut syscon::Handle,
        mode: Mode,
        _sck: swm::Function<I::Sck, swm::state::Assigned<SckPin>>,
        _mosi: swm::Function<I::Mosi, swm::state::Assigned<MosiPin>>,
        _miso: swm::Function<I::Miso, swm::state::Assigned<MisoPin>>,
    ) -> SPI<I, Enabled<Master>>
    where
        CLOCK: ClockSource,
    {
        self.enable::<CLOCK>(syscon);

        self.spi
            .div
            .write(|w| unsafe { w.divval().bits(clock.divval) });

        self.configure(mode, MASTER_A::MASTER_MODE);

        SPI {
            spi: self.spi,
            _state: Enabled(Master),
        }
    }

    /// Enable the SPI peripheral in slave mode
    ///
    /// This method is only available, if `SPI` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `SPI` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable_as_slave<C, SckPin, MosiPin, MisoPin, Ssel, SselPin>(
        self,
        _clock: &C,
        syscon: &mut syscon::Handle,
        mode: Mode,
        _sck: swm::Function<I::Sck, swm::state::Assigned<SckPin>>,
        _mosi: swm::Function<I::Mosi, swm::state::Assigned<MosiPin>>,
        _miso: swm::Function<I::Miso, swm::state::Assigned<MisoPin>>,
        _ssel: swm::Function<Ssel, swm::state::Assigned<SselPin>>,
    ) -> SPI<I, Enabled<Slave>>
    where
        C: ClockSource,
        Ssel: SlaveSelect<I>,
    {
        self.enable::<C>(syscon);
        self.configure(mode, MASTER_A::SLAVE_MODE);

        SPI {
            spi: self.spi,
            _state: Enabled(Slave),
        }
    }

    fn enable<C>(&self, syscon: &mut syscon::Handle)
    where
        C: ClockSource,
    {
        syscon.enable_clock(&self.spi);
        C::select(&self.spi, syscon);
    }

    fn configure(&self, mode: Mode, master: MASTER_A) {
        self.spi.cfg.write(|w| {
            match mode.polarity {
                Polarity::IdleHigh => {
                    w.cpol().high();
                }
                Polarity::IdleLow => {
                    w.cpol().low();
                }
            }
            match mode.phase {
                Phase::CaptureOnFirstTransition => {
                    w.cpha().clear_bit();
                }
                Phase::CaptureOnSecondTransition => {
                    w.cpha().set_bit();
                }
            }
            w.master().variant(master);
            w.enable().enabled();
            w
        });

        // Configure word length.
        self.spi.txctl.write(|w| {
            // 8 bit length
            unsafe { w.len().bits(7) }
        });

        // Configuring the word length via TXCTL has no effect until TXDAT is
        // written, so we're doing this here. This is not disruptive, as in
        // master mode, we'll usually overwrite this anyway when starting a
        // transaction, while in slave mode, we actually need some dummy data in
        // TXDAT when receiving the first byte, to prevent a TX underrun error.
        self.spi.txdat.write(|w| unsafe { w.data().bits(0xff) });
    }
}

impl<I, Mode> SPI<I, Enabled<Mode>>
where
    I: Instance,
{
    /// Enable interrupts
    ///
    /// Enables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    pub fn enable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.enable(&self.spi);
    }

    /// Disable interrupts
    ///
    /// Disables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    pub fn disable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.disable(&self.spi);
    }

    /// Indicates whether the SPI instance is ready to receive
    ///
    /// Corresponds to the RXRDY flag in the STAT register.
    pub fn is_ready_to_receive(&self) -> bool {
        self.spi.stat.read().rxrdy().bit_is_set()
    }

    /// Indicates whether the SPI instance is ready to transmit
    ///
    /// Corresponds to the TXRDY flag in the STAT register.
    pub fn is_ready_to_transmit(&self) -> bool {
        self.spi.stat.read().txrdy().bit_is_set()
    }

    /// Indicates whether a slave select signal has been asserted
    ///
    /// Corresponds to the SSA flag in the STAT register. The flag is cleared
    /// before this method returns.
    pub fn is_slave_select_asserted(&self) -> bool {
        // Can't read field through API. Issue:
        // https://github.com/lpc-rs/lpc-pac/issues/52
        let flag = self.spi.stat.read().bits() & (0x1 << 4) != 0;
        self.spi.stat.write(|w| w.ssa().set_bit());
        flag
    }

    /// Indicates whether a slave select signal has been deasserted
    ///
    /// Corresponds to the SSD flag in the STAT register. The flag is cleared
    /// before this method returns.
    pub fn is_slave_select_deasserted(&self) -> bool {
        // Can't read field through API. Issue:
        // https://github.com/lpc-rs/lpc-pac/issues/52
        let flag = self.spi.stat.read().bits() & (0x1 << 5) != 0;
        self.spi.stat.write(|w| w.ssd().set_bit());
        flag
    }

    /// Indicates whether the master is currently idle
    ///
    /// Corresponds to the MSTIDLE flag in the STAT register.
    pub fn is_master_idle(&self) -> bool {
        self.spi.stat.read().mstidle().bit_is_set()
    }

    /// Disable the SPI peripheral
    ///
    /// This method is only available, if `SPI` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `SPI` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(self, syscon: &mut syscon::Handle) -> SPI<I, Disabled> {
        syscon.disable_clock(&self.spi);

        SPI {
            spi: self.spi,
            _state: Disabled,
        }
    }
}

impl<I> SPI<I, Enabled<Master>>
where
    I: Instance,
{
    /// Start an SPI transfer using DMA
    ///
    /// Sends all words in the provided buffer, writing the replies back into
    /// it.
    ///
    /// # Panics
    ///
    /// Panics, if the length of `buffer` is 0 or larger than 1024.
    pub fn transfer_all(
        self,
        buffer: &'static mut [u8],
        rx_channel: dma::Channel<I::RxChannel, Enabled>,
        tx_channel: dma::Channel<I::TxChannel, Enabled>,
    ) -> Transfer<Ready, I> {
        Transfer::new(self, buffer, rx_channel, tx_channel)
    }
}

impl<I> SPI<I, Enabled<Slave>>
where
    I: Instance,
{
    /// Receive a word
    pub fn receive(&mut self) -> nb::Result<u8, RxOverrunError> {
        let stat = self.spi.stat.read();

        // Can't read field through API. Issue:
        // https://github.com/lpc-rs/lpc-pac/issues/52
        if stat.bits() & (0x1 << 2) != 0 {
            return Err(nb::Error::Other(RxOverrunError));
        }
        if stat.rxrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(self.spi.rxdat.read().rxdat().bits() as u8)
    }

    /// Transmit a word
    pub fn transmit(&mut self, word: u8) -> nb::Result<(), TxUnderrunError> {
        let stat = self.spi.stat.read();

        // Can't read field through API. Issue:
        // https://github.com/lpc-rs/lpc-pac/issues/52
        if stat.bits() & (0x1 << 3) != 0 {
            return Err(nb::Error::Other(TxUnderrunError));
        }
        if stat.txrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        self.spi
            .txdat
            .write(|w| unsafe { w.data().bits(word as u16) });

        Ok(())
    }
}

impl<I, State> SPI<I, State> {
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
        self.spi
    }
}

impl<I: Instance> FullDuplex<u8> for SPI<I, Enabled<Master>> {
    type Error = Infallible;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.spi.stat.read().rxrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(self.spi.rxdat.read().rxdat().bits() as u8)
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        if self.spi.stat.read().txrdy().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        self.spi
            .txdat
            .write(|w| unsafe { w.data().bits(word as u16) });

        Ok(())
    }
}

impl<I: Instance> embedded_hal::blocking::spi::transfer::Default<u8>
    for SPI<I, Enabled<Master>>
{
}

impl<I: Instance> embedded_hal::blocking::spi::write::Default<u8>
    for SPI<I, Enabled<Master>>
{
}

/// Indicates that SPI is in master mode
///
/// Used as a type parameter on [`SPI`].
///
/// [`SPI`]: struct.SPI.html
pub struct Master;

/// Indicates that SPI is in slave mode
///
/// Used as a type parameter on [`SPI`].
///
/// [`SPI`]: struct.SPI.html
pub struct Slave;

/// Receiver Overrun Error
#[derive(Debug)]
pub struct RxOverrunError;

/// Transmitter Underrun Error
#[derive(Debug)]
pub struct TxUnderrunError;
