//! API for Direct Memory Access (DMA)
//!
//! The DMA controller is described in the user manual, chapter 12.

mod buffer;
mod descriptors;
mod gen;
mod peripheral;

pub mod channels;
pub mod transfer;

pub use self::{
    channels::Channel,
    gen::*,
    peripheral::DMA,
    transfer::{Dest, Payload, Source, Transfer},
};
