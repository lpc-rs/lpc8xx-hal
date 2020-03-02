#![no_main]
#![no_std]

use lpc8xx_hal::{delay::Delay, prelude::*, Peripherals};
use panic_halt as _;

#[rtfm::app(device = lpc8xx_hal::pac)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        let p = Peripherals::take().unwrap();

        let mut delay = Delay::new(cx.core.SYST);

        let mut syscon = p.SYSCON.split();
        let gpio = p.GPIO.enable(&mut syscon.handle);

        let mut led = p.pins.pio1_1.into_output_pin(gpio.tokens.pio1_1);

        loop {
            led.set_high().unwrap();
            delay.delay_ms(700_u16);
            led.set_low().unwrap();
            delay.delay_ms(50_u16);
        }
    }
};
