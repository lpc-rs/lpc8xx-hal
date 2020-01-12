//! PwmPin implementation based on CTimer
//!
//!

// Use the timer as one 32 bit timer
// Don't implement prescaling, since it isn't needed
// Currently only implemented for lpc845
use crate::pac::ctimer0::{MR, MSR};
use crate::pac::CTIMER0;
use crate::reg_proxy::RegProxy;
use crate::swm::{self, PinTrait, T0_MAT0, T0_MAT1, T0_MAT2};
use crate::syscon;
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

/// An unconfigured PwmPin
pub struct UnconfiguredPwmPin<CTOutput> {
    number: u8,
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    output: PhantomData<CTOutput>,
}

/// TODO
pub struct CTimerPwmPin {
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    number: u8,
}

impl CTimer {
    pub(crate) fn new(ct: CTIMER0) -> Self {
        Self { ct }
    }

    /// TODO
    pub fn start_pwm(
        self,
        period: u32,
        prescaler: u32,
        syscon: &mut syscon::Handle,
    ) -> (
        UnconfiguredPwmPin<T0_MAT0>,
        UnconfiguredPwmPin<T0_MAT1>,
        UnconfiguredPwmPin<T0_MAT2>,
    ) {
        syscon.enable_clock(&self.ct);
        unsafe { self.ct.pr.write(|w| w.prval().bits(prescaler)) };
        // Use MAT3  to reset the counter
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
            UnconfiguredPwmPin {
                number: 0,
                mr: RegProxy::new(),
                msr: RegProxy::new(),
                output: PhantomData {},
            },
            UnconfiguredPwmPin {
                number: 1,
                mr: RegProxy::new(),
                msr: RegProxy::new(),
                output: PhantomData {},
            },
            UnconfiguredPwmPin {
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

impl<CTOutput> UnconfiguredPwmPin<CTOutput> {
    /// Assings a pin to an `UnconfiguredPwmOutput`,
    /// allowing it to be used as a pwm output
    pub fn configure<PWM>(
        self,
        _: swm::Function<CTOutput, swm::state::Assigned<PWM>>,
    ) -> CTimerPwmPin
    where
        PWM: PinTrait,
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
    fn enable(&mut self) {
        // TODO
    }

    fn disable(&mut self) {
        // TODO
    }

    fn get_duty(&self) -> Self::Duty {
        self.msr[self.number as usize].read().match_shadow().bits()
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.mr[3].read().match_().bits()
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        unsafe {
            self.msr[self.number as usize]
                .write(|w| w.match_shadow().bits(duty))
        };
    }
}

reg!(MR, [MR; 4], CTIMER0, mr);
reg!(MSR, [MSR; 4], CTIMER0, msr);
