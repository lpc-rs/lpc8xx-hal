#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"
#
# Check for formatting with the stable rustfmt
if [ "$TRAVIS_RUST_VERSION" != beta ] &&  [ "$TRAVIS_RUST_VERSION" != nightly ]; then
    cargo fmt -- --check
fi

cargo build --verbose --features=82x-rt,no-target-warning --examples
cargo build --verbose --features=845-rt,no-target-warning --examples
