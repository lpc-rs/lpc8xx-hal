#![no_std]

use embedded_hal as hal;
#[cfg(feature = "82x")]
pub use lpc82x_pac as raw;

#[cfg(feature = "845")]
pub use lpc845_pac as raw;

pub mod clock;
#[cfg(feature = "82x")]
pub mod gpio;
#[cfg(feature = "82x")]
pub mod swm;
#[cfg(feature = "82x")]
pub mod syscon;
#[macro_use]
pub(crate) mod reg_proxy;

pub mod prelude {
    pub use crate::clock::{
        Enabled as _lpc82x_hal_clock_Enabled, Frequency as _lpc82x_hal_clock_Frequency,
    };
    pub use crate::hal::prelude::*;

}

/// Contains types that encode the state of hardware initialization
///
/// The types in this module are used by structs representing peripherals or
/// other hardware components, to encode the initialization state of the
/// underlying hardware as part of the type.
pub mod init_state {
    /// Indicates that the hardware component is enabled
    ///
    /// This usually indicates that the hardware has been initialized and can be
    /// used for its intended purpose. Contains an optional payload that APIs
    /// can use to keep data that is only available while enabled.
    pub struct Enabled<T = ()>(pub T);

    /// Indicates that the hardware component is disabled
    pub struct Disabled;
}
