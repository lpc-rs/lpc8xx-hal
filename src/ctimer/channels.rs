//! Contains types related to CTIMER PWM channels

use crate::swm;

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
