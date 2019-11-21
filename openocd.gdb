target remote | openocd -f target/openocd.cfg
load
# Required to work around a weird OpenOCD error message when uploading to the
# LPC845-BRK.
monitor reset
continue
