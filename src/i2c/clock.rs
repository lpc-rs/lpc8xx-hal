use core::marker::PhantomData;

/// A struct containing the clock configuration for a peripheral
pub struct I2cClock<Clock> {
    pub(crate) divval: u16,
    pub(crate) mstsclhigh: u8,
    pub(crate) mstscllow: u8,
    pub(crate) _clock: PhantomData<Clock>,
}
