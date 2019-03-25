#![no_std]

#[cfg(feature = "82x")]
pub use lpc82x_pac as target;

#[cfg(feature = "845")]
pub use lpc845_pac as target;
