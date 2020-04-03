use core::marker::PhantomData;

use crate::pins::{self, Pin};

use super::{
    assignment::{AssignFunction, UnassignFunction},
    function_kind::FunctionKind,
    handle::Handle,
    state::{Assigned, State, Unassigned},
};

/// A fixed or movable function that can be assigned to a pin
///
/// The type parameter `T` identifies the fixed or movable function that an
/// instance of `Function` controls. The other type paramter, `State`, tracks
/// whether this function is assigned to a pin, and which pin it is assigned to.
pub struct Function<T, S> {
    ty: T,
    _state: S,
}

impl<T, S> Function<T, S>
where
    S: State,
{
    pub(crate) fn new(ty: T) -> Self {
        Self {
            ty,
            _state: S::new(),
        }
    }
}

impl<T> Function<T, Unassigned> {
    /// Assign this function to a pin
    ///
    /// This method is only available if a number of requirements are met:
    /// - `Function` must be in the [`Unassigned`] state, as a function can only
    ///   be assigned to one pin.
    /// - The [`Pin`] must be in the SWM state ([`pins::state::Swm`]). See
    ///   documentation on [`Pin`] for information on pin state management.
    /// - The function must be assignable to the pin. Movable functions can be
    ///   assigned to any pin, but fixed functions can be assigned to only one
    ///   pin.
    /// - The state of the pin must allow another function of this type to be
    ///   assigned. Input functions can always be assigned, but only one output
    ///   or bidirectional function can be assigned to a given pin at any time.
    ///
    /// Code attempting to call this method while these requirement are not met,
    /// will not compile.
    ///
    /// Consumes this instance of `Function`, as well as the provided [`Pin`],
    /// and returns new instances. The returned `Function` instance will have its
    /// state set to indicate that it has been assigned to the pin. The returned
    /// [`Pin`] will have its state updated to indicate that a function of this
    /// `Function`'s type has been assigned.
    ///
    /// # Examples
    ///
    /// Assign one output and one input function to the same pin:
    ///
    /// ``` no_run
    /// use lpc8xx_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let mut syscon = p.SYSCON.split();
    /// let mut swm = p.SWM.split();
    ///
    /// #[cfg(feature = "82x")]
    /// let mut swm_handle = swm.handle;
    /// #[cfg(feature = "845")]
    /// let mut swm_handle = swm.handle.enable(&mut syscon.handle);
    ///
    /// // Assign output function to a pin
    /// let (u0_txd, pio0_0) = swm.movable_functions.u0_txd.assign(
    ///     p.pins.pio0_0.into_swm_pin(),
    ///     &mut swm_handle,
    /// );
    ///
    /// // Assign input function to the same pin
    /// let (u1_rxd, pio0_0) = swm.movable_functions.u1_rxd.assign(
    ///     pio0_0,
    ///     &mut swm_handle,
    /// );
    /// ```
    ///
    /// [`Unassigned`]: state/struct.Unassigned.html
    pub fn assign<P, S>(
        mut self,
        mut pin: Pin<P, S>,
        swm: &mut Handle,
    ) -> (
        Function<T, Assigned<P>>,
        <Pin<P, S> as AssignFunction<T, T::Kind>>::Assigned,
    )
    where
        T: FunctionTrait<P>,
        P: pins::Trait,
        S: pins::State,
        Pin<P, S>: AssignFunction<T, T::Kind>,
    {
        self.ty.assign(&mut pin.ty, swm);

        let function = Function {
            ty: self.ty,
            _state: Assigned(PhantomData),
        };

        (function, pin.assign())
    }
}

impl<T, P> Function<T, Assigned<P>> {
    /// Unassign this function from a pin
    ///
    /// This method is only available if a number of requirements are met:
    /// - The function must be assigned to the provided pin. This means
    ///   `Function` must be in the [`Assigned`] state, and the type parameter
    ///   of [`Assigned`] must indicate that the function is assigned to the
    ///   same pin that is provided as an argument.
    /// - The [`Pin`] must be in the SWM state ([`pins::state::Swm`]), and the
    ///   state must indicate that a function of this `Function`'s type is
    ///   currently assigned. This should always be the case, if the previous
    ///   condition is met, as it should be impossible to create inconsistent
    ///   states between `Function`s and [`Pin`]s without using `unsafe`.
    ///
    /// Code attempting to call this method while these requirement are not met,
    /// will not compile.
    ///
    /// Consumes this instance of `Function`, as well as the provided [`Pin`],
    /// and returns new instances. The returned `Function` instance will have
    /// its state set to indicate that it is no longer assigned to a pin. The
    /// returned [`Pin`] will have its state updated to indicate that one fewer
    /// function of this type is now assigned.
    ///
    /// # Examples
    ///
    /// Unassign a function that has been previously assigned to a pin:
    ///
    /// ``` no_run
    /// # use lpc8xx_hal::Peripherals;
    /// #
    /// # let p = Peripherals::take().unwrap();
    /// #
    /// # let mut swm = p.SWM.split();
    /// # let mut syscon = p.SYSCON.split();
    /// #
    /// # #[cfg(feature = "82x")]
    /// # let mut swm_handle = swm.handle;
    /// # #[cfg(feature = "845")]
    /// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
    /// #
    /// # // Assign output function to a pin
    /// # let (u0_txd, pio0_0) = swm.movable_functions.u0_txd.assign(
    /// #     p.pins.pio0_0.into_swm_pin(),
    /// #     &mut swm_handle,
    /// # );
    /// #
    /// // U0_TXD must have been previously assigned to the pin, or the
    /// // following code will not compile. See documentation of
    /// // `Function::assign`.
    /// let (u0_txd, pio0_0) = u0_txd.unassign(pio0_0, &mut swm_handle);
    /// ```
    ///
    /// [`Assigned`]: state/struct.Assigned.html
    pub fn unassign<S>(
        mut self,
        mut pin: Pin<P, S>,
        swm: &mut Handle,
    ) -> (
        Function<T, Unassigned>,
        <Pin<P, S> as UnassignFunction<T, T::Kind>>::Unassigned,
    )
    where
        T: FunctionTrait<P>,
        P: pins::Trait,
        S: pins::State,
        Pin<P, S>: UnassignFunction<T, T::Kind>,
    {
        self.ty.unassign(&mut pin.ty, swm);

        let function = Function {
            ty: self.ty,
            _state: Unassigned,
        };

        (function, pin.unassign())
    }
}

/// Implemented for all fixed and movable functions
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer [`Function::assign`] and [`Function::unassign`] for the public
/// API that uses this trait.
pub trait FunctionTrait<P: pins::Trait> {
    /// Whether this is an input or output function
    ///
    /// There are also bidirectional functions, but for our purposes, they are
    /// treated as output functions.
    type Kind: FunctionKind;

    /// Internal method to assign a function to a pin
    fn assign(&mut self, pin: &mut P, swm: &mut Handle);

    /// Internal method to unassign a function from a pin
    fn unassign(&mut self, pin: &mut P, swm: &mut Handle);
}
