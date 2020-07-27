//! API for Direct Memory Access (DMA)
//!
//! The DMA controller is described in the user manual, chapter 12.

mod buffer;
mod descriptors;
mod gen;
mod peripheral;
mod transfer;

pub mod channels;

pub use self::{
    channels::Channel,
    gen::*,
    peripheral::DMA,
    transfer::{Dest, Payload, Source, Transfer},
};
