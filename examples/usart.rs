#![no_main]
#![no_std]

extern crate panic_rtt_target;

use lpc8xx_hal::{cortex_m_rt::entry, prelude::*, usart, Peripherals};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let p = Peripherals::take().unwrap();

    let swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    #[cfg(feature = "82x")]
    let mut handle = swm.handle;
    #[cfg(feature = "845")]
    let mut handle = swm.handle.enable(&mut syscon.handle); // SWM isn't enabled by default on LPC845.

    #[cfg(feature = "82x")]
    // Set baud rate to 115200 baud
    //
    // The common peripheral clock for all UART units, U_PCLK, needs to be set
    // to 16 times the desired baud rate. This results in a frequency of
    // 1843200 Hz for U_PCLK.
    //
    // We assume the main clock runs at 12 Mhz. To get close to the desired
    // frequency for U_PCLK, we divide that by 6 using UARTCLKDIV, resulting in
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
    let clock_config = {
        syscon.uartfrg.set_clkdiv(6);
        syscon.uartfrg.set_frgmult(22);
        syscon.uartfrg.set_frgdiv(0xff);
        usart::Clock::new(&syscon.uartfrg, 0, 16)
    };

    #[cfg(feature = "845")]
    // Set baud rate to 115200 baud
    let clock_config = usart::Clock::new_with_baudrate(115200);

    // Make the rx & tx pins available to the switch matrix API, by changing
    // their state using `into_swm_pin`. This is required, because we're going
    // to use the switch matrix to assign the USART0 functions to those pins.
    //
    // WARNING: The pinout for the lpc845brk uses tx/rx as seen from the
    // perspective from the serial adapter, so this is used the opposite way

    #[cfg(feature = "82x")]
    let tx_pin = p.pins.pio0_7.into_swm_pin();
    #[cfg(feature = "82x")]
    let rx_pin = p.pins.pio0_18.into_swm_pin();
    #[cfg(feature = "845")]
    let tx_pin = p.pins.pio0_25.into_swm_pin();
    #[cfg(feature = "845")]
    let rx_pin = p.pins.pio0_24.into_swm_pin();

    // Assign U0_RXD & U0_TXD to the rx & tx pins. On the LPCXpresso824-MAX &
    // LPC845-BRK development boards, they're connected to the integrated USB to
    // Serial converter. So by using the pins, we can use them to communicate
    // with a host PC, without additional hardware.
    let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(rx_pin, &mut handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd.assign(tx_pin, &mut handle);

    // Enable USART0
    let mut serial = p.USART0.enable_async(
        &clock_config,
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
        usart::Settings::default(),
    );

    // Read all incoming bytes and echo them back.
    loop {
        let b = nb::block!(serial.read()).expect("Error reading from USART");
        nb::block!(serial.write(b)).expect("Error writing to USART");
    }
}
