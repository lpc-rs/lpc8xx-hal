# LPC845 HAL [![crates.io](https://img.shields.io/crates/v/lpc845-hal.svg)](https://crates.io/crates/lpc845-hal) [![Documentation](https://docs.rs/lpc845-hal/badge.svg)](https://docs.rs/lpc845-hal) [![Build Status](https://travis-ci.com/lpc-rs/lpc82x-hal.svg?branch=master)](https://travis-ci.com/lpc-rs/lpc82x-hal)

## Introduction

Hardware Abstraction Layer (HAL) for [NXP LPC845] microcontrollers, written in the [Rust] programming language. LPC845 HAL provides a high-level interface to the features of LPC845 MCUs, that is safe, convenient, and efficient.

LPC845 HAL leverages Rust's type system to prevent common mistakes. Things like attempting to use a peripheral that has not been properly initialized, or attempting to assign conflicting functions to the same pin, will all result in compile-time errors.

This crate is an implementation of [embedded-hal]. Please consider if you can make your code platform-independent, by depending on [embedded-hal] instead of this library.

[NXP LPC845]: https://www.nxp.com/products/processors-and-microcontrollers/arm-based-processors-and-mcus/lpc-cortex-m-mcus/lpc800-series-cortex-m0-plus-mcus/low-cost-microcontrollers-mcus-based-on-arm-cortex-m0-plus-cores:LPC84x
[Rust]: https://www.rust-lang.org/
[embedded-hal]: https://crates.io/crates/embedded-hal


## Status

LPC845 HAL is very much a WIP, not ready to be used by anyone

