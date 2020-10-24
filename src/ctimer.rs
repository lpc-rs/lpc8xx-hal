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
//! // Use 8 bit pwm
//! let channels = p.CTIMER0.start_pwm(256, 0, &mut syscon.handle);
//!
//! let pwm_output = p.pins.pio1_2.into_swm_pin();
//!
//! let (pwm_output, _) = swm.movable_functions.t0_mat0.assign(
//!     pwm_output,
//!     &mut swm_handle,
//! );
//!
//! let mut pwm_pin = channels.channel1.attach(pwm_output);
//! loop {
//!     for i in 0..pwm_pin.get_max_duty() {
//!         delay.delay_ms(4_u8);
//!         pwm_pin.set_duty(i);
//!     }
//! }
//! ```

pub mod channels;

use crate::{
    pac::{
        ctimer0::{MR, MSR},
        CTIMER0,
    },
    syscon,
};

use self::channels::{state::Detached, Channels};

/// Interface to a CTimer peripheral
///
/// Controls the CTimer.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct CTIMER {
    inner: CTIMER0,
}

impl CTIMER {
    pub(crate) fn new(ct: CTIMER0) -> Self {
        Self { inner: ct }
    }

    /// Start the PWM timer, with a predefined period and prescaler
    ///
    /// The `period` sets resolution of the pwm and is returned with
    /// `get_max_duty`.
    pub fn start_pwm(
        self,
        period: u32,
        prescaler: u32,
        syscon: &mut syscon::Handle,
    ) -> Channels<Detached, Detached, Detached> {
        syscon.enable_clock(&self.inner);
        unsafe { self.inner.pr.write(|w| w.prval().bits(prescaler)) };
        // Use MAT3 to reset the counter
        unsafe { self.inner.mr[3].write(|w| w.match_().bits(period)) };
        self.inner.mcr.write(|w| {
            w.mr3r().set_bit();
            // Use shadow registers for the pwm output matches
            w.mr0rl().set_bit();
            w.mr1rl().set_bit();
            w.mr2rl().set_bit()
        });

        self.inner.pwmc.write(|w| {
            w.pwmen0().set_bit();
            w.pwmen1().set_bit();
            w.pwmen2().set_bit()
        });

        // Start the timer
        self.inner.tcr.write(|w| w.cen().set_bit());

        Channels::new()
    }

    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> CTIMER0 {
        self.inner
    }
}

reg!(MR, [MR; 4], CTIMER0, mr);
reg!(MSR, [MSR; 4], CTIMER0, msr);
