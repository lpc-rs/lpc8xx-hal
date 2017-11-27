//! Common types for system clocks
//!
//! This module defines types that are helpful for working with system clocks.


/// Represents a number of ticks of a given clock
///
/// This struct is used to represent an amount of time, a duration, but in a
/// low-level way that hardware peripherals can understand and handle. It is
/// meant to be a common denominator that higher-level time APIs can be built on
/// top of.
pub struct Ticks<'clock, C: 'clock> {
    /// The number of ticks
    pub value: u32,

    /// Reference to the clock
    ///
    /// This reference exists to prevent any configuration of the clock that
    /// would invalidate this struct, as configuration would require a mutable
    /// reference to the clock, presumably.
    ///
    /// The prime example of such invalidating configuration that this reference
    /// would prevent is a change of the clock frequency. If, for example, an
    /// instance of this struct is intended to represent a duration of 10ms,
    /// this duration would change, if the clock frequency were to be changed.
    pub clock: &'clock C,
}

impl<'clock, Clock> Clone for Ticks<'clock, Clock> {
    fn clone(&self) -> Self {
        Ticks {
            value: self.value,
            clock: self.clock,
        }
    }
}

impl<'clock, Clock> Copy for Ticks<'clock, Clock> {}


/// Implemented by clocks that can return a frequency
///
/// Implementations of this trait might be very simple, for clocks that run at
/// one specific frequency. Or they might be more complex, for clocks whose
/// frequency can be configured.
///
/// Some clocks might not have an implementation of this trait at all. An
/// example of this might be a type that represents an external clock that is
/// fed into the microcontroller via a pin.
pub trait Frequency {
    /// The frequency of the clock in Hz
    ///
    /// This method must never return `0`.
    fn hz(&self) -> u32;
}


/// Marker trait that identifies a clock as currently enabled
///
/// A clock that is always enabled can implement this trait unconditionally.
/// Clocks that can be disabled can use an additional type or type parameter to
/// implement this trait, as shown in the following example:
///
/// ``` rust
/// use lpc82x_hal::clock;
///
///
/// struct MyClock<State = clock::state::Disabled> {
///     _state: State,
/// }
///
/// impl MyClock {
///     /// Consume the instance with disabled state, return one with enabled
///     /// state
///     pub fn enable(self) -> MyClock<clock::state::Enabled> {
///         // Enable the clock
///         // ...
///
///         MyClock {
///             _state: clock::state::Enabled,
///         }
///     }
/// }
///
/// impl clock::Enabled for MyClock<clock::state::Enabled> {}
/// ```
pub trait Enabled {}


/// Contains types that mark the state of a given clock instance
pub mod state {
    /// Implemented by types that indicate a clock state
    ///
    /// This trait can be used as a trait bound for generic type parameters that
    /// indicate a clock state. This can be done for documentation purposes, to
    /// make it clear from a clock's reference documentation which states it
    /// can have.
    pub trait ClockState {}

    /// Marks the clock as being disabled
    pub struct Disabled;
    impl ClockState for Disabled {}

    /// Marks the clock as being enabled
    pub struct Enabled;
    impl ClockState for Enabled {}
}
