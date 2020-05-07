//! I2C example using the VL53L0X distance sensor by ST
//!
//! This example expects the microcontroller to be connected to a VL53L0X
//! satellite board in the following way:
//! - PIO0_11/I2C0_SDA to SDA_I
//! - PIO0_10/I2C0_SCL to SCL_I
//! - VSS to GND
//! - VDD to VDD
//! - VDD to XSDN_I, via a 10 kOhm pull-up resistor

#![no_main]
#![no_std]

extern crate panic_halt;

use core::fmt::Write;

use lpc8xx_hal::{cortex_m_rt::entry, i2c, prelude::*, usart, Peripherals};

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let i2c = p.I2C0;
    let mut swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    // Set baud rate to 115200 baud
    //
    // The common peripheral clock for all UART units, U_PCLK, needs to be set
    // to 16 times the desired baud rate. This results in a frequency of
    // 1843200 Hz for U_PLCK.
    //
    // We assume the main clock runs at 12 Mhz. To get close to the desired
    // frequency for U_PLCK, we divide that by 6 using UARTCLKDIV, resulting in
    // a frequency of 2 Mhz.
    //
    // To get to the desired 1843200 Hz, we need to further divide the frequency
    // using the fractional baud rate generator. The fractional baud rate
    // generator divides the frequency by `1 + MULT/DIV`.
    //
    // DIV must always be 256. To achieve this, we need to set the UARTFRGDIV to
    // 0xff. MULT can then be fine-tuned to get as close as possible to the
    // desired value. We choose the value 22, which we write into UARTFRGMULT.
    //
    // Finally, we can set an additional divider value for the UART unit by
    // writing to the BRG register. As we are already close enough to the
    // desired value, we write 0, resulting in no further division.
    //
    // All of this is somewhat explained in the user manual, section 13.3.1.
    syscon.uartfrg.set_clkdiv(6);
    syscon.uartfrg.set_frgmult(22);
    syscon.uartfrg.set_frgdiv(0xff);

    let (u0_rxd, _) = swm
        .movable_functions
        .u0_rxd
        .assign(p.pins.pio0_0.into_swm_pin(), &mut swm.handle);
    let (u0_txd, _) = swm
        .movable_functions
        .u0_txd
        .assign(p.pins.pio0_4.into_swm_pin(), &mut swm.handle);

    let mut serial = p.USART0.enable(
        &usart::Clock::new(&syscon.uartfrg, 0, 16),
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
    );

    serial
        .bwrite_all(b"Initializing I2C...\n")
        .expect("Write should never fail");

    let (i2c0_sda, _) = swm
        .fixed_functions
        .i2c0_sda
        .assign(p.pins.pio0_11.into_swm_pin(), &mut swm.handle);
    let (i2c0_scl, _) = swm
        .fixed_functions
        .i2c0_scl
        .assign(p.pins.pio0_10.into_swm_pin(), &mut swm.handle);

    let i2c_clock = i2c::Clock::new_400khz();
    let mut i2c =
        i2c.enable(&i2c_clock, &mut syscon.handle, i2c0_sda, i2c0_scl);

    serial
        .bwrite_all(b"Writing data...\n")
        .expect("Write should never fail");

    // Write index of reference register
    i2c.write(0x52, &[0xC0]).expect("Failed to write data");

    serial
        .bwrite_all(b"Receiving data...\n")
        .expect("Write should never fail");

    // Read value from reference register
    let mut buffer = [0u8; 1];
    i2c.read(0x52, &mut buffer).expect("Failed to read data");

    write!(serial, "{:#X}\n", buffer[0]).expect("Write should never fail");

    if buffer[0] == 0xEE {
        serial
            .bwrite_all(b"SUCCESS!\n")
            .expect("Write should never fail");
    } else {
        serial
            .bwrite_all(b"FAILURE!\n")
            .expect("Write should never fail");
    }

    loop {}
}
