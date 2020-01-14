#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{cortex_m_rt::entry, prelude::*, Peripherals};

use nb::block;
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
    let [mut timer, _, _, _] = p.MRT0.split(&mut syscon.handle);

    #[cfg(feature = "82x")]
    let gpio = p.GPIO; // GPIO is initialized by default on LPC82x.
    #[cfg(feature = "845")]
    let gpio = p.GPIO.enable(&mut syscon.handle);

    // Select pin for LED
    #[cfg(feature = "82x")]
    let led = swm.pins.pio0_12;
    #[cfg(feature = "845")]
    let led = swm.pins.pio1_1;

    // Configure the LED pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let mut led = led.into_gpio_pin(&gpio).into_output();

    // Start the timer with an intervall of 12_000_000 ticks
    timer.start(12_000_000u32);

    // Blink the LED using the systick with the delay traits
    loop {
        block!(timer.wait()).unwrap();
        led.set_high().unwrap();
        block!(timer.wait()).unwrap();
        led.set_low().unwrap();
    }
}
