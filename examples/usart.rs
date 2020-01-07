#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{
    cortex_m_rt::entry, prelude::*, syscon::clocksource::PeripheralClockConfig, Peripherals,
};

#[cfg(feature = "845")]
use lpc8xx_hal::pac::syscon::frg::frgclksel::SEL_A;

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let mut swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    // TODO
    //
    // For some reason, the clock for swm need to be enabled, even though
    // it should be enabled from the start
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
    let clock_config = {
        syscon.uartfrg.set_clkdiv(6);
        syscon.uartfrg.set_frgmult(22);
        syscon.uartfrg.set_frgdiv(0xff);
        PeripheralClockConfig::new(&syscon.uartfrg, 0)
    };

    #[cfg(feature = "845")]
    // Set baud rate to 115200 baud
    //
    // This is pretty much the same as for the LPC82x. The differences are
    // that we're using a fractional generator that can be used as a clock
    // source for other peripherals (which doesn't make a difference in this
    // case), and we get our division by 6 by setting the BRGVAL of the
    // USART instance (setting its value to 5 means division by 6).
    let clock_config = {
        syscon.frg0.select_clock(SEL_A::FRO);
        syscon.frg0.set_mult(22);
        syscon.frg0.set_div(0xFF);
        PeripheralClockConfig::new(&syscon.frg0, 5)
    };
    // The internal oscillator FRO can also be used as a clock source.
    // Since it can only be divided by a whole number, it's doesn't work for
    // high baudrates, but for 9600 Baud it works fine
    //
    // let clock_config = { PeripheralClockConfig::new(&syscon.iosc, (12_000_000 / (9_600 * 16)) as u16) };

    // Make the rx & tx pins available to the switch matrix API, by changing
    // their state using `into_swm_pin`. This is required, because we're going
    // to use the switch matrix to assigne the USART0 functions to those pins.
    //
    // WARNING: The pinout for the lpc845brk uses tx/rx as seen from the
    // perspective from the serial adapter, so this is used the opposite way

    #[cfg(feature = "82x")]
    let tx_pin = swm.pins.pio0_7.into_swm_pin();
    #[cfg(feature = "82x")]
    let rx_pin = swm.pins.pio0_18.into_swm_pin();
    #[cfg(feature = "845")]
    let tx_pin = swm.pins.pio0_25.into_swm_pin();
    #[cfg(feature = "845")]
    let rx_pin = swm.pins.pio0_24.into_swm_pin();

    // Assign U0_RXD & U0_TXD to the rx & tx pins. On the LPCXpresso824-MAX &
    // LPC845-BRK development boards, they're connected to the integrated USB to
    // Serial converter. So by using the pins, we can use them to communicate
    // with a host PC, without additional hardware.
    let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(rx_pin, &mut swm.handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd.assign(tx_pin, &mut swm.handle);

    // Enable USART0
    let serial = p
        .USART0
        .enable(&clock_config, &mut syscon.handle, u0_rxd, u0_txd);

    // Send a string via USART0, blocking until it has been sent
    serial
        .tx()
        .bwrite_all(b"Hello, world!\n")
        .expect("UART write shouldn't fail");

    // We're done. Let's do nothing until someone resets the microcontroller.
    loop {}
}
