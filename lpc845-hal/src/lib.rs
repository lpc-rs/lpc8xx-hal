//! # LPC845 Hardware Abstraction Layer

#![deny(missing_docs)]
#![no_std]

#[cfg(test)]
extern crate std;

extern crate nb;

extern crate cortex_m;
extern crate embedded_hal;
extern crate void;

pub extern crate lpc845_pac as raw;
