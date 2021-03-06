#![no_main]
#![no_std]

extern crate panic_rtt_target;

use lpc8xx_hal::{cortex_m_rt::entry, gpio::Level, Peripherals};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    // Get access to the device's peripherals. Since only one instance of this
    // struct can exist, the call to `take` returns an `Option<Peripherals>`.
    // If we tried to call the method a second time, it would return `None`, but
    // we're only calling it the one time here, so we can safely `unwrap` the
    // `Option` without causing a panic.
    let p = Peripherals::take().unwrap();

    // Initialize the APIs of the peripherals we need.
    let gpio = {
        let mut syscon = p.SYSCON.split();
        p.GPIO.enable(&mut syscon.handle)
    };

    // Select pin for LED
    let led = p.pins.pio1_1;

    // Select pin for button
    let button = p.pins.pio0_4;

    // Configure the LED pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let mut led = led.into_output_pin(gpio.tokens.pio1_1, Level::Low);

    // Configure the button pin. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let button = button.into_input_pin(gpio.tokens.pio0_4);

    // Display the state of the button on the led
    loop {
        // If the button is high (not pressed)
        if button.is_high() {
            // Disable the LED
            led.set_high();
        } else {
            // Otherwise, enable it
            led.set_low();
        }
    }
}
