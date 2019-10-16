#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use lpc8xx_hal::prelude::*;
use lpc8xx_hal::Peripherals;

use cortex_m_rt::entry;

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
    // let mut wkt = p.WKT.enable(&mut syscon.handle);
    let gpio = p.GPIO.enable(&mut syscon.handle);

    // Configure the PIO1_1 pin. The API tracks the state of pins at
    // compile-time, to prevent any mistakes.
    let mut pio1_1 = swm.pins.pio1_1.into_gpio_pin(&gpio).into_output();

    // Blink the LED
    loop {
        // For this simple demo accurate timing isn't required and this is the
        // simplest Method to delay
        for _ in 0..1000000 {
            #[allow(deprecated)]
            pio1_1.set_high();
        }
        for _ in 0..1000000 {
            #[allow(deprecated)]
            pio1_1.set_low();
        }
    }
}
