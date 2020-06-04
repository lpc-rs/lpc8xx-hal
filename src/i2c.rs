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
//! use lpc8xx_hal::{
//!     prelude::*,
//!     Peripherals,
//!     i2c,
//! };
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut swm    = p.SWM.split();
//! let mut syscon = p.SYSCON.split();
//!
//! #[cfg(feature = "82x")]
//! let mut swm_handle = swm.handle;
//! #[cfg(feature = "845")]
//! let mut swm_handle = swm.handle.enable(&mut syscon.handle);
//!
//! let (i2c0_sda, _) = swm.fixed_functions.i2c0_sda.assign(
//!     p.pins.pio0_11.into_swm_pin(),
//!     &mut swm_handle,
//! );
//! let (i2c0_scl, _) = swm.fixed_functions.i2c0_scl.assign(
//!     p.pins.pio0_10.into_swm_pin(),
//!     &mut swm_handle,
//! );
//!
//! let mut i2c = p.I2C0.enable_master(
//!     &i2c::Clock::new_400khz(),
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

mod clock;
mod instances;
mod interrupts;
mod peripheral;

pub use self::{
    clock::{Clock, ClockSource},
    instances::Instance,
    interrupts::Interrupts,
    peripheral::{Error, Master, Slave, I2C},
};
