//! Interface to the pin interrupts/pattern matching engine
//!
//! This API is currently limited. It exposes a subset of the pin interrupts
//! functionality, and none of the pattern matching functionality.

mod gen;
mod interrupt;
mod peripheral;
mod traits;

pub use self::{gen::*, interrupt::Interrupt, peripheral::PININT};
