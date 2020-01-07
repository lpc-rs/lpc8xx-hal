#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{clock::Ticks, cortex_m_rt::entry, prelude::*, sleep, Peripherals};

#[entry]
fn main() -> ! {
    // Get access to the device's peripherals. Since only one instance of this
    // struct can exist, the call to `take` returns an `Option<Peripherals>`.
    // If we tried to call the method a second time, it would return `None`, but
    // we're only calling it the one time here, so we can safely `unwrap` the
    // `Option` without causing a panic.
    let p = Peripherals::take().unwrap();

    // Initialize the APIs of the peripherals we need.
    let swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();
    let mut wkt = p.WKT.enable(&mut syscon.handle);
    #[cfg(feature = "82x")]
    let gpio = p.GPIO; // GPIO is initialized by default on LPC82x.
    #[cfg(feature = "845")]
    let gpio = p.GPIO.enable(&mut syscon.handle);

    // We're going to need a clock for sleeping. Let's use the internal oscillator/IRC/FRO-derived clock
    // that runs at 750 kHz.
    let clock = syscon.iosc_derived_clock;

    // Select pin for LED
    #[cfg(feature = "82x")]
    let led = swm.pins.pio0_12;
    #[cfg(feature = "845")]
    let led = swm.pins.pio1_1;

    // Configure the LED pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let mut led = led.into_gpio_pin(&gpio).into_output();

    // Let's already initialize the durations that we're going to sleep for
    // between changing the LED state. We do this by specifying the number of
    // clock ticks directly, but a real program could use a library that allows
    // us to specify the time in milliseconds.
    // Each duration also keeps a reference to the clock, as to prevent other
    // parts of the program from accidentally disabling the clock, or changing
    // its settings.
    let low_time = Ticks {
        value: 37_500,
        clock: &clock,
    }; //  50 ms
    let high_time = Ticks {
        value: 712_500,
        clock: &clock,
    }; // 950 ms

    // Since this is a simple example, we don't want to deal with interrupts
    // here. Let's just use busy waiting as a sleeping strategy.
    let mut sleep = sleep::Busy::prepare(&mut wkt);

    // Blink the LED
    loop {
        led.set_high().unwrap();
        sleep.sleep(high_time);
        led.set_low().unwrap();
        sleep.sleep(low_time);
    }
}
