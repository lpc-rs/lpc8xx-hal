//! API to control pins
//!
//! The most important part of this API is [`Pin`]. Please refer to its
//! documentation, to learn how to use this module.
//!
//! [`Pin`]: struct.Pin.html

mod gen;
mod pin;
mod traits;

pub mod state;

pub use self::{
    gen::*, pin::DynamicPinDirection, pin::Pin, state::State, traits::Trait,
};
