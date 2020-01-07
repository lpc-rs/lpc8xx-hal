#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{cortex_m_rt::entry, prelude::*, Peripherals};

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
    #[cfg(feature = "82x")]
    let gpio = p.GPIO; // GPIO is initialized by default on LPC82x.
    #[cfg(feature = "845")]
    let gpio = {
        let mut syscon = p.SYSCON.split();
        p.GPIO.enable(&mut syscon.handle)
    };

    // Select pin for LED
    #[cfg(feature = "82x")]
    let led = swm.pins.pio0_12;
    #[cfg(feature = "845")]
    let led = swm.pins.pio1_1;

    // Configure the LED pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let mut led = led.into_gpio_pin(&gpio).into_output();

    // Blink the LED
    //
    // For this simple demo accurate timing isn't required and this is the
    // simplest method to delay. The values are chosen to give a nice blinking
    // pattern in release mode.
    loop {
        for _ in 0..1_000_000 {
            led.set_high().unwrap();
        }
        for _ in 0..100_000 {
            led.set_low().unwrap();
        }
    }
}
