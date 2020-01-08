#!/usr/bin/env bash
set -e

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"
#
# Check for formatting with the stable rustfmt
if [ "$TRAVIS_RUST_VERSION" != beta ] &&  [ "$TRAVIS_RUST_VERSION" != nightly ]; then
    cargo fmt -- --check
fi

cargo build --verbose --no-default-features --features=82x-rt --examples
cargo build --verbose --no-default-features --features=845-rt --examples
