#!/usr/bin/env bash

# `.cargo/config` defaults us to the microcontroller's target triple. We need
# to override this here, to run `cargo test`. You may need to adapt this,
# depending on you platform.
TARGET=x86_64-unknown-linux-gnu

# Need to clean to work around this issue:
# https://github.com/braun-robotics/rust-lpc82x-hal/issues/105
cargo clean &&
cargo test --verbose --features="compiletest" --target=$TARGET &&
cargo build --verbose --features="rt" --examples --release
