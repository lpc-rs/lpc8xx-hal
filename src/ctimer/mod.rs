//! API for the CTimer peripheral
//!
//! Currently, only PWM output functionality is implemented.
//!
//! # Example
//!
//! ```no_run
//! use lpc8xx_hal::{
//!     delay::Delay,
//!     prelude::*,
//!     Peripherals,
//!     pac::CorePeripherals,
//! };
//!
//! let cp = CorePeripherals::take().unwrap();
//! let p = Peripherals::take().unwrap();
//!
//! let swm = p.SWM.split();
//! let mut delay = Delay::new(cp.SYST);
//! let mut syscon = p.SYSCON.split();
//!
//! let mut swm_handle = swm.handle.enable(&mut syscon.handle);
//!
//! let pwm_output = p.pins.pio1_2.into_swm_pin();
//!
//! let (pwm_output, _) = swm.movable_functions.t0_mat0.assign(
//!     pwm_output,
//!     &mut swm_handle,
//! );
//!
//! // Use 8 bit pwm
//! let ctimer = p.CTIMER0
//!     .enable(256, 0, &mut syscon.handle)
//!     .attach(pwm_output);
//!
//! let mut pwm_pin = ctimer.channels.channel1;
//! loop {
//!     for i in 0..pwm_pin.get_max_duty() {
//!         delay.delay_ms(4_u8);
//!         pwm_pin.set_duty(i);
//!     }
//! }
//! ```

pub mod channel;

mod gen;
mod peripheral;

pub use self::{
    channel::Channel,
    gen::*,
    peripheral::{Channels1, Channels12, Channels123, CTIMER},
};
