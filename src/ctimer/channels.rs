//! Contains types related to CTIMER PWM channels

use core::marker::PhantomData;

use crate::{
    pac::ctimer0::{MR, MSR},
    pins,
    reg_proxy::RegProxy,
    swm,
};

/// A CTIMER PWM channel
pub struct Channel<T, State> {
    number: u8,
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    _channel: PhantomData<T>,
    _state: PhantomData<State>,
}

impl<T, State> Channel<T, State> {
    pub(super) fn new(number: u8) -> Self {
        Self {
            number,
            mr: RegProxy::new(),
            msr: RegProxy::new(),
            _channel: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<T> Channel<T, state::Detached>
where
    T: Trait,
{
    /// Assigns a pin to a `DetachedPwmPin`,
    /// allowing it to be used as a pwm output
    pub fn attach<PWM>(
        self,
        _: swm::Function<T::Output, swm::state::Assigned<PWM>>,
    ) -> super::CTimerPwmPin
    where
        PWM: pins::Trait,
    {
        super::CTimerPwmPin {
            mr: self.mr,
            msr: self.msr,
            number: self.number,
        }
    }
}

macro_rules! channels {
    (
        $(
            $channel:ident: $output:ident;
        )*
    ) => {
        $(
            /// Identifies a CTIMER PWM channel
            pub struct $channel;

            impl Trait for $channel {
                type Output = swm::$output;
            }
        )*
    };
}

channels! {
    Channel1: T0_MAT0;
    Channel2: T0_MAT1;
    Channel3: T0_MAT2;
}

/// Implemented for all CTIMER PWM channels
pub trait Trait {
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
}
