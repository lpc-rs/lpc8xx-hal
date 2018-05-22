#![no_main]
#![no_std]


#[macro_use]
extern crate cortex_m_rt;
extern crate lpc82x_hal;
extern crate panic_abort;


use cortex_m_rt::ExceptionFrame;

use lpc82x_hal::prelude::*;
use lpc82x_hal::{
    raw,
    GPIO,
    SWM,
    SYSCON,
    WKT,
};
use lpc82x_hal::clock::Ticks;
use lpc82x_hal::sleep;


entry!(main);

fn main() -> ! {
    // Create the struct we're going to use to access all the peripherals. This
    // is unsafe, because we're only allowed to create one instance.
    let mut peripherals = raw::Peripherals::take().unwrap();

    // Create the peripheral interfaces.
    let     gpio   = GPIO::new(peripherals.GPIO_PORT);
    let     swm    = SWM::new(peripherals.SWM);
    let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
    let     wkt    = WKT::new(peripherals.WKT);

    // Other peripherals need to be initialized. Trying to use the API before
    // initializing them will actually lead to compile-time errors.
    let     gpio_handle = gpio.handle.enable(&mut syscon.handle);
    let mut swm_handle  = swm.handle.enable(&mut syscon.handle);
    let mut wkt         = wkt.enable(&mut syscon.handle);

    // We're going to need a clock for sleeping. Let's use the IRC-derived clock
    // that runs at 750 kHz.
    let clock = syscon.irc_derived_clock.enable(
        &mut syscon.handle,
        syscon.irc,
        syscon.ircout,
    );

    // In the next step, we need to configure the pin PIO0_3 and its fixed
    // function SWCLK. The API tracks the state of both of those, to prevent any
    // mistakes on our side. However, since we could have changed the state of
    // the hardware before initializing the API, the API can't know what state
    // it is currently in.
    // Let's affirm that we haven't changed anything, and that PIO0_3 and SWCLK
    // are still in their initial states.
    let pio0_3 = unsafe { gpio.pins.pio0_3.affirm_default_state()          };
    let swclk  = unsafe { swm.fixed_functions.swclk.affirm_default_state() };

    // Configure PIO0_3 as GPIO output, so we can use it to blink an LED.
    let (_, pio0_3) = swclk
        .unassign(pio0_3, &mut swm_handle);
    let mut pio0_3 = pio0_3
        .into_unused_pin()
        .into_gpio_pin(&gpio_handle)
        .into_output();

    // Let's already initialize the durations that we're going to sleep for
    // between changing the LED state. We do this by specifying the number of
    // clock ticks directly, but a real program could use a library that allows
    // us to specify the time in milliseconds.
    // Each duration also keeps a reference to the clock, as to prevent other
    // parts of the program from accidentally disabling the clock, or changing
    // its settings.
    let high_time = Ticks { value:  37_500, clock: &clock }; //  50 ms
    let low_time  = Ticks { value: 712_500, clock: &clock }; // 950 ms

    // Since this is a simple example, we don't want to deal with interrupts
    // here. Let's just use busy waiting as a sleeping strategy.
    let mut sleep = sleep::Busy::prepare(&mut wkt);

    // Blink the LED
    loop {
        pio0_3.set_high();
        sleep.sleep(high_time);
        pio0_3.set_low();
        sleep.sleep(low_time);
    }
}


exception!(*, default_handler);
exception!(HardFault, handle_hard_fault);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception or interrupt: {}", irqn);
}

fn handle_hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
