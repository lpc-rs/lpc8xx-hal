use core::marker::PhantomData;

/// Defines the clock configuration for a USART instance
pub struct Clock<Clock> {
    pub(crate) psc: u16,
    pub(crate) osrval: u8,
    pub(crate) _clock: PhantomData<Clock>,
}

// `impl` blocks are defined in the target-specific sub-modules of
// `syscon::clock_source`.
