//! API for USART
//!
//! The entry point to this API is [`USART`]. Currently, only some limited UART
//! functionality is implemented.
//!
//! The USART peripheral is described in the user manual, chapter 13.
//!
//! # Examples
//!
//! ``` no_run
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::Peripherals;
//! use lpc82x_hal::usart::{
//!     BaudRate,
//!     USART,
//! };
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut syscon = p.SYSCON.split();
//! let mut swm    = p.SWM.split();
//!
//! // Set baud rate to 115200 baud
//! // Please refer to the USART example in the repository for a full
//! // explanation of this value.
//! syscon.uartfrg.set_clkdiv(6);
//! syscon.uartfrg.set_frgmult(22);
//! syscon.uartfrg.set_frgdiv(0xff);
//! let baud_rate = BaudRate::new(&syscon.uartfrg, 0);
//!
//! let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
//!     swm.pins.pio0_0.into_swm_pin(),
//!     &mut swm.handle,
//! );
//! let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
//!     swm.pins.pio0_4.into_swm_pin(),
//!     &mut swm.handle,
//! );
//!
//! // Initialize USART0. This should never fail, as the only reason `init`
//! // returns a `Result::Err` is when the transmitter is busy, which it
//! // shouldn't be right now.
//! let mut serial = p.USART0.enable(
//!     &baud_rate,
//!     &mut syscon.handle,
//!     u0_rxd,
//!     u0_txd,
//! );
//!
//! // Use a blocking method to write a string
//! serial.tx().bwrite_all(b"Hello, world!");
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/examples

mod clock;
mod instances;
mod peripheral;
mod rx;
mod tx;

pub use self::{
    clock::Clock,
    instances::Instance,
    peripheral::USART,
    rx::{Error, Rx},
    tx::Tx,
};
