//! Contains types related to CTIMER PWM channels

use core::{convert::Infallible, marker::PhantomData};

use embedded_hal::PwmPin;
use embedded_hal_alpha::pwm::PwmPin as PwmPinAlpha;

use crate::{
    init_state::Enabled,
    pac::{
        ctimer0::{MR, MSR},
        CTIMER0,
    },
    reg_proxy::RegProxy,
};

use self::state::Attached;

/// A CTIMER PWM channel
pub struct Channel<T, PeripheralState, State> {
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    channel: PhantomData<T>,
    peripheral_state: PhantomData<PeripheralState>,
    _state: PhantomData<State>,
}

impl<T, PeripheralState, State> Channel<T, PeripheralState, State> {
    pub(super) fn new() -> Self {
        Self {
            mr: RegProxy::new(),
            msr: RegProxy::new(),
            channel: PhantomData,
            peripheral_state: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<T> PwmPin for Channel<T, Enabled, Attached>
where
    T: Trait,
{
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
        self.msr[T::ID as usize].read().match_shadow().bits()
    }

    /// Returns the maximum duty cycle value
    fn get_max_duty(&self) -> Self::Duty {
        self.mr[3].read().match_().bits()
    }

    /// Sets a new duty cycle
    fn set_duty(&mut self, duty: Self::Duty) {
        unsafe {
            self.msr[T::ID as usize].write(|w| w.match_shadow().bits(duty))
        };
    }
}

impl<T> PwmPinAlpha for Channel<T, Enabled, Attached>
where
    T: Trait,
{
    type Error = Infallible;
    type Duty = u32;

    /// The behaviour of `enable` is implementation defined and does nothing in
    /// this implementation
    fn try_enable(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// The behaviour of `disable` is implementation defined and does nothing in
    /// this implementation
    // Accessing pwmc would require some kind of lock, which is inconvenient
    // and would involve a hidden `CriticalSection`
    fn try_disable(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Returns the current duty cycle
    fn try_get_duty(&self) -> Result<Self::Duty, Self::Error> {
        Ok(self.msr[T::ID as usize].read().match_shadow().bits())
    }

    /// Returns the maximum duty cycle value
    fn try_get_max_duty(&self) -> Result<Self::Duty, Self::Error> {
        Ok(self.mr[3].read().match_().bits())
    }

    /// Sets a new duty cycle
    fn try_set_duty(&mut self, duty: Self::Duty) -> Result<(), Self::Error> {
        unsafe {
            Ok(self.msr[T::ID as usize].write(|w| w.match_shadow().bits(duty)))
        }
    }
}

/// Implemented for all CTIMER PWM channels
pub trait Trait: private::Sealed {
    /// Identifies the channel
    const ID: u8;

    /// The SWM function that needs to be assigned to this channels output pin
    type Output;
}

/// Contains types that indicate which state a channel is in
pub mod state {
    /// Indicates that a channel is detached
    ///
    /// Detached channels don't have an output function assigned and can't be
    /// used for PWM output.
    pub struct Detached;

    /// Indicates that a channel is attached
    pub struct Attached;
}

pub(super) mod private {
    pub trait Sealed {}
}

reg!(MR, [MR; 4], CTIMER0, mr);
reg!(MSR, [MSR; 4], CTIMER0, msr);
