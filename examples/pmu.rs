#![no_main]
#![no_std]

extern crate panic_rtt_target;

use lpc8xx_hal::{
    cortex_m::interrupt,
    cortex_m_rt::entry,
    nb::block,
    pac::{Interrupt, NVIC},
    pmu::LowPowerClock,
    prelude::*,
    syscon::WktWakeup,
    usart, CorePeripherals, Peripherals,
};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut pmu = p.PMU.split();
    let mut swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    // 115200 baud
    syscon.uartfrg.set_clkdiv(6);
    syscon.uartfrg.set_frgmult(22);
    syscon.uartfrg.set_frgdiv(0xff);
    let clock_config = usart::Clock::new(&syscon.uartfrg, 0, 16);

    let (u0_rxd, _) = swm
        .movable_functions
        .u0_rxd
        .assign(p.pins.pio0_0.into_swm_pin(), &mut swm.handle);
    let (u0_txd, _) = swm
        .movable_functions
        .u0_txd
        .assign(p.pins.pio0_4.into_swm_pin(), &mut swm.handle);

    let mut serial =
        p.USART0
            .enable(&clock_config, &mut syscon.handle, u0_rxd, u0_txd);

    let _ = pmu.low_power_clock.enable(&mut pmu.handle);

    let mut wkt = p.WKT.enable(&mut syscon.handle);
    wkt.select_clock::<LowPowerClock>();

    let five_seconds: u32 = 10_000 * 5;

    // Need to re-assign some stuff that's needed inside the closure. Otherwise
    // it will try to move stuff that's still borrowed outside of it.
    let mut pmu = pmu.handle;
    let mut scb = cp.SCB;
    let mut syscon = syscon.handle;

    interrupt::free(|_| {
        // Enable the interrupt for the self-wake-up timer. Doing this within
        // `interrupt::free` will allow the interrupt to wake up the system, if
        // it's sleeping. But the interrupt handler won't run, which means we
        // don't have to define one.
        //
        // This is safe, as this won't interfere with the critical section.
        unsafe { NVIC::unmask(Interrupt::WKT) }

        // Busy Waiting
        serial
            .bwrite_all(b"5 seconds of busy waiting...\n")
            .expect("UART write shouldn't fail");
        wkt.start(five_seconds);
        while let Err(nb::Error::WouldBlock) = wkt.wait() {}

        // The timer has finished running and the counter is at zero. Therefore
        // the interrupt is currently pending. If we don't do anything about
        // this, it will stay pending and will interfere with the following
        // calls to `wait`.
        // This means we need to clear the interrupt. To prevent it from
        // becoming pending again right away, we always do this _after_ starting
        // the timer from here on out.

        // Sleep mode
        serial
            .bwrite_all(b"5 seconds of sleep mode...\n")
            .expect("UART write shouldn't fail");
        wkt.start(five_seconds);
        NVIC::unpend(Interrupt::WKT);
        while let Err(nb::Error::WouldBlock) = wkt.wait() {
            pmu.enter_sleep_mode(&mut scb);
        }

        // Without this, the WKT interrupt won't wake up the system from
        // deep-sleep and power-down modes.
        syscon.enable_interrupt_wakeup::<WktWakeup>();

        // Deep-sleep mode
        serial
            .bwrite_all(b"5 seconds of deep-sleep mode...\n")
            .expect("UART write shouldn't fail");
        block!(serial.flush()).expect("Flush shouldn't fail");
        wkt.start(five_seconds);
        NVIC::unpend(Interrupt::WKT);
        while let Err(nb::Error::WouldBlock) = wkt.wait() {
            unsafe { pmu.enter_deep_sleep_mode(&mut scb) };
        }

        // Power-down mode
        serial
            .bwrite_all(b"5 seconds of power-down mode...\n")
            .expect("UART write shouldn't fail");
        block!(serial.flush()).expect("Flush shouldn't fail");
        wkt.start(five_seconds);
        NVIC::unpend(Interrupt::WKT);
        while let Err(nb::Error::WouldBlock) = wkt.wait() {
            unsafe { pmu.enter_power_down_mode(&mut scb) };
        }

        // A demonstration of deep power-down mode is currently missing from
        // this example, due to some problems with my setup that prevent me from
        // testing it for the time being.

        serial
            .bwrite_all(b"Done\n")
            .expect("UART write shouldn't fail");

        loop {}
    })
}
