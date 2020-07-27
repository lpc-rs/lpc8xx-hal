//! API for Direct Memory Access (DMA)
//!
//! The DMA controller is described in the user manual, chapter 12.

mod buffer;
mod channels;
mod descriptors;
mod peripheral;
mod transfer;

pub use self::{
    channels::*,
    peripheral::{Handle, DMA},
    transfer::{Dest, Payload, Source, Transfer},
};
