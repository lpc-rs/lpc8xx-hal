#!/usr/bin/env bash

# Need to clean to work around this issue:
# https://github.com/braun-robotics/rust-lpc82x-hal/issues/105
cargo clean &&
cargo test --verbose --features="compiletest" &&
cargo build --verbose --features="rt" --examples --target=thumbv6m-none-eabi --release
