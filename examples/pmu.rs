#![no_main]
#![no_std]


extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate lpc82x_hal;
extern crate nb;
extern crate panic_abort;


use cortex_m::interrupt;
use cortex_m_rt::ExceptionFrame;

use lpc82x_hal::prelude::*;
use lpc82x_hal::{
    raw,
    Peripherals,
};
use lpc82x_hal::pmu::LowPowerClock;
use lpc82x_hal::raw::Interrupt;
use lpc82x_hal::usart::BaudRate;


entry!(main);

fn main() -> ! {
    let mut cp = raw::CorePeripherals::take().unwrap();
    let mut p  = Peripherals::take().unwrap();

    let mut pmu    = p.pmu.split();
    let mut swm    = p.swm.split();
    let mut syscon = p.syscon.split();

    // 115200 baud
    syscon.uartfrg.set_clkdiv(6);
    syscon.uartfrg.set_frgmult(22);
    syscon.uartfrg.set_frgdiv(0xff);
    let baud_rate = BaudRate::new(&syscon.uartfrg, 0);

    let (u0_rxd, _) = swm.movable_functions.u0_rxd.assign(
        swm.pins.pio0_0.into_swm_pin(),
        &mut swm.handle,
    );
    let (u0_txd, _) = swm.movable_functions.u0_txd.assign(
        swm.pins.pio0_4.into_swm_pin(),
        &mut swm.handle,
    );

    let mut serial = p.usart0
        .enable(
            &baud_rate,
            &mut syscon.handle,
            u0_rxd,
            u0_txd,
        )
        .expect("UART initialization shouldn't fail");

    let _ = pmu.low_power_clock.enable(&mut pmu.handle);

    // Need to re-assign PMU handle. Otherwise, the closure will try to capture
    // all of `PMU`, which can't be moved because `low_power_clock` has been
    // moved out of it in the previous assignment.
    let mut pmu = pmu.handle;

    let mut wkt = p.wkt.enable(&mut syscon.handle);
    wkt.select_clock::<LowPowerClock>();

    let five_seconds: u32 = 10_000 * 5;

    interrupt::free(|_| {
        cp.NVIC.enable(Interrupt::WKT);

        serial.bwrite_all(b"5 seconds of busy waiting...\n")
            .expect("UART write shouldn't fail");
        wkt.start(five_seconds);
        while let Err(nb::Error::WouldBlock) = wkt.wait() {}

        serial.bwrite_all(b"5 seconds of sleep mode...\n")
            .expect("UART write shouldn't fail");
        wkt.start(five_seconds);
        cp.NVIC.clear_pending(Interrupt::WKT);
        while let Err(nb::Error::WouldBlock) = wkt.wait() {
            pmu.enter_sleep_mode(&mut cp.SCB);
        }

        serial.bwrite_all(b"Done\n")
            .expect("UART write shouldn't fail");

        loop {}
    })
}


exception!(*, default_handler);
exception!(HardFault, handle_hard_fault);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception or interrupt: {}", irqn);
}

fn handle_hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
