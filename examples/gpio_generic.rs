#![no_main]
#![no_std]

extern crate panic_rtt_target;

use lpc8xx_hal::{
    cortex_m_rt::entry,
    delay::Delay,
    gpio::{direction::Dynamic, GpioPin, Level},
    pins::{DynamicPinDirection, GenericPin},
    prelude::*,
    CorePeripherals, Peripherals,
};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    // Get access to the device's peripherals. Since only one instance of this
    // struct can exist, the call to `take` returns an `Option<Peripherals>`.
    // If we tried to call the method a second time, it would return `None`, but
    // we're only calling it the one time here, so we can safely `unwrap` the
    // `Option` without causing a panic.
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    // Initialize the APIs of the peripherals we need.
    let mut delay = Delay::new(cp.SYST);

    let mut syscon = p.SYSCON.split();
    let gpio = p.GPIO.enable(&mut syscon.handle);

    // Select pins for all three LEDs
    let (green_led, green_led_token) = (p.pins.pio1_0, gpio.tokens.pio1_0);
    let (blue_led, blue_led_token) = (p.pins.pio1_1, gpio.tokens.pio1_1);
    let (red_led, red_led_token) = (p.pins.pio1_2, gpio.tokens.pio1_2);

    // Generate Generic Dynamic Pins from `Token` + `Pin`s and gather them for further batch
    // processing.
    let mut leds: [GpioPin<GenericPin, Dynamic>; 3] = [
        green_led.into_generic_dynamic_pin(
            green_led_token,
            Level::High, // led is off initially
            DynamicPinDirection::Output,
        ),
        blue_led.into_generic_dynamic_pin(
            blue_led_token,
            Level::High, // led is off initially
            DynamicPinDirection::Output,
        ),
        red_led.into_generic_dynamic_pin(
            red_led_token,
            Level::High, // led is off initially
            DynamicPinDirection::Output,
        ),
    ];

    loop {
        // Blink all LED colors by looping through the array that holds them
        // This should make the on-board LED blink like this:
        // 🟢 🔵 🔴 🟢 🔵 🔴 🟢 🔵 🔴 ...
        for led in leds.iter_mut() {
            blink_led(led, &mut delay);
        }
    }
}

/// Turn `led` on for 1000 ms
fn blink_led(led: &mut GpioPin<GenericPin, Dynamic>, delay: &mut Delay) {
    led.set_low();
    delay.delay_ms(1_000_u16);
    led.set_high();
}
