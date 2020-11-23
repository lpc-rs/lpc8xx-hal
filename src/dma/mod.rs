//! API for Direct Memory Access (DMA)
//!
//! The entry point to this API is the [`DMA`] struct.
//!
//! The DMA peripheral is described in the following user manuals:
//! - LPC82x user manual, chapter 12
//! - LPC84x user manual, chapter 16
//!
//! [`DMA`]: struct.DMA.html

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

pub(crate) use self::buffer::Buffer;
