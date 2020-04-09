//! Contains types that indicate the state of fixed or movable functions

use core::marker::PhantomData;

/// Implemented by types that indicate the state of SWM functions
///
/// This trait is implemented by types that indicate the state of SWM functions.
/// It exists only to document which types those are. Users should not need to
/// implement this trait, nor use it directly.
pub trait State {
    /// Returns an instance of the state
    ///
    /// This method is intended for internal use. Any changes to this method
    /// won't be considered breaking changes.
    fn new() -> Self;
}

/// Indicates that a function is unassigned
pub struct Unassigned;

impl State for Unassigned {
    fn new() -> Self {
        Unassigned
    }
}

/// Indicates that a function is assigned to a pin
pub struct Assigned<Pin>(pub(crate) PhantomData<Pin>);

impl<Pin> State for Assigned<Pin> {
    fn new() -> Self {
        Assigned(PhantomData)
    }
}
