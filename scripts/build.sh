#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

# `.cargo/config` defaults us to the microcontroller's target triple. We need
# to override this here, to run `cargo test`. You may need to adapt this,
# depending on you platform.
TARGET=x86_64-unknown-linux-gnu

# Need to clean to work around this issue:
# https://github.com/lpc-rs/lpc8xx-hal/issues/105
cargo test --manifest-path lpc82x-hal/Cargo.toml --verbose --target=$TARGET
# We need to add the linker file here, since `.cargo/config` gets overwritten by `RUSTFLAGS`
RUSTFLAGS="${RUSTFLAGS} -C link-arg=-Tlink.x" cargo build --manifest-path lpc82x-hal/Cargo.toml --verbose --features="rt" --examples --release
cargo test --manifest-path lpc845-hal/Cargo.toml --verbose --target=$TARGET
RUSTFLAGS="${RUSTFLAGS} -C link-arg=-Tlink.x" cargo build --manifest-path lpc845-hal/Cargo.toml --verbose --features="rt" --examples --release
