#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

cargo build --verbose --features=rt,82x --examples
cargo build --verbose --features=rt,845 --examples
