#!/usr/bin/env bash
set -e

# Target triple of the host machine. Set to x86-64 Linux by default, as that's
# the correct value for the CI server.
#
# Unfotunately there doesn't seem to be a way to tell Cargo to just use the
# target of the host machine, and since we override the default target in
# `.cargo/config` for convenience, this variable is required.
HOST_TARGET=${HOST_TARGET:-x86_64-unknown-linux-gnu}

# Fail build, if there are any warnings.
export RUSTFLAGS="-D warnings"

# Check for formatting with the stable rustfmt
if [ "$TRAVIS_RUST_VERSION" != beta ] && [ "$TRAVIS_RUST_VERSION" != nightly ]; then
    cargo fmt -- --check
fi

function build() {
    cargo test --verbose --features=$1,no-target-warning --target=$HOST_TARGET
    cargo build --verbose --features=$1-rt,no-target-warning --examples
}

build 82x
build 845
