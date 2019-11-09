# LPC8xx HAL [![crates.io](https://img.shields.io/crates/v/lpc8xx-hal.svg)](https://crates.io/crates/lpc8xx-hal) [![Documentation](https://docs.rs/lpc8xx-hal/badge.svg)](https://docs.rs/lpc8xx-hal) [![Build Status](https://travis-ci.com/lpc-rs/lpc8xx-hal.svg?branch=master)](https://travis-ci.com/lpc-rs/lpc8xx-hal)

## Introduction

Hardware Abstraction Layer (HAL) for the [NXP LPC800] series of microcontrollers, written in the [Rust] programming language. The [LPC82x] and [LPC845] are currently supported. LPC8xx HAL provides a high-level interface to the features of LPC800 MCUs, that is safe, convenient, and efficient.

LPC8xx HAL leverages Rust's type system to prevent common mistakes. Things like attempting to use a peripheral that has not been properly initialized, or attempting to assign conflicting functions to the same pin, will all result in compile-time errors.

This crate is an implementation of [embedded-hal]. Please consider if you can make your code platform-independent, by depending on [embedded-hal] instead of this library.

[NXP LPC800]: https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-:MC_71785
[LPC82x]: https://www.nxp.com/products/processors-and-microcontrollers/arm-based-processors-and-mcus/lpc-cortex-m-mcus/lpc800-series-cortex-m0-plus-mcus/low-cost-microcontrollers-mcus-based-on-arm-cortex-m0-plus-cores:LPC82X
[LPC845]: https://www.nxp.com/products/processors-and-microcontrollers/arm-based-processors-and-mcus/lpc-cortex-m-mcus/lpc800-series-cortex-m0-plus-mcus/low-cost-microcontrollers-mcus-based-on-arm-cortex-m0-plus-cores:LPC84x
[Rust]: https://www.rust-lang.org/
[embedded-hal]: https://crates.io/crates/embedded-hal


## Status

LPC82x HAL is still under heavy development. It is lacking APIs for many peripherals, and the APIs that already exist are mostly incomplete.

**Do you need a feature that is currently missing? Please [open an issue]!**

The existing APIs are expected to evolve significantly in the future. API stability is *not* guaranteed, which means future versions might not be compatible with code using the current version.


## Usage

To include LPC8xx HAL in you project, add the following to your `Cargo.toml`:

``` toml
[dependencies]
lpc8xx-hal = "0.6"
```

If you want to use LPC8xx HAL in an application (as opposed to a library), there are additional things that need to be set up. Please refer to the [API Reference] for details.


## Documentation

The **[API Reference]** should contain everything you need to use this library. If you think that anything's missing, please [open an issue].

For functionality that is not yet covered by this crate, you may need to fall back to [`lpc82x-pac`]/[`lpc845-pac`]. Please refer to their respective documentation if necessary.

The authoritative source on the supported MCUs are their respective user manuals, available from NXP.

[lpc82x-pac]: https://crates.io/crates/lpc82x-pac
[lpc845-pac]: https://crates.io/crates/lpc84x-pac


## Help Wanted

Are you familiar with the LPC8xx family? We need your help, even if you are not using LPC8xx HAL. Some design issues require feedback from people familiar with the hardware and how it is used. Check out the [help wanted] tag on the issue tracker.

Do you want to contribute to LPC8xx HAL? There's a number of [good first issues] on the issue tracker. If you're unsure about anything, check out our documentation on [how to contribute], or just ask!

[help wanted]: https://github.com/lpc-rs/lpc8xx-hal/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22
[good first issues]: https://github.com/lpc-rs/lpc8xx-hal/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[how to contribute]: https://github.com/lpc-rs/lpc8xx-hal/blob/master/CONTRIBUTING.md


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/lpc-rs/lpc8xx-hal/blob/master/LICENSE


**Supported by [Braun Embedded](https://braun-embedded.com/)**


[open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues/new
