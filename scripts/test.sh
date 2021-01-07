#!/usr/bin/env bash
set -e

# Run unit tests (from the test-suite/ directory) directly on the device
#
# If called without argments, unit tests for all targets are run. This requires
# all required dev boards to be connected.
#
# If only one target should be tested, it must be passed as an argument:
# - ./scripts/test.sh 82x
# - ./scripts/test.sh 845

# The user can optionally pass the target to run the tests for.
TARGET=$1

function test() {
    TARGET=$1

    echo ""
    echo "### Testing target $TARGET"
    echo ""

    # Select probe-run configuration
    [ "$TARGET" == "82x" ] && \
        export PROBE_RUN_CHIP=LPC824M201JHI33 && \
        export PROBE_RUN_PROBE=0d28:0204
    [ "$TARGET" == "845" ] && \
        export PROBE_RUN_CHIP=LPC845M301JHI48 && \
        export PROBE_RUN_PROBE=1fc9:0132

    # Set linker configuration. We have to do this from here, using an
    # environment variable, as otherwise the different Cargo configuration files
    # will interact in weird ways. Setting the rustc flags in an environment
    # variable overrides all `rustflags` keys in Cargo configuration, so we have
    # full control here.
    export RUSTFLAGS="\
        -C linker=flip-link \
        -C link-arg=-Tlink.x \
        -C link-arg=-Tdefmt.x
    "

    (
        cd test-suite &&
        cargo test -p tests --features="$TARGET")
}

if [ -n "$TARGET" ]; then
    test $TARGET
else
    test "82x"
    test "845"
fi
