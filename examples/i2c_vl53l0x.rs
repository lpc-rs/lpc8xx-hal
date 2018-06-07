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


#[macro_use]
extern crate cortex_m_rt;
extern crate lpc82x_hal;
extern crate nb;
extern crate panic_abort;


use core::fmt::Write;

use cortex_m_rt::ExceptionFrame;

use lpc82x_hal::Peripherals;
use lpc82x_hal::prelude::*;
use lpc82x_hal::usart::BaudRate;


entry!(main);

fn main() -> ! {
    let mut p = Peripherals::take().unwrap();

    let mut i2c    = p.i2c0;
    let mut swm    = p.swm.split();
    let mut syscon = p.syscon.split();

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

    let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
        swm.pins.pio0_0.into_swm_pin(),
        &mut swm.handle,
    );
    let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
        swm.pins.pio0_4.into_swm_pin(),
        &mut swm.handle,
    );

    let serial = p.usart0
        .enable(
            &BaudRate::new(&syscon.uartfrg, 0),
            &mut syscon.handle,
            u0_rxd,
            u0_txd,
        );
    let mut serial = match serial {
        Ok(uart) =>
            uart,
        Err(nb::Error::WouldBlock) =>
            // It blocks if the transmitter is busy, and there's no reason for
            // that.
            panic!("USART initialization shouldn't need to block"),

        // This wasn't needed previously, because the compiler correctly
        // recognized that the error type is `!`, which doesn't need to be
        // handled. No idea why that changed.
        Err(nb::Error::Other(_)) =>
            unreachable!(),
    };

    serial.bwrite_all(b"Initializing I2C...\n")
        .expect("Write should never fail");

    swm.fixed_functions.i2c0_sda.assign(
        swm.pins.pio0_11.into_swm_pin(),
        &mut swm.handle,
    );
    swm.fixed_functions.i2c0_scl.assign(
        swm.pins.pio0_10.into_swm_pin(),
        &mut swm.handle,
    );

    syscon.handle.enable_clock(&mut i2c);
    syscon.handle.clear_reset(&mut i2c);

    // We need the I2C mode for the pins set to standard/fast mode, according to
    // the user manual, section 15.3.1. This is already the default value (see
    // user manual, sections 8.5.8 and 8.5.9).

    // Set I2C clock frequency
    // Here's my thinking: The main clock runs at 12 Mhz by default. The minimum
    // low and high times of SCL are set to 2 clock cyles each (see
    // below), meaning a full SCL cycle should take 4 clock ticks. Therefore
    // dividing the main clock by 8 (which is achieved by writing 7 below),
    // should result in an I2C frequency near 400 kHz (375 kHz to be precise).
    // None of that is correct, of course. When actually running, I'm measuring
    // an SCL frequency of 79.6 kHz. I wish I knew why.
    i2c.clkdiv.write(|w| unsafe { w.divval().bits(7) });

    // SCL low and high times are left at their default values (two clock cycles
    // each). See user manual, section 15.6.9.

    // Enable master mode
    // Set all other configuration values to default.
    i2c.cfg.write(|w| w.msten().enabled());

    while !i2c.stat.read().mststate().is_idle() {}

    serial.bwrite_all(b"Write to slave...\n")
        .expect("Write should never fail");

    // Write slave address with rw bit set to 0
    i2c.mstdat.write(|w| unsafe { w.data().bits(0x52) });

    // Start transmission
    i2c.mstctl.write(|w| w.mststart().start());

    while !i2c.stat.read().mststate().is_transmit_ready() {}

    serial.bwrite_all(b"Address ACK'd.\n")
        .expect("Write should never fail");

    // Write index of reference register
    i2c.mstdat.write(|w| unsafe { w.data().bits(0xC0) });

    // Continue transmission
    i2c.mstctl.write(|w| w.mstcontinue().continue_());

    while !i2c.stat.read().mststate().is_transmit_ready() {}

    serial.bwrite_all(b"Data ACK'd.\n")
        .expect("Write should never fail");

    // Stop transmission
    i2c.mstctl.modify(|_, w| w.mststop().stop());

    while !i2c.stat.read().mststate().is_idle() {}

    serial.bwrite_all(b"Receive from slave...\n")
        .expect("Write should never fail");

    // Write slave address with rw bit set to 1
    i2c.mstdat.write(|w| unsafe { w.data().bits(0x53) });

    // Start transmission
    i2c.mstctl.write(|w| w.mststart().start());

    while !i2c.stat.read().mststate().is_receive_ready() {}

    serial.bwrite_all(b"Received data.\n")
        .expect("Write should never fail");

    let data = i2c.mstdat.read().data().bits();

    write!(serial, "{:#X}\n", data)
        .expect("Write should never fail");

    if data == 0xEE {
        serial.bwrite_all(b"SUCCESS!\n")
            .expect("Write should never fail");
    }
    else {
        serial.bwrite_all(b"FAILURE!\n")
            .expect("Write should never fail");
    }

    loop {}
}


exception!(*, default_handler);
exception!(HardFault, handle_hard_fault);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception or interrupt: {}", irqn);
}

fn handle_hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
