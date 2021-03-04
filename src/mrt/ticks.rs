use core::convert::TryFrom;

use embedded_time::duration::{
    Microseconds, Milliseconds, Nanoseconds, Seconds,
};

use super::MAX_VALUE;

/// Represents a number of ticks of the MRT timer
///
/// `Ticks` has various `From` and `TryFrom` implementations that provide
/// integration with `embedded_time` duration types. This not only provides a
/// more convenient API, it also makes it possible to use the MRT generically,
/// through the [`CountDown`] trait and a bound like
/// `Timer::Time: TryFrom<Milliseconds>`, without requiring any knowledge of the
/// timer's frequency.
///
/// However, these conversions have performance implications. For best results,
/// you should use constants for the original values that you want to convert,
/// to give the compiler a chance to perform the conversion at compile-time.
///
/// [`CountDown`]: embedded_hal::timer::CountDown
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ticks(pub(super) u32);

impl Ticks {
    /// Creates a `Tick` instance with the given number of ticks
    ///
    /// This method is provided as a fallback to avoid performance overhead, in
    /// case the user knows that `value` fits within `MAX_VALUE`, but the
    /// compiler can't perform the necessary optimization. Please use any of the
    /// `From` or `TryFrom` implementations instead, if you can afford it.
    ///
    /// # Safety
    ///
    /// The user must guarantee that `value <= MAX_VALUE`.
    pub unsafe fn from_u32(value: u32) -> Self {
        Self(value)
    }

    /// Returns the number of ticks of this `Tick` instance.
    /// You may also use the `Into` implementations instead.
    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for Ticks {
    type Error = TickConversionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > MAX_VALUE.0 {
            return Err(TickConversionError);
        }

        Ok(Self(value))
    }
}

impl From<Ticks> for u32 {
    fn from(ticks: Ticks) -> Self {
        ticks.0
    }
}

// Eventually, `Ticks` will need a const-generic argument or something, but as
// long as everything is hardcoded to 12 MHz, the following will do.

impl From<Nanoseconds> for Ticks {
    fn from(value: Nanoseconds) -> Self {
        // This can't possibly fail:
        // - The multiplication can't overflow after converting to `u64`.
        // - After the division, the value is guaranteed to fit into the `u32`
        //   again.
        // - The maximum possible `value` leads to a result that is smaller than
        //   `MAX_VALUE`.
        Self((value.0 as u64 * 12 / 1_000) as u32)
    }
}

impl TryFrom<Microseconds> for Ticks {
    type Error = TickConversionError;

    fn try_from(value: Microseconds) -> Result<Self, Self::Error> {
        let value = value.0.checked_mul(12).ok_or(TickConversionError)?;
        Self::try_from(value)
    }
}

impl TryFrom<Milliseconds> for Ticks {
    type Error = TickConversionError;

    fn try_from(value: Milliseconds) -> Result<Self, Self::Error> {
        let value = value.0.checked_mul(12_000).ok_or(TickConversionError)?;
        Self::try_from(value)
    }
}

impl TryFrom<Seconds> for Ticks {
    type Error = TickConversionError;

    fn try_from(value: Seconds) -> Result<Self, Self::Error> {
        let value =
            value.0.checked_mul(12_000_000).ok_or(TickConversionError)?;
        Self::try_from(value)
    }
}

#[derive(Debug, Eq, PartialEq)]
/// Indicates that a conversion to [`Tick`] failed
///
/// This is the case when the resulting value is larger than [`MAX_VALUE`].
pub struct TickConversionError;
