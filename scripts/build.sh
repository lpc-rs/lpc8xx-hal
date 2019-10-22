#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

cargo build --verbose --no-default-features --features=82x-rt --examples
cargo build --verbose --no-default-features --features=845-rt --examples
