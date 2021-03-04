use crate::{
    pac::{self, mrt0::CHANNEL},
    reg_proxy::Reg,
};

use super::Channel;

/// Implemented for types that identify MRT channels
pub trait Trait: Reg<Target = CHANNEL> + sealed::Sealed {}

macro_rules! channels {
    ($($channel:ident, $field:ident, $index:expr;)*) => {
        /// Provides access to the MRT channels
        pub struct Channels {
            $(
                #[allow(missing_docs)]
                pub $field: Channel<$channel>,
            )*
        }

        impl Channels {
            pub(super) fn new() -> Self {
                Self {
                    $($field: Channel::new(),)*
                }
            }
        }

        $(
            /// Represents one of the MRT channels
            ///
            /// Used as a type parameter for [`Channel`].
            pub struct $channel;

            reg_cluster_array!($channel, CHANNEL, pac::MRT0, channel, $index);

            impl sealed::Sealed for $channel {}
            impl Trait for $channel {}
        )*
    }
}

channels!(
    MRT0, mrt0, 0;
    MRT1, mrt1, 1;
    MRT2, mrt2, 2;
    MRT3, mrt3, 3;
);

mod sealed {
    pub trait Sealed {}
}
