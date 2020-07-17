use core::convert::Infallible;

use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};

use crate::{init_state, swm, syscon};

use super::{Clock, ClockSource, Instance, Interrupts};

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

impl<I> SPI<I, init_state::Disabled>
where
    I: Instance,
{
    pub(crate) fn new(spi: I) -> Self {
        Self {
            spi,
            _state: init_state::Disabled,
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
    ) -> SPI<I, init_state::Enabled<Master>>
    where
        CLOCK: ClockSource,
    {
        self.enable::<CLOCK>(syscon);

        self.spi
            .div
            .write(|w| unsafe { w.divval().bits(clock.divval) });

        self.configure(mode, true);

        SPI {
            spi: self.spi,
            _state: init_state::Enabled(Master),
        }
    }

    fn enable<C>(&self, syscon: &mut syscon::Handle)
    where
        C: ClockSource,
    {
        syscon.enable_clock(&self.spi);
        C::select(&self.spi, syscon);
    }

    fn configure(&self, mode: Mode, master: bool) {
        self.spi.txctl.write(|w| {
            // 8 bit length
            unsafe { w.len().bits(7) }
        });

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
            match master {
                true => {
                    w.master().master_mode();
                }
                false => {
                    w.master().slave_mode();
                }
            }
            w.enable().enabled();
            w
        });
    }
}

impl<I, Mode> SPI<I, init_state::Enabled<Mode>>
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
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> SPI<I, init_state::Disabled> {
        syscon.disable_clock(&self.spi);

        SPI {
            spi: self.spi,
            _state: init_state::Disabled,
        }
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

impl<I: Instance> FullDuplex<u8> for SPI<I, init_state::Enabled<Master>> {
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
    for SPI<I, init_state::Enabled<Master>>
{
}

impl<I: Instance> embedded_hal::blocking::spi::write::Default<u8>
    for SPI<I, init_state::Enabled<Master>>
{
}

/// Indicates that SPI is in master mode
pub struct Master;

/// Indicates that SPI is in slave mode
pub struct Slave;
