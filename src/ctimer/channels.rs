//! Contains types related to CTIMER PWM channels

use core::marker::PhantomData;

use embedded_hal::PwmPin;

use crate::{
    pac::ctimer0::{MR, MSR},
    pins,
    reg_proxy::RegProxy,
    swm,
};

/// A CTIMER PWM channel
pub struct Channel<T, State> {
    mr: RegProxy<MR>,
    msr: RegProxy<MSR>,
    channel: PhantomData<T>,
    _state: PhantomData<State>,
}

impl<T, State> Channel<T, State> {
    pub(super) fn new() -> Self {
        Self {
            mr: RegProxy::new(),
            msr: RegProxy::new(),
            channel: PhantomData,
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
    pub fn attach<Pin>(
        self,
        _: swm::Function<T::Output, swm::state::Assigned<Pin>>,
    ) -> Channel<T, state::Attached>
    where
        Pin: pins::Trait,
    {
        Channel {
            mr: self.mr,
            msr: self.msr,
            channel: self.channel,
            _state: PhantomData,
        }
    }
}

impl<T> PwmPin for Channel<T, state::Attached>
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

macro_rules! channels {
    (
        $(
            $channel:ident: $id:expr, $output:ident;
        )*
    ) => {
        $(
            /// Identifies a CTIMER PWM channel
            pub struct $channel;

            impl Trait for $channel {
                const ID: u8 = $id;
                type Output = swm::$output;
            }
        )*
    };
}

channels! {
    Channel1: 0, T0_MAT0;
    Channel2: 1, T0_MAT1;
    Channel3: 2, T0_MAT2;
}

/// Implemented for all CTIMER PWM channels
pub trait Trait {
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
