# LPC82x HAL

## Introduction

Hardware Abstraction Layer (HAL) for [NXP LPC82x] microcontrollers, written in the [Rust] programming language. LPC82x HAL aims to provide a high-level interface to the features of LPC82x MPUs that is safe and efficient. It also tries to be convenient where possible, but this is a secondary goal compared to safety and efficiency.

LPC82x HAL uses Rust's type system to prevent many mistakes at compile-time. For example, it won't let you use a peripheral that isn't initialized yet, or it prevents you from measuring time using a clock that's currently disabled. The goal is to provide an API that is impossible to use incorrectly.

[NXP LPC82x]: https://www.nxp.com/products/processors-and-microcontrollers/arm-based-processors-and-mcus/lpc-cortex-m-mcus/lpc800-series-cortex-m0-plus-mcus/low-cost-microcontrollers-mcus-based-on-arm-cortex-m0-plus-cores:LPC82X
[Rust]: https://www.rust-lang.org/


## Status

This project is in early development. It is incomplete, as many LPC82x features are not covered, and the existing APIs are mostly provisional. The current plan is to focus on improving the existing APIs, and add new features slowly, as use cases become available.

Please [open an issue], if you need any features that are missing from this library, or better yet, [submit a pull request] with your enhancements.

Existing APIs are expected to evolve significantly in the future. API stability is *not* guaranteed, which means future versions might not be compatible with code using the current version.

This crate currently requires a nightly version of the Rust toolchain. If you installed Rust via [rustup], you can switch to the nightly version with `rustup default nightly`.

[open an issue]: https://github.com/braun-robotics/rust-lpc82x-hal/issues/new
[submit a pull request]: https://github.com/braun-robotics/rust-lpc82x-hal/blob/master/CONTRIBUTING.md
[rustup]: https://rustup.rs/


## Usage

To include LPC82x HAL in you project via Cargo, add the following to your `Cargo.toml`:

``` toml
[dependencies]
lpc82x-hal = { git = "https://github.com/braun-robotics/rust-lpc82x-hal.git" }
```

If you want to use LPC82x in your application (as opposed to a library), there are additional things that need to be set up. Please refer to the [API Reference] for details.

This crate is an implementation of [embedded-hal]. If you're writing a library, please consider whether you can make your it platform-independent by only depending on [embedded-hal] instead.

[embedded-hal]: https://github.com/japaric/embedded-hal


## Documentation

The **[API Reference]** is the main source of documentation for this crate. For functionality that is not yet covered by this crate, you may need to fall back to [rust-lpc82x]. Please refer to the [rust-lpc82x documentation], should that be necessary.

The authoritative source on LPC82x is the **[LPC82x User Manual]**.

[rust-lpc82x]: https://crates.io/crates/lpc82x
[rust-lpc82x documentation]: https://docs.rs/lpc82x/
[LPC82x User Manual]: https://www.nxp.com/docs/en/user-guide/UM10800.pdf


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-robotics/rust-lpc82x-hal/blob/master/LICENSE


**Supported by [Braun Robotics](https://braun-robotics.com/)**


[API Reference]: https://braun-robotics.github.io/rust-lpc82x-hal/lpc82x_hal/index.html
