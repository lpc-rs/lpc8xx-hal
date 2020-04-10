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
//! let (pwm_channel,_, _ ) =
//!     p.CTIMER0.start_pwm(256, 0, &mut syscon.handle);
//!
//! let pwm_output = p.pins.pio1_2.into_swm_pin();
//!
//! let (pwm_output, _) = swm.movable_functions.t0_mat0.assign(
//!     pwm_output,
//!     &mut swm_handle,
//! );
//!
//! let mut pwm_pin = pwm_channel.attach(pwm_output);
//! loop {
//!     for i in 0..pwm_pin.get_max_duty() {
//!         delay.delay_ms(4_u8);
//!         pwm_pin.set_duty(i);
//!     }
//! }
//! ```

use crate::{
    pac::{
        ctimer0::{MR, MSR},
        CTIMER0,
    },
    pins,
    reg_proxy::RegProxy,
    swm::{self, T0_MAT0, T0_MAT1, T0_MAT2},
    syscon,
};

use core::marker::PhantomData;
use embedded_hal::PwmPin;

/// Interface to a CTimer peripheral
///
/// Controls the CTimer.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct CTimer {
    ct: CTIMER0,
}

/// A detached [`CTimerPwmPin`]
///
/// Use `attach` to assign an output to it.
///
/// [`CTimerPwmPin`]: struct.CTimerPwmPin.html
pub struct DetachedPwmPin<CTOutput> {
    number: u8,
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    output: PhantomData<CTOutput>,
}

/// Represents a pwm channel assigned to an output pin
///
/// # `embedded-hal` traits
/// - [`embedded_hal::PwmPin`]
///
/// [`embedded_hal::PwmPin`]: #impl-PwmPin
pub struct CTimerPwmPin {
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    number: u8,
}

impl CTimer {
    pub(crate) fn new(ct: CTIMER0) -> Self {
        Self { ct }
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
    ) -> (
        DetachedPwmPin<T0_MAT0>,
        DetachedPwmPin<T0_MAT1>,
        DetachedPwmPin<T0_MAT2>,
    ) {
        syscon.enable_clock(&self.ct);
        unsafe { self.ct.pr.write(|w| w.prval().bits(prescaler)) };
        // Use MAT3 to reset the counter
        unsafe { self.ct.mr[3].write(|w| w.match_().bits(period)) };
        self.ct.mcr.write(|w| {
            w.mr3r().set_bit();
            // Use shadow registers for the pwm output matches
            w.mr0rl().set_bit();
            w.mr1rl().set_bit();
            w.mr2rl().set_bit()
        });

        self.ct.pwmc.write(|w| {
            w.pwmen0().set_bit();
            w.pwmen1().set_bit();
            w.pwmen2().set_bit()
        });

        // Start the timer
        self.ct.tcr.write(|w| w.cen().set_bit());
        (
            DetachedPwmPin {
                number: 0,
                mr: RegProxy::new(),
                msr: RegProxy::new(),
                output: PhantomData {},
            },
            DetachedPwmPin {
                number: 1,
                mr: RegProxy::new(),
                msr: RegProxy::new(),
                output: PhantomData {},
            },
            DetachedPwmPin {
                number: 2,
                mr: RegProxy::new(),
                msr: RegProxy::new(),
                output: PhantomData {},
            },
        )
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
        self.ct
    }
}

impl<CTOutput> DetachedPwmPin<CTOutput> {
    /// Assigns a pin to a `DetachedPwmPin`,
    /// allowing it to be used as a pwm output
    pub fn attach<PWM>(
        self,
        _: swm::Function<CTOutput, swm::state::Assigned<PWM>>,
    ) -> CTimerPwmPin
    where
        PWM: pins::Trait,
    {
        CTimerPwmPin {
            mr: self.mr,
            msr: self.msr,
            number: self.number,
        }
    }
}

impl PwmPin for CTimerPwmPin {
    type Duty = u32;
    /// The behaviour of `enable` is implementation defined and does nothing in
    /// this implementation
    fn enable(&mut self) {}

    /// The behaviour of `disable` is implementation defined and does nothing in
    /// this implementation
    // Accessing pwmc would require some kind of lock, which is inconvenient
    // and would involve a hidden `CriticalSection`
    fn disable(&mut self) {}

    /// Returns the current duty cycle
    fn get_duty(&self) -> Self::Duty {
        self.msr[self.number as usize].read().match_shadow().bits()
    }

    /// Returns the maximum duty cycle value
    fn get_max_duty(&self) -> Self::Duty {
        self.mr[3].read().match_().bits()
    }

    /// Sets a new duty cycle
    fn set_duty(&mut self, duty: Self::Duty) {
        unsafe {
            self.msr[self.number as usize]
                .write(|w| w.match_shadow().bits(duty))
        };
    }
}

reg!(MR, [MR; 4], CTIMER0, mr);
reg!(MSR, [MSR; 4], CTIMER0, msr);
