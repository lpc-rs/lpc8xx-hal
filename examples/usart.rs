#![no_main]
#![no_std]


extern crate panic_halt;


use lpc8xx_hal::prelude::*;
use lpc8xx_hal::Peripherals;
use lpc8xx_hal::usart::BaudRate;

use cortex_m_rt::entry;


#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let mut swm    = p.SWM.split();
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
    // passing it as an argument to `BaudRate::new` (this will set the BRG
    // register). As we are already close enough to the desired value, we pass
    // 0, resulting in no further division.
    //
    // All of this is somewhat explained in the user manual, section 13.3.1.
    syscon.uartfrg.set_clkdiv(6);
    syscon.uartfrg.set_frgmult(22);
    syscon.uartfrg.set_frgdiv(0xff);
    let baud_rate = BaudRate::new(&syscon.uartfrg, 0);

    // Make PIO0_7 and PIO0_18 available to the switch matrix API, by changing
    // their state using `into_swm_pin`. This is required, because we're going
    // to use the switch matrix to assigne the USART0 functions to those pins.
    let pio0_7  = swm.pins.pio0_7.into_swm_pin();
    let pio0_18 = swm.pins.pio0_18.into_swm_pin();

    // Assign U0_RXD to PIO0_18 and U0_TXD to PIO0_7. On the LPCXpresso824-MAX
    // development board, those pins are bridged to the board's USB port. So by
    // using the pins, we can use them to communicate with a host PC, without
    // additional hardware.
    let (u0_rxd, _) = swm.movable_functions.u0_rxd
        .assign(pio0_18, &mut swm.handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd
        .assign(pio0_7,  &mut swm.handle);

    // Enable USART0
    let serial = p.USART0.enable(
        &baud_rate,
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
    );

    // Send a string via USART0, blocking until it has been sent
    serial.tx().bwrite_all(b"Hello, world!\n")
        .expect("UART write shouldn't fail");

    // We're done. Let's do nothing until someone resets the microcontroller.
    loop {}
}
