#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{cortex_m_rt::entry, gpio::Level, prelude::*, Device};

#[entry]
fn main() -> ! {
    // Get access to the device's peripherals. Since only one instance of this
    // struct can exist, the call to `take` returns an `Option<Device>`.
    // If we tried to call the method a second time, it would return `None`, but
    // we're only calling it the one time here, so we can safely `unwrap` the
    // `Option` without causing a panic.
    let device = Device::take().unwrap();

    // Initialize the APIs of the peripherals we need.
    let gpio = {
        let mut syscon = device.SYSCON.split();
        device.GPIO.enable(&mut syscon.handle)
    };

    // Select pin for LED
    let led = device.pins.pio1_1;

    // Select pin for button
    let button = device.pins.pio0_4;

    // Configure the LED pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let mut led = led.into_output_pin(gpio.tokens.pio1_1, Level::Low);

    // Configure the button pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let button = button.into_input_pin(gpio.tokens.pio0_4);

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
