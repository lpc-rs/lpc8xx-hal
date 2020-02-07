//! I2C example using an 256 byte eeprom
//!
//! This example expects the microcontroller to be connected to the eeprom in
//! the following way:
//! - PIO0_11/I2C0_SDA to SDA
//! - PIO0_10/I2C0_SCL to SCL
//! - VSS to GND
//! - VDD to VDD

#![no_main]
#![no_std]

extern crate panic_halt;

use core::fmt::Write;

use lpc8xx_hal::{
    cortex_m_rt::entry,
    delay::Delay,
    prelude::*,
    syscon::clocksource::{I2cClock, UsartClock},
    CorePeripherals, Peripherals,
};

#[entry]
fn main() -> ! {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut delay = Delay::new(cp.SYST);
    let i2c = p.I2C0;
    let swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    #[cfg(feature = "82x")]
    let mut handle = swm.handle;
    #[cfg(feature = "845")]
    let mut handle = swm.handle.enable(&mut syscon.handle); // SWM isn't enabled by default on LPC845.

    #[cfg(feature = "82x")]
    // Set baud rate to 115200 baud
    //
    // See the usart example for a detailed explanation on how the usart setup works
    let clock_config = {
        syscon.uartfrg.set_clkdiv(6);
        syscon.uartfrg.set_frgmult(22);
        syscon.uartfrg.set_frgdiv(0xff);
        UsartClock::new(&syscon.uartfrg, 0, 16)
    };
    #[cfg(feature = "845")]
    // Set baud rate to 115200 baud
    let clock_config = UsartClock::new_with_baudrate(115200);
    #[cfg(feature = "82x")]
    let tx_pin = swm.pins.pio0_7.into_swm_pin();
    #[cfg(feature = "82x")]
    let rx_pin = swm.pins.pio0_18.into_swm_pin();
    #[cfg(feature = "845")]
    let tx_pin = swm.pins.pio0_25.into_swm_pin();
    #[cfg(feature = "845")]
    let rx_pin = swm.pins.pio0_24.into_swm_pin();

    let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(rx_pin, &mut handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd.assign(tx_pin, &mut handle);

    let serial =
        p.USART0
            .enable(&clock_config, &mut syscon.handle, u0_rxd, u0_txd);

    serial
        .tx()
        .bwrite_all(b"Initializing I2C...\n")
        .expect("Write should never fail");

    let (i2c0_sda, _) = swm
        .fixed_functions
        .i2c0_sda
        .assign(swm.pins.pio0_11.into_swm_pin(), &mut handle);
    let (i2c0_scl, _) = swm
        .fixed_functions
        .i2c0_scl
        .assign(swm.pins.pio0_10.into_swm_pin(), &mut handle);

    let i2c_clock = I2cClock::new_400khz();
    let mut i2c =
        i2c.enable(&i2c_clock, &mut syscon.handle, i2c0_sda, i2c0_scl);

    // Address of the eeprom
    // ADJUST THIS
    let address = 0b1010_0000;

    serial
        .tx()
        .bwrite_all(b"Writing data...\n")
        .expect("Write should never fail");

    // Write an 'Hi' to address 0 & 1
    i2c.write(address, &[0, b'H', b'i'])
        .expect("Failed to write data");

    serial
        .tx()
        .bwrite_all(b"Reading data...\n")
        .expect("Write should never fail");

    // Wait a bit until the write has gone through
    delay.delay_ms(1_000_u16);

    // Read value from the eeprom
    let mut buffer = [0u8; 2];
    // Set the address to 0 again
    i2c.write(address, &[0]).expect("Failed to write data");
    // Read the two bytes at 0 & 1
    i2c.read(address, &mut buffer).expect("Failed to read data");

    write!(serial.tx(), "{:?}\n", &buffer).expect("Write should never fail");

    // Check if they're correct
    if buffer == *b"Hi" {
        serial
            .tx()
            .bwrite_all(b"SUCCESS!\n")
            .expect("Write should never fail");
    } else {
        serial
            .tx()
            .bwrite_all(b"FAILURE!\n")
            .expect("Write should never fail");
    }

    loop {}
}
