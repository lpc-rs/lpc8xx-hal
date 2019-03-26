#![no_std]

use embedded_hal as hal;
#[cfg(feature = "82x")]
pub use lpc82x_pac as raw;

#[cfg(feature = "845")]
pub use lpc845_pac as raw;

pub mod clock;
pub mod gpio;
pub mod swm;
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

// Provide common peripheral names
// When in doubt, use the names from the new svd files
mod raw_compat {
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::gpio;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::gpio_port as gpio;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::ACOMP;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::ADC as ADC0;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::ADC0;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::CMP as ACOMP;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::DMA as DMA0;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::DMA0;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::FLASHCTRL as FLASH_CTRL;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::FLASH_CTRL;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::GPIO;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::GPIO_PORT as GPIO;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::MRT as MRT0;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::MRT0;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::SCT as SCT0;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::SCT0;
    #[cfg(feature = "82x")]
    pub(crate) use crate::raw::SWM as SWM0;
    #[cfg(feature = "845")]
    pub(crate) use crate::raw::SWM0;
}
