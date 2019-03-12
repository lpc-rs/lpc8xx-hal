#![no_main]
#![no_std]


#[macro_use]
extern crate cortex_m_rt;
extern crate lpc82x_hal;
extern crate panic_abort;


use lpc82x_hal::prelude::*;
use lpc82x_hal::Peripherals;
use lpc82x_hal::clock::Ticks;
use lpc82x_hal::sleep;


#[entry]
fn main() -> ! {
    // Get access to the device's peripherals. Since only one instance of this
    // struct can exist, the call to `take` returns an `Option<Peripherals>`.
    // If we tried to call the method a second time, it would return `None`, but
    // we're only calling it the one time here, so we can safely `unwrap` the
    // `Option` without causing a panic.
    let p = Peripherals::take().unwrap();

    // Initialize the APIs of the peripherals we need.
    let     swm    = p.SWM.split();
    let mut syscon = p.SYSCON.split();
    let mut wkt    = p.WKT.enable(&mut syscon.handle);

    // We're going to need a clock for sleeping. Let's use the IRC-derived clock
    // that runs at 750 kHz.
    let clock = syscon.irc_derived_clock;

    // Configure the PIO0_12 pin. The API tracks the state of pins at
    // compile-time, to prevent any mistakes.
    let mut pio0_12 = swm.pins.pio0_12
        .into_gpio_pin(&p.GPIO)
        .into_output();

    // Let's already initialize the durations that we're going to sleep for
    // between changing the LED state. We do this by specifying the number of
    // clock ticks directly, but a real program could use a library that allows
    // us to specify the time in milliseconds.
    // Each duration also keeps a reference to the clock, as to prevent other
    // parts of the program from accidentally disabling the clock, or changing
    // its settings.
    let low_time  = Ticks { value:  37_500, clock: &clock }; //  50 ms
    let high_time = Ticks { value: 712_500, clock: &clock }; // 950 ms

    // Since this is a simple example, we don't want to deal with interrupts
    // here. Let's just use busy waiting as a sleeping strategy.
    let mut sleep = sleep::Busy::prepare(&mut wkt);

    // Blink the LED
    loop {
        pio0_12.set_high();
        sleep.sleep(high_time);
        pio0_12.set_low();
        sleep.sleep(low_time);
    }
}
