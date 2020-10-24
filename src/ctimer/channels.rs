//! Contains types related to CTIMER PWM channels

use core::marker::PhantomData;

use embedded_hal::PwmPin;

use crate::{
    init_state::Enabled,
    pac::{
        ctimer0::{MR, MSR},
        CTIMER0,
    },
    reg_proxy::RegProxy,
    swm,
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

macro_rules! channels {
    (
        $(
            $channel:ident:
                $field: ident,
                $id:expr,
                $output:ident,
                $state:ident;
        )*
    ) => {
        /// Contains all CTIMER PWM channels
        ///
        /// Can be accessed via `CTIMER`.
        #[allow(missing_docs)]
        pub struct Channels<PeripheralState, $($state,)*> {
            $(pub $field: Channel<$channel, PeripheralState, $state>,)*
        }

        impl<PeripheralState, $($state,)*>
            Channels<PeripheralState, $($state,)*>
        {
            pub(super) fn new() -> Self {
                Self {
                    $($field: Channel::new(),)*
                }
            }
        }

        $(
            /// Identifies a CTIMER PWM channel
            pub struct $channel;

            impl private::Sealed for $channel {}

            impl Trait for $channel {
                const ID: u8 = $id;
                type Output = swm::$output;
            }
        )*
    };
}

channels! {
    Channel1: channel1, 0, T0_MAT0, State1;
    Channel2: channel2, 1, T0_MAT1, State2;
    Channel3: channel3, 2, T0_MAT2, State3;
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

mod private {
    pub trait Sealed {}
}

reg!(MR, [MR; 4], CTIMER0, mr);
reg!(MSR, [MSR; 4], CTIMER0, msr);
