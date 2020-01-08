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
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/lpc82x-hal/examples

use embedded_hal::blocking::i2c;
use void::Void;

use crate::{
    init_state, pac,
    swm::{self, I2C0_SCL, I2C0_SDA, PIO0_10, PIO0_11},
    syscon,
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
pub struct I2C<State = init_state::Enabled> {
    i2c: pac::I2C0,
    _state: State,
}

impl I2C<init_state::Disabled> {
    pub(crate) fn new(i2c: pac::I2C0) -> Self {
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
    /// This method expects the I2C mode for PIO0_10 and PIO0_11 to be set to
    /// standard/fast mode. This is the default value.
    ///
    /// The I2C clock frequency is hardcoded to a specific value. For unknown
    /// reasons, this seems to be 79.6 kHz.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(
        mut self,
        syscon: &mut syscon::Handle,
        _: swm::Function<I2C0_SDA, swm::state::Assigned<PIO0_11>>,
        _: swm::Function<I2C0_SCL, swm::state::Assigned<PIO0_10>>,
    ) -> I2C<init_state::Enabled> {
        syscon.enable_clock(&mut self.i2c);

        // We need the I2C mode for the pins set to standard/fast mode,
        // according to the user manual, section 15.3.1. This is already the
        // default value (see user manual, sections 8.5.8 and 8.5.9).

        // Set I2C clock frequency
        // Here's my thinking: The main clock runs at 12 Mhz by default. The
        // minimum low and high times of SCL are set to 2 clock cyles each (see
        // below), meaning a full SCL cycle should take 4 clock ticks. Therefore
        // dividing the main clock by 8 (which is achieved by writing 7 below),
        // should result in an I2C frequency near 400 kHz (375 kHz to be
        // precise).
        // None of that is correct, of course. When actually running, I'm
        // measuring an SCL frequency of 79.6 kHz. I wish I knew why.
        self.i2c.clkdiv.write(|w| unsafe { w.divval().bits(7) });

        // SCL low and high times are left at their default values (two clock
        // cycles each). See user manual, section 15.6.9.

        // Enable master mode
        // Set all other configuration values to default.
        self.i2c.cfg.write(|w| w.msten().enabled());

        I2C {
            i2c: self.i2c,
            _state: init_state::Enabled(()),
        }
    }
}

impl i2c::Write for I2C<init_state::Enabled> {
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
            while !self.i2c.stat.read().mststate().is_transmit_ready() {}

            // Write byte
            self.i2c.mstdat.write(|w| unsafe { w.data().bits(b) });

            // Continue transmission
            self.i2c.mstctl.write(|w| w.mstcontinue().continue_());
        }

        // Wait until peripheral is ready to transmit
        while !self.i2c.stat.read().mststate().is_transmit_ready() {}

        // Stop transmission
        self.i2c.mstctl.modify(|_, w| w.mststop().stop());

        Ok(())
    }
}

impl i2c::Read for I2C<init_state::Enabled> {
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
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        // Wait until peripheral is idle
        while !self.i2c.stat.read().mststate().is_idle() {}

        // Write slave address with rw bit set to 1
        self.i2c
            .mstdat
            .write(|w| unsafe { w.data().bits(address | 0x01) });

        // Start transmission
        self.i2c.mstctl.write(|w| w.mststart().start());

        for b in buffer {
            // Wait until peripheral is ready to receive
            while !self.i2c.stat.read().mststate().is_receive_ready() {}

            // Read received byte
            *b = self.i2c.mstdat.read().data().bits();
        }

        Ok(())
    }
}

impl<State> I2C<State> {
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
    pub fn free(self) -> pac::I2C0 {
        self.i2c
    }
}
