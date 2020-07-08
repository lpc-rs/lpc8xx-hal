use core::convert::TryInto as _;

use embedded_hal::blocking::i2c;

use crate::{init_state, swm, syscon};

use super::{master, Clock, ClockSource, Error, Instance, Interrupts};

/// Interface to an I2C peripheral
///
/// Please refer to the [module documentation] for more information.
///
/// # Limitations
///
/// This API only supports master mode.
///
/// Additional limitations are documented on the specific methods that they
/// apply to.
///
/// # `embedded-hal` traits
/// - [`embedded_hal::blocking::i2c::Read`] for synchronous reading
/// - [`embedded_hal::blocking::i2c::Write`] for synchronous writing
///
/// [`embedded_hal::blocking::i2c::Read`]: #impl-Read
/// [`embedded_hal::blocking::i2c::Write`]: #impl-Write
/// [module documentation]: index.html
pub struct I2C<I, State> {
    i2c: I,
    _state: State,
}

impl<I> I2C<I, init_state::Disabled>
where
    I: Instance,
{
    pub(crate) fn new(i2c: I) -> Self {
        I2C {
            i2c: i2c,
            _state: init_state::Disabled,
        }
    }

    /// Enable the I2C peripheral in master mode
    ///
    /// This method is only available, if `I2C` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `I2C` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// # Limitations
    ///
    /// This method does not check that the supplied clock configuration matches
    /// the configuration of the pins. You need to verify manually that this is
    /// the case. What this means exactly may depend on your specific part.
    /// Check out the LPC84x user manual, section 19.4, for example.
    ///
    /// If you don't mess with the IOCON configuration and use I2C clock rates
    /// of up to 400 kHz, you should be fine.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable_master<SdaPin, SclPin, C>(
        mut self,
        clock: &Clock<C>,
        syscon: &mut syscon::Handle,
        _: swm::Function<I::Sda, swm::state::Assigned<SdaPin>>,
        _: swm::Function<I::Scl, swm::state::Assigned<SclPin>>,
    ) -> I2C<I, init_state::Enabled<Master>>
    where
        C: ClockSource,
    {
        syscon.enable_clock(&mut self.i2c);

        C::select(&self.i2c, syscon);

        // We need the I2C mode for the pins set to standard/fast mode,
        // according to the user manual, section 15.3.1. This is already the
        // default value (see user manual, sections 8.5.8 and 8.5.9).

        // Set I2C clock frequency
        self.i2c
            .clkdiv
            .write(|w| unsafe { w.divval().bits(clock.divval) });
        self.i2c.msttime.write(|w| {
            w.mstsclhigh().bits(clock.mstsclhigh);
            w.mstscllow().bits(clock.mstscllow)
        });

        // Enable master mode
        // Set all other configuration values to default.
        self.i2c.cfg.write(|w| w.msten().enabled());

        I2C {
            i2c: self.i2c,
            _state: init_state::Enabled(Master),
        }
    }
}

impl<I> I2C<I, init_state::Enabled<Master>>
where
    I: Instance,
{
    /// Wait while the peripheral is busy
    ///
    /// Once this method returns, the peripheral should either be idle or in a
    /// state that requires software interaction.
    fn wait_for_state(&self, expected: master::State) -> Result<(), Error> {
        while self.i2c.stat.read().mstpending().is_in_progress() {
            Error::read(&self.i2c)?;
        }

        let actual = self.i2c.stat.read().mststate().variant().try_into();
        if Ok(&expected) != actual.as_ref() {
            return Err(Error::UnexpectedState { expected, actual });
        }

        Ok(())
    }
}

impl<I, Mode> I2C<I, init_state::Enabled<Mode>>
where
    I: Instance,
{
    /// Enable interrupts
    ///
    /// Enables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    pub fn enable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.enable(&self.i2c);
    }

    /// Disable interrupts
    ///
    /// Disables all interrupts set to `true` in `interrupts`. Interrupts set to
    /// `false` are not affected.
    pub fn disable_interrupts(&mut self, interrupts: Interrupts) {
        interrupts.disable(&self.i2c);
    }

    /// Read and clear a detected error
    ///
    /// The `read` and `write` methods will return an error and clear it, if one
    /// was detected. However, if multiple errors occur, only one error will be
    /// returned and cleared.
    ///
    /// This method can be used to read and clear all currently detected errors
    /// before resuming normal operation.
    pub fn read_error(&mut self) -> Result<(), Error> {
        Error::read(&self.i2c)
    }
}

impl<I, State> I2C<I, State>
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
        self.i2c
    }
}

impl<I> i2c::Write for I2C<I, init_state::Enabled<Master>>
where
    I: Instance,
{
    type Error = Error;

    /// Write to the I2C bus
    ///
    /// Please refer to the [embedded-hal documentation] for details.
    ///
    /// [embedded-hal documentation]: https://docs.rs/embedded-hal/0.2.1/embedded_hal/blocking/i2c/trait.Write.html#tymethod.write
    fn write(&mut self, address: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.wait_for_state(master::State::Idle)?;

        // Write slave address with rw bit set to 0
        self.i2c
            .mstdat
            .write(|w| unsafe { w.data().bits(address & 0xfe) });

        // Start transmission
        self.i2c.mstctl.write(|w| w.mststart().start());

        for &b in data {
            self.wait_for_state(master::State::TxReady)?;

            // Write byte
            self.i2c.mstdat.write(|w| unsafe { w.data().bits(b) });

            // Continue transmission
            self.i2c.mstctl.write(|w| w.mstcontinue().continue_());
        }

        self.wait_for_state(master::State::TxReady)?;

        // Stop transmission
        self.i2c.mstctl.write(|w| w.mststop().stop());

        Ok(())
    }
}

impl<I> i2c::Read for I2C<I, init_state::Enabled<Master>>
where
    I: Instance,
{
    type Error = Error;

    /// Read from the I2C bus
    ///
    /// Please refer to the [embedded-hal documentation] for details.
    ///
    /// [embedded-hal documentation]: https://docs.rs/embedded-hal/0.2.1/embedded_hal/blocking/i2c/trait.Read.html#tymethod.read
    fn read(
        &mut self,
        address: u8,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.wait_for_state(master::State::Idle)?;

        // Write slave address with rw bit set to 1
        self.i2c
            .mstdat
            .write(|w| unsafe { w.data().bits(address | 0x01) });

        // Start transmission
        self.i2c.mstctl.write(|w| w.mststart().start());

        for b in buffer {
            // Continue transmission
            self.i2c.mstctl.write(|w| w.mstcontinue().continue_());

            self.wait_for_state(master::State::RxReady)?;

            // Read received byte
            *b = self.i2c.mstdat.read().data().bits();
        }

        // Stop transmission
        self.i2c.mstctl.write(|w| w.mststop().stop());

        Ok(())
    }
}

/// Used as a type parameter by [`I2C`] to indicate master mode
///
/// [`I2C`]: struct.I2C.html
pub struct Master;

/// Used as a type parameter by [`I2C`] to indicate slave mode
///
/// [`I2C`]: struct.I2C.html
pub struct Slave;
