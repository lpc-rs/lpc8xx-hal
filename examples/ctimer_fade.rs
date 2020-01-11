#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{cortex_m_rt::entry, delay::Delay, prelude::*, Peripherals};

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
    let mut delay = Delay::new(p.SYST);
    let mut syscon = p.SYSCON.split();

    let mut handle = swm.handle.enable(&mut syscon.handle);

    // Use 8 bit pwm
    let (red_pwm, green_pwm, blue_pwm) =
        p.CTIMER0.start_pwm(256, 0, &mut syscon.handle);

    // Select pin for the RGB LED
    let green = swm.pins.pio1_0.into_swm_pin();
    let blue = swm.pins.pio1_1.into_swm_pin();
    let red = swm.pins.pio1_2.into_swm_pin();

    // Configure the LED pins. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let (red, _) = swm.movable_functions.t0_mat0.assign(red, &mut handle);
    let (green, _) = swm.movable_functions.t0_mat1.assign(green, &mut handle);
    let (blue, _) = swm.movable_functions.t0_mat2.assign(blue, &mut handle);

    let mut red = red_pwm.configure(red);
    let mut green = green_pwm.configure(green);
    let mut blue = blue_pwm.configure(blue);
    // Fade each color after anothe
    loop {
        for i in 0..red.get_max_duty() {
            delay.delay_ms(4_u8);
            red.set_duty(i);
        }
        for i in (0..red.get_max_duty()).rev() {
            delay.delay_ms(4_u8);
            red.set_duty(i);
        }
        for i in 0..green.get_max_duty() {
            delay.delay_ms(4_u8);
            green.set_duty(i);
        }
        for i in (0..green.get_max_duty()).rev() {
            delay.delay_ms(4_u8);
            green.set_duty(i);
        }
        for i in 0..blue.get_max_duty() {
            delay.delay_ms(4_u8);
            blue.set_duty(i);
        }
        for i in (0..blue.get_max_duty()).rev() {
            delay.delay_ms(4_u8);
            blue.set_duty(i);
        }
    }
}
