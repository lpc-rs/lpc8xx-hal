#!/usr/bin/env bash

cargo build --examples --features="rt" --target=thumbv6m-none-eabi --release
