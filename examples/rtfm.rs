#![no_main]
#![no_std]

use lpc8xx_hal::{
    delay::Delay,
    gpio::{direction::Output, GpioPin, Level},
    pins::PIO1_1,
    prelude::*,
    Device,
};
use panic_halt as _;

#[rtfm::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        delay: Delay,
        led: GpioPin<PIO1_1, Output>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let device = Device::take().unwrap();

        let delay = Delay::new(cx.core.SYST);

        let mut syscon = device.SYSCON.split();
        let gpio = device.GPIO.enable(&mut syscon.handle);

        let led = device
            .pins
            .pio1_1
            .into_output_pin(gpio.tokens.pio1_1, Level::Low);

        init::LateResources { delay, led }
    }

    #[idle(resources = [delay, led])]
    fn idle(cx: idle::Context) -> ! {
        let delay = cx.resources.delay;
        let led = cx.resources.led;

        loop {
            led.set_high().unwrap();
            delay.delay_ms(700_u16);
            led.set_low().unwrap();
            delay.delay_ms(50_u16);
        }
    }
};
