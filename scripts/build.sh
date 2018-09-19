#!/usr/bin/env bash

cargo test --verbose --features="compiletest" &&
cargo build --verbose --features="rt" --examples --target=thumbv6m-none-eabi --release
