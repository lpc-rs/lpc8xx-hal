#![no_main]
#![no_std]

extern crate panic_rtt_target;

use embedded_hal_ryankurte::pwm::Pwm as _;
use lpc8xx_hal::{
    cortex_m_rt::entry, ctimer::Channels123, delay::Delay, prelude::*,
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
    let swm = p.SWM.split();
    let mut delay = Delay::new(cp.SYST);
    let mut syscon = p.SYSCON.split();

    let mut handle = swm.handle.enable(&mut syscon.handle);

    // Select pin for the RGB LED
    let green = p.pins.pio1_0.into_swm_pin();
    let blue = p.pins.pio1_1.into_swm_pin();
    let red = p.pins.pio1_2.into_swm_pin();

    // Configure the LED pins. The API tracks the state of pins at compile time,
    // to prevent any mistakes.
    let (red, _) = swm.movable_functions.t0_mat0.assign(red, &mut handle);
    let (green, _) = swm.movable_functions.t0_mat1.assign(green, &mut handle);
    let (blue, _) = swm.movable_functions.t0_mat2.assign(blue, &mut handle);

    const MAX_PERIOD: u32 = 12_000_000;
    const MIN_PERIOD: u32 = MAX_PERIOD / 12;

    let periods = (MIN_PERIOD..MAX_PERIOD).step_by(MIN_PERIOD as usize);

    let mut ctimer = p
        .CTIMER0
        .enable(MAX_PERIOD, 0, &mut syscon.handle)
        .attach(red)
        .attach(green)
        .attach(blue);

    loop {
        for period in periods.clone().rev() {
            ctimer.set_period(period);

            ctimer
                .try_set_duty(&Channels123::Channel1, period / 8)
                .unwrap();
            ctimer
                .try_set_duty(&Channels123::Channel2, period / 4)
                .unwrap();
            ctimer
                .try_set_duty(&Channels123::Channel3, period / 2)
                .unwrap();

            delay.delay_ms(period / 12_000);
        }
        for period in periods.clone() {
            ctimer.set_period(period);

            ctimer
                .try_set_duty(&Channels123::Channel1, period / 8)
                .unwrap();
            ctimer
                .try_set_duty(&Channels123::Channel2, period / 4)
                .unwrap();
            ctimer
                .try_set_duty(&Channels123::Channel3, period / 2)
                .unwrap();

            delay.delay_ms(period / 12_000);
        }
    }
}
