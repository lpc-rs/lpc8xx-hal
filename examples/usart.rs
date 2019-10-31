#![no_main]
#![no_std]


extern crate panic_halt;


use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    cortex_m_rt::entry,
    usart::BaudRate,
};


#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let mut swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    // TODO
    swm.handle = swm
        .handle
        .disable(&mut syscon.handle)
        .enable(&mut syscon.handle);

    #[cfg(feature = "82x")]
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
    let baud_rate = {
        syscon.uartfrg.set_clkdiv(6);
        syscon.uartfrg.set_frgmult(22);
        syscon.uartfrg.set_frgdiv(0xff);
        BaudRate::new(&syscon.uartfrg, 0)
    };

    #[cfg(feature = "845")]
    // Set baud rate to 115200 baud
    //
    // FRG0 is used as the clock for USART0, the
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
    let baud_rate = {
        syscon.frg0.set_frgmult(22);
        syscon.frg0.set_frgdiv(0xFF);
        BaudRate::new(&syscon.frg0, 5)
    };

    #[cfg(feature = "82x")]
    // Make PIO0_7 and PIO0_18 available to the switch matrix API, by changing
    // their state using `into_swm_pin`. This is required, because we're going
    // to use the switch matrix to assigne the USART0 functions to those pins.
    let tx_pin = swm.pins.pio0_7.into_swm_pin();
    #[cfg(feature = "82x")]
    let rx_pin = swm.pins.pio0_18.into_swm_pin();

    #[cfg(feature = "845")]
    // Make PIO0_25 and PIO0_24 available to the switch matrix API, by changing
    // their state using `into_swm_pin`. This is required, because we're going
    // to use the switch matrix to assigne the USART0 functions to those pins.
    //
    // WARNING: The pinout for the lpc845brk uses tx/rx as seen from the
    // perspective from the serial adapter, so this is used the opposite way
    let tx_pin = swm.pins.pio0_25.into_swm_pin();
    #[cfg(feature = "845")]
    let rx_pin = swm.pins.pio0_24.into_swm_pin();

    // Assign U0_RXD to PIO0_18 and U0_TXD to PIO0_7. On the LPCXpresso824-MAX
    // development board, those pins are bridged to the board's USB port. So by
    // using the pins, we can use them to communicate with a host PC, without
    // additional hardware.
    let (u0_rxd, _) = swm.movable_functions.u0_rxd
        .assign(rx_pin, &mut swm.handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd
        .assign(tx_pin,  &mut swm.handle);

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
