//! API for the SPI peripherals
//!
//! # Example
//!
//! ``` no_run
//! use lpc8xx_hal::{
//!     prelude::*,
//!     Peripherals,
//!     spi,
//! };
//!
//! let mut p  = Peripherals::take().unwrap();
//! let mut swm = p.SWM.split();
//! let mut syscon = p.SYSCON.split();
//!
//! #[cfg(feature = "82x")]
//! let mut swm_handle = swm.handle;
//! #[cfg(feature = "845")]
//! let mut swm_handle = swm.handle.enable(&mut syscon.handle);
//!
//! let (spi0_sck, _) = swm.movable_functions.spi0_sck.assign(
//!     p.pins.pio0_13.into_swm_pin(),
//!     &mut swm_handle,
//! );
//! let (spi0_mosi, _) = swm
//!     .movable_functions
//!     .spi0_mosi
//!     .assign(p.pins.pio0_14.into_swm_pin(), &mut swm_handle);
//! let (spi0_miso, _) = swm
//!     .movable_functions
//!     .spi0_miso
//!     .assign(p.pins.pio0_15.into_swm_pin(), &mut swm_handle);
//!
//! #[cfg(feature = "82x")]
//! let spi_clock = spi::Clock::new(&(), 0);
//! #[cfg(feature = "845")]
//! let spi_clock = spi::Clock::new(&syscon.iosc, 0);
//!
//! // Enable SPI0
//! let mut spi = p.SPI0.enable_as_master(
//!     &spi_clock,
//!     &mut syscon.handle,
//!     embedded_hal::spi::MODE_0,
//!     spi0_sck,
//!     spi0_mosi,
//!     spi0_miso,
//! );
//!
//! let mut tx_data = [0x00, 0x01];
//! let rx_data = spi.transfer(&mut tx_data)
//!     .expect("Transfer shouldn't fail");
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
    instances::{Instance, SlaveSelect},
    interrupts::Interrupts,
    peripheral::{Master, Slave, SPI},
};

pub use crate::embedded_hal::spi::{
    Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3,
};
