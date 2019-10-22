#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

cargo build --verbose --no-default-features --features=rt,82x --examples
cargo build --verbose --no-default-features --features=rt_845,845 --examples
