//! Clock configuration for the peripherals
#[cfg(feature = "82x")]
mod clocksource_82x;
#[cfg(feature = "82x")]
pub use clocksource_82x::*;
#[cfg(feature = "845")]
mod clocksource_845;
#[cfg(feature = "845")]
pub use clocksource_845::*;
