use crate::{init_state, swm, syscon};

use super::{Clock, ClockSource, Error, Instance, Interrupts, Master, Slave};

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
/// [module documentation]: index.html
pub struct I2C<I: Instance, State, MasterMode, SlaveMode> {
    /// API for I2C master mode
    pub master: Master<I, State, MasterMode>,

    /// API for I2C slave mode
    pub slave: Slave<I, State, SlaveMode>,

    i2c: I,
}

impl<I> I2C<I, init_state::Disabled, init_state::Disabled, init_state::Disabled>
where
    I: Instance,
{
    pub(crate) fn new(i2c: I) -> Self {
        I2C {
            master: Master::new(),
            slave: Slave::new(),

            i2c: i2c,
        }
    }

    /// Enable this I2C instance
    ///
    /// This method is only available, if `I2C` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `I2C` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable<SdaPin, SclPin>(
        mut self,
        _: swm::Function<I::Scl, swm::state::Assigned<SclPin>>,
        _: swm::Function<I::Sda, swm::state::Assigned<SdaPin>>,
        syscon: &mut syscon::Handle,
    ) -> I2C<I, init_state::Enabled, init_state::Disabled, init_state::Disabled>
    {
        syscon.enable_clock(&mut self.i2c);

        I2C {
            master: Master::new(),
            slave: Slave::new(),

            i2c: self.i2c,
        }
    }
}

impl<I, SlaveMode> I2C<I, init_state::Enabled, init_state::Disabled, SlaveMode>
where
    I: Instance,
{
    /// Enable master mode
    ///
    /// This method is only available, if the I2C instance is enabled, but
    /// master mode is disabled. Code that attempts to call this method when
    /// this is not the case will not compile.
    ///
    /// Consumes this instance of `Master` and returns another instance that has
    /// its type state updated.
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
    pub fn enable_master_mode<C>(
        self,
        clock: &Clock<C>,
        syscon: &mut syscon::Handle,
    ) -> I2C<I, init_state::Enabled, init_state::Enabled, SlaveMode>
    where
        C: ClockSource,
    {
        C::select(&self.i2c, syscon);

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
        self.i2c.cfg.modify(|_, w| w.msten().enabled());

        I2C {
            master: Master::new(),
            slave: Slave::new(),

            i2c: self.i2c,
        }
    }
}

impl<I, MasterMode>
    I2C<I, init_state::Enabled, MasterMode, init_state::Disabled>
where
    I: Instance,
{
    /// Enable slave mode
    ///
    /// This method is only available, if the peripheral instance is enabled and
    /// slave mode is disabled. Code that attempts to call this method when this
    /// is not the case will not compile.
    ///
    /// Consumes this instance of `I2C` and returns another instance that has
    /// its type state updated.
    pub fn enable_slave_mode(
        self,
        address: u8,
    ) -> I2C<I, init_state::Enabled, MasterMode, init_state::Enabled> {
        // Enable slave mode
        // Set all other configuration values to default.
        self.i2c.cfg.modify(|_, w| w.slven().enabled());

        // Set provided address
        self.i2c.slvadr[0].write(|w| {
            w.sadisable().enabled();

            // Sound, as all possible 7-bit values are acceptable here.
            unsafe { w.slvadr().bits(address >> 1) }
        });

        I2C {
            master: Master::new(),
            slave: Slave::new(),

            i2c: self.i2c,
        }
    }
}

impl<I, MasterMode, SlaveMode>
    I2C<I, init_state::Enabled, MasterMode, SlaveMode>
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
        Error::read::<I>()
    }
}

impl<I, State, MasterMode, SlaveMode> I2C<I, State, MasterMode, SlaveMode>
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
