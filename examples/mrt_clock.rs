#![no_main]
#![no_std]

extern crate panic_rtt_target;

use embedded_time::{duration::Extensions as _, Clock as _};
use lpc8xx_hal::{cortex_m_rt::entry, gpio::Level, mrt, Peripherals};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let p = Peripherals::take().unwrap();

    let mut syscon = p.SYSCON.split();
    let gpio = p.GPIO.enable(&mut syscon.handle);
    let mut mrt = p.MRT0.split(&mut syscon.handle).mrt0;

    let mut led = p
        .pins
        .pio1_1
        .into_output_pin(gpio.tokens.pio1_1, Level::Low);

    loop {
        mrt.start(mrt::MAX_VALUE);

        let timer = mrt.new_timer(1u32.seconds());
        timer.start().unwrap().wait().unwrap();

        led.toggle();
    }
}
