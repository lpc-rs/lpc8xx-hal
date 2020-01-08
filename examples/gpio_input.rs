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

    let gpio = {
        let mut syscon = p.SYSCON.split();
        p.GPIO.enable(&mut syscon.handle)
    };

    // Select pin for LED
    let led = swm.pins.pio1_1;

    // Select pin for button
    let button = swm.pins.pio0_4;

    // Configure the LED pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let mut led = led.into_gpio_pin(&gpio).into_output();

    // Configure the button pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let button = button.into_gpio_pin(&gpio).into_input();

    // Display the state of the button on the led
    loop {
        // If the button is high (not pressed)
        if button.is_high().unwrap() {
            // Disable the LED
            led.set_high().unwrap();
        } else {
            // Otherwise, enable it
            led.set_low().unwrap();
        }
    }
}
