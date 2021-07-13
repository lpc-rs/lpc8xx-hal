#![no_main]
#![no_std]

extern crate panic_rtt_target;

#[rtic::app(device = lpc8xx_hal::pac, peripherals = false)]
mod app {
    use lpc8xx_hal::{
        delay::Delay,
        gpio::{direction::Output, GpioPin, Level},
        pins::PIO1_1,
        prelude::*,
        Peripherals,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        delay: Delay,
        led: GpioPin<PIO1_1, Output>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_target::rtt_init_print!();

        let p = Peripherals::take().unwrap();

        let delay = Delay::new(cx.core.SYST);

        let mut syscon = p.SYSCON.split();
        let gpio = p.GPIO.enable(&mut syscon.handle);

        let led = p
            .pins
            .pio1_1
            .into_output_pin(gpio.tokens.pio1_1, Level::Low);

        (Shared {}, Local { delay, led }, init::Monotonics())
    }

    #[idle(local = [delay, led])]
    fn idle(cx: idle::Context) -> ! {
        let delay = cx.local.delay;
        let led = cx.local.led;

        loop {
            led.set_high();
            delay.delay_ms(700_u16);
            led.set_low();
            delay.delay_ms(50_u16);
        }
    }
}
