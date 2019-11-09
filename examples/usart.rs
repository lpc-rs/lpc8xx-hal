#![no_main]
#![no_std]


extern crate panic_halt;


use lpc8xx_hal::{
    prelude::*,
    Peripherals,
    cortex_m_rt::entry,
    usart::BaudRate,
};

#[cfg(feature = "845")]
use lpc8xx_hal::pac::syscon::frg::frgclksel::SEL_A::MAIN_CLK;


#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let mut swm    = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    // Set baud rate to 115200 baud. How this workds exactly varies depends on
    // the target platform.
    #[cfg(feature = "82x")]
    let baud_rate = {
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
        BaudRate::new(&syscon.uartfrg, 0)
    };
    #[cfg(feature = "845")]
    let baud_rate = {
        // This is pretty much the same as for the LPC82x. The differences are
        // that we're using a fractional generator that can be used as a clock
        // source for other peripherals (which doesn't make a difference in this
        // case), and we get our division by 6 by setting the BRGVAL of the
        // USART instance (setting its value to 5 means division by 6).
        syscon.frg0.select_clock(MAIN_CLK);
        syscon.frg0.set_mult(22);
        syscon.frg0.set_div(0xff);
        BaudRate::new(&syscon.frg0, 5)
    };

    // Here we assign USART0's receive (U0_RXD) and transmit (U0_TXD) functions
    // to specific pins. Those happen to be the pins that are connected to the
    // on-board programmer on the respective development boards for the target
    // platforms. The on-board programmer makes the serial connection available
    // to the host PC via USB.
    #[cfg(feature = "82x")]
    let (u0_rxd, u0_txd) = (
        swm.pins.pio0_18.into_swm_pin(),
        swm.pins.pio0_7.into_swm_pin(),
    );
    #[cfg(feature = "845")]
    let (u0_rxd, u0_txd) = (
        swm.pins.pio0_25.into_swm_pin(),
        swm.pins.pio0_24.into_swm_pin(),
    );

    let (u0_rxd, _) = swm.movable_functions.u0_rxd
        .assign(u0_rxd, &mut swm.handle);
    let (u0_txd, _) = swm.movable_functions.u0_txd
        .assign(u0_txd, &mut swm.handle);

    // Enable USART0
    let serial = p.USART0.enable(
        &baud_rate,
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
    );

    loop {
        // Send a string via USART0, blocking until it has been sent
        serial.tx().bwrite_all(b"Hello, world!\n")
            .expect("UART write shouldn't fail");
    }
}
