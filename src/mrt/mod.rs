//! API for the MRT (Multi-Rate Timer) peripheral
//!
//! Please be aware that this doesn't try to abstract everything, it only
//! implements the embedded-hal `Timer` functionality.
//!
//! The MRT consists of 4 channels, which are mostly separate and can each act
//! as a run-of-the-mill timer.

mod channel;
mod gen;
mod peripheral;
mod ticks;

pub use self::{
    channel::Channel,
    gen::*,
    peripheral::MRT,
    ticks::{TickConversionError, Ticks},
};

/// The maximum timer value
pub const MAX_VALUE: Ticks = Ticks(0x7fff_ffff - 1);
