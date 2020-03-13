//! API for the I2C peripherals
//!
//! Please be aware that this is a very basic implementation, with lots of
//! important things missing. Please be careful when using this API.
//!
//! The I2C peripherals are described in the user manual, chapter 15.
//!
//! # Examples
//!
//! Write data to an I2C slave:
//!
//! ``` no_run
//! # let address = 0x0;
//! # let data    = [0; 8];
//! #
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::Peripherals;
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut swm    = p.SWM.split();
//! let mut syscon = p.SYSCON.split();
//!
//! let (i2c0_sda, _) = swm.fixed_functions.i2c0_sda.assign(
//!     swm.pins.pio0_11.into_swm_pin(),
//!     &mut swm.handle,
//! );
//! let (i2c0_scl, _) = swm.fixed_functions.i2c0_scl.assign(
//!     swm.pins.pio0_10.into_swm_pin(),
//!     &mut swm.handle,
//! );
//!
//! let mut i2c = p.I2C0.enable(
//!     &mut syscon.handle,
//!     i2c0_sda,
//!     i2c0_scl,
//! );
//!
//! i2c.write(address, &data)
//!     .expect("Failed to write data");
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/examples

use core::ops::Deref;
use embedded_hal::blocking::i2c;
use void::Void;

use crate::{
    init_state,
    pac::{self, Interrupt},
    swm::{self},
    syscon::{self, clock_source::I2cClock, PeripheralClock},
};

/// Interface to an I2C peripheral
///
/// Please refer to the [module documentation] for more information.
///
/// # Limitations
///
/// This API has the following limitations:
/// - Only I2C0 is supported.
/// - Only master mode is supported.
/// - Errors are not handled.
///
/// Additional limitations are documented on the specific methods that they
/// apply to.
///
/// [module documentation]: index.html
pub struct I2C<I, State = init_state::Enabled> {
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

    /// Enable the I2C peripheral
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
    /// This method expects the mode for SDA & SCL pins to be set to
    /// standard/fast mode. This is the default value.
    ///
    /// The I2C clock frequency is hardcoded to a specific value. For unknown
    /// reasons, this seems to be 79.6 kHz.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable<SdaPin, SclPin, Clock>(
        mut self,
        clock: &I2cClock<Clock>,
        syscon: &mut syscon::Handle,
        _: swm::Function<I::Sda, swm::state::Assigned<SdaPin>>,
        _: swm::Function<I::Scl, swm::state::Assigned<SclPin>>,
    ) -> I2C<I, init_state::Enabled>
    where
        I2cClock<Clock>: PeripheralClock<I>,
    {
        syscon.enable_clock(&mut self.i2c);

        clock.select_clock(syscon);
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
            _state: init_state::Enabled(()),
        }
    }
}

impl<I> i2c::Write for I2C<I, init_state::Enabled>
where
    I: Instance,
{
    type Error = Void;

    /// Write to the I2C bus
    ///
    /// Please refer to the [embedded-hal documentation] for details.
    ///
    /// # Limitations
    ///
    /// Writing multiple bytes should work, but has not been tested.
    ///
    /// [embedded-hal documentation]: https://docs.rs/embedded-hal/0.2.1/embedded_hal/blocking/i2c/trait.Write.html#tymethod.write
    fn write(&mut self, address: u8, data: &[u8]) -> Result<(), Self::Error> {
        // Wait until peripheral is idle
        while !self.i2c.stat.read().mststate().is_idle() {}

        // Write slave address with rw bit set to 0
        self.i2c
            .mstdat
            .write(|w| unsafe { w.data().bits(address & 0xfe) });

        // Start transmission
        self.i2c.mstctl.write(|w| w.mststart().start());

        for &b in data {
            // Wait until peripheral is ready to transmit
            while self.i2c.stat.read().mstpending().is_in_progress() {}

            // Write byte
            self.i2c.mstdat.write(|w| unsafe { w.data().bits(b) });

            // Continue transmission
            self.i2c.mstctl.write(|w| w.mstcontinue().continue_());
        }

        // Wait until peripheral is ready to transmit
        while self.i2c.stat.read().mstpending().is_in_progress() {}

        // Stop transmission
        self.i2c.mstctl.modify(|_, w| w.mststop().stop());

        Ok(())
    }
}

impl<I> i2c::Read for I2C<I, init_state::Enabled>
where
    I: Instance,
{
    type Error = Void;

    /// Read from the I2C bus
    ///
    /// Please refer to the [embedded-hal documentation] for details.
    ///
    /// # Limitations
    ///
    /// Reading multiple bytes should work, but has not been tested.
    ///
    /// [embedded-hal documentation]: https://docs.rs/embedded-hal/0.2.1/embedded_hal/blocking/i2c/trait.Read.html#tymethod.read
    fn read(
        &mut self,
        address: u8,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        // Wait until peripheral is idle
        while !self.i2c.stat.read().mststate().is_idle() {}

        // Write slave address with rw bit set to 1
        self.i2c
            .mstdat
            .write(|w| unsafe { w.data().bits(address | 0x01) });

        // Start transmission
        self.i2c.mstctl.write(|w| w.mststart().start());

        for b in buffer {
            // Continue transmission
            self.i2c.mstctl.write(|w| w.mstcontinue().continue_());

            // Wait until peripheral is ready to receive
            while self.i2c.stat.read().mstpending().is_in_progress() {}

            // Read received byte
            *b = self.i2c.mstdat.read().data().bits();
        }

        // Stop transmission
        self.i2c.mstctl.modify(|_, w| w.mststop().stop());

        Ok(())
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

/// Internal trait for I2C peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Instance:
    Deref<Target = pac::i2c0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
{
    /// The interrupt that is triggered for this I2C peripheral
    const INTERRUPT: Interrupt;

    /// The movable function that needs to be assigned to this I2C's SDA pin
    type Sda;

    /// The movable function that needs to be assigned to this I2C's SCL pin
    type Scl;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $interrupt:ident,
            $rx:ident,
            $tx:ident;
        )*
    ) => {
        $(
            impl Instance for pac::$instance {
                const INTERRUPT: Interrupt = Interrupt::$interrupt;

                type Sda = swm::$rx;
                type Scl = swm::$tx;
            }
        )*
    };
}

instances!(
    I2C0, I2C0, I2C0_SDA, I2C0_SCL;
    I2C1, I2C1, I2C1_SDA, I2C1_SCL;
    I2C2, I2C2, I2C2_SDA, I2C2_SCL;
    I2C3, I2C3, I2C3_SDA, I2C3_SCL;
);
