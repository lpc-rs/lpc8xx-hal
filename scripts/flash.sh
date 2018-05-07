#!/usr/bin/env bash

# Script for building an example and uploading it to an LPC82x microcontroller.
# It assumes that a version of lpc21isp that supports LPC82x is installed. This
# fork is known to work: https://github.com/hannobraun/lpc21isp
#
# Example:
# ./scripts/flash.sh blinker /dev/ttyUSB0

EXAMPLE=$1
DEVICE=$2

if [ $# -ne 2 ]
then
    echo "Usage: $0 EXAMPLE DEVICE"
    exit 1
fi

TARGET_DIR=target/thumbv6m-none-eabi/release/examples
ELF_FILE=$TARGET_DIR/$EXAMPLE
BIN_FILE=$TARGET_DIR/$EXAMPLE.bin

cargo build \
    --target thumbv6m-none-eabi \
    --release \
    --example $EXAMPLE \
    --features="rt" &&

arm-none-eabi-objcopy \
    -O binary \
    $ELF_FILE \
    $BIN_FILE &&

lpc21isp -bin -verify -term $BIN_FILE $DEVICE 115200 0
