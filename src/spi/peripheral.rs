use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};

use crate::{
    init_state, pins,
    swm::{self, FunctionTrait},
    syscon::{self, clock_source::PeripheralClock},
};

use super::{Clock, Instance};

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
pub struct SPI<I, State = init_state::Enabled> {
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

    /// Enable the SPI peripheral
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
    /// [`BaudRate`]: struct.BaudRate.html
    /// [module documentation]: index.html
    pub fn enable<SckPin, MosiPin, MisoPin, CLOCK>(
        self,
        clock: &Clock<CLOCK>,
        syscon: &mut syscon::Handle,
        mode: Mode,
        _: swm::Function<I::Sck, swm::state::Assigned<SckPin>>,
        _: swm::Function<I::Mosi, swm::state::Assigned<MosiPin>>,
        _: swm::Function<I::Miso, swm::state::Assigned<MisoPin>>,
    ) -> SPI<I, init_state::Enabled>
    where
        SckPin: pins::Trait,
        MosiPin: pins::Trait,
        MisoPin: pins::Trait,
        I::Sck: FunctionTrait<SckPin>,
        I::Mosi: FunctionTrait<MosiPin>,
        I::Miso: FunctionTrait<MisoPin>,
        Clock<CLOCK>: PeripheralClock<I>,
    {
        syscon.enable_clock(&self.spi);

        clock.select_clock(syscon);

        self.spi
            .div
            .write(|w| unsafe { w.divval().bits(clock.divval) });

        self.spi.txctl.write(|w| {
            // 8 bit length
            unsafe { w.len().bits(7) }
        });

        self.spi.cfg.write(|w| {
            if mode.polarity == Polarity::IdleHigh {
                w.cpol().high();
            } else {
                w.cpol().low();
            }
            if mode.phase == Phase::CaptureOnFirstTransition {
                w.cpha().clear_bit();
            } else {
                w.cpha().set_bit();
            }
            w.enable().enabled();
            w.master().master_mode()
        });

        SPI {
            spi: self.spi,
            _state: init_state::Enabled(()),
        }
    }
}

impl<I> SPI<I, init_state::Enabled>
where
    I: Instance,
{
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

impl<I: Instance> FullDuplex<u8> for SPI<I> {
    type Error = ();

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.spi.stat.read().rxrdy().bit_is_set() {
            Ok(self.spi.rxdat.read().rxdat().bits() as u8)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        if self.spi.stat.read().txrdy().bit_is_set() {
            self.spi
                .txdat
                .write(|w| unsafe { w.data().bits(word as u16) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<I: Instance> embedded_hal::blocking::spi::transfer::Default<u8>
    for SPI<I>
{
}

impl<I: Instance> embedded_hal::blocking::spi::write::Default<u8> for SPI<I> {}
