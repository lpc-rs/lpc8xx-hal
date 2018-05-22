#![no_main]
#![no_std]


#[macro_use]
extern crate cortex_m_rt;
extern crate lpc82x_hal;
extern crate panic_abort;


use cortex_m_rt::ExceptionFrame;

use lpc82x_hal::prelude::*;
use lpc82x_hal::{
    GPIO,
    SYSCON,
    SWM,
    raw,
};
use lpc82x_hal::usart::{
    BaudRate,
    USART,
};


entry!(main);

fn main() -> ! {
    let mut peripherals = raw::Peripherals::take().unwrap();

    let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
    let     swm    = SWM::new(peripherals.SWM);
    let     gpio   = GPIO::new(peripherals.GPIO_PORT);
    let     usart0 = USART::new(peripherals.USART0);

    let mut swm_handle = swm.handle.enable(&mut syscon.handle);

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

    // Prepare PIO0_0 and PIO0_4. The `init` method we call below needs pins to
    // assign the USART's movable function to. For that, the pins need to be
    // unused. Since PIO0_0 and PIO0_4 are unused by default, we just have to
    // promise the API that we didn't change the default state up till now.
    let pio0_0 = unsafe { gpio.pins.pio0_0.affirm_default_state() }
        .into_swm_pin();
    let pio0_4 = unsafe { gpio.pins.pio0_4.affirm_default_state() }
        .into_swm_pin();

    // We also need to provide USART0's movable functions. Those need to be
    // unassigned, and since they are unassigned by default, we just need to
    // promise the API that we didn't change them.
    let u0_rxd = unsafe { swm.movable_functions.u0_rxd.affirm_default_state() };
    let u0_txd = unsafe { swm.movable_functions.u0_txd.affirm_default_state() };

    let (u0_rxd, _) = u0_rxd.assign(pio0_0, &mut swm_handle);
    let (u0_txd, _) = u0_txd.assign(pio0_4, &mut swm_handle);

    // Initialize USART0. This should never fail, as the only reason `init`
    // returns a `Result::Err` is when the transmitter is busy, which it
    // shouldn't be right now.
    let mut serial = usart0
        .enable(
            &baud_rate,
            &mut syscon.handle,
            u0_rxd,
            u0_txd,
        )
        .expect("UART initialization shouldn't fail");

    // Write a string, blocking until it has finished writing
    serial.bwrite_all(b"Hello, world!\n")
        .expect("UART write shouldn't fail");

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
