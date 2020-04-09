//! Contains types that indicate pin states
//!
//! Please refer to [`Pin`] for documentation about how these types are used.
//!
//! [`Pin`]: ../struct.Pins.html

use core::marker::PhantomData;

/// Implemented by types that indicate pin state
///
/// [`Pin`] uses this type as a trait bound for the type parameter that
/// indicates the pin's state. This is done for the purpose of
/// documentation, to show which states a pin can be in. Other than that,
/// this trait should not be relevant to users of this crate.
///
/// [`Pin`]: ../struct.Pin.html
pub trait State {}

/// Marks a [`Pin`] as being unused
///
/// [`Pin`]: ../struct.Pin.html
pub struct Unused;

impl Unused {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl State for Unused {}

/// Marks a [`Pin`]  as being assigned to the analog-to-digital converter
///
/// [`Pin`]: ../struct.Pin.html
pub struct Analog;

impl State for Analog {}

/// Marks a [`Pin`]  as being available for switch matrix function assigment
///
/// The type parameters of this struct track whether output and input
/// functions have been assigned to a pin:
///
/// - `Output` tracks whether an output function has been assigned. Zero or
///   one output functions can be assigned to a pin at a time.
/// - `Inputs` tracks the number of assigned input functions. Any number of
///   input functions can be assigned to a pin at the same time.
///
/// Both type parameters use nested tuples to count the number of assigned
/// functions. The empty tuple, `()`, represents zero assigned functions,
/// the empty tuple nested in another tuple, `((),)`, represents one
/// function being assigned, `(((),))` represents two assigned functions,
/// and so forth. This is a bit of a hack, of course, but it should do until
/// [const generics] become available.
///
/// [const generics]: https://github.com/rust-lang/rust/issues/44580
/// [`Pin`]: ../struct.Pin.html
pub struct Swm<Output, Inputs>(
    pub(crate) PhantomData<Output>,
    pub(crate) PhantomData<Inputs>,
);

impl<Output, Inputs> Swm<Output, Inputs> {
    pub(crate) const fn new() -> Self {
        Swm(PhantomData, PhantomData)
    }
}

impl<Output, Inputs> State for Swm<Output, Inputs> {}
