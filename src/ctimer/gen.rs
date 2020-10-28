use crate::swm;

use super::channel::{self, Channel};

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

            impl channel::private::Sealed for $channel {}

            impl channel::Trait for $channel {
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
