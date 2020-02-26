//! APIs for the switch matrix (SWM)
//!
//! The entry point to this API is [`SWM`]. Please refer to [`SWM`]'s
//! documentation for additional information.
//!
//! The switch matrix is described in the user manual, chapter 7.

use core::marker::PhantomData;

use crate::{
    init_state, pac,
    pins::{
        self,
        pin_state::{self, PinState},
        Pin, PinTrait,
    },
    syscon,
};

/// Entry point to the switch matrix (SWM) API
///
/// The SWM API is split into multiple parts, which are all available through
/// [`swm::Parts`]. You can use [`SWM::split`] to gain access to [`swm::Parts`].
///
/// You can also use this struct to gain access to the raw peripheral using
/// [`SWM::free`]. This is the main reason this struct exists, as it's no longer
/// possible to do this after the API has been split.
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`swm::Parts`]: struct.Parts.html
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct SWM<State = init_state::Enabled> {
    swm: pac::SWM0,
    state: State,
}

impl SWM<init_state::Disabled> {
    /// Create a disabled SWM peripheral
    ///
    /// This method creates an `SWM` instance that it assumes is in the
    /// [`Disabled`] state. As it's only possible to enable a [`Disabled`] `SWM`
    /// instance, it's also safe to pass an already [`Enabled`] instance.
    ///
    /// # Safety
    ///
    /// This method creates an `SWM` instance that it assumes is in the default
    /// state. It's up to the caller to verify this assumption.
    ///
    /// [`Disabled`]: ../init_state/struct.Enabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub unsafe fn new(swm: pac::SWM0) -> Self {
        SWM {
            swm,
            state: init_state::Disabled,
        }
    }
}

impl SWM<init_state::Enabled> {
    /// Create a enabled SWM peripheral
    ///
    /// # Safety
    ///
    /// This method creates an `SWM` instance that it assumes is already in the
    /// default [`Enabled`] state. It's up to the caller to verify this assumption.
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub unsafe fn new_enabled(swm: pac::SWM0) -> Self {
        SWM {
            swm,
            state: init_state::Enabled(()),
        }
    }
}

impl<STATE> SWM<STATE> {
    /// Splits the SWM API into its component parts
    ///
    /// This is the regular way to access the SWM API. It exists as an explicit
    /// step, as it's no longer possible to gain access to the raw peripheral
    /// using [`SWM::free`] after you've called this method.
    pub fn split(self) -> Parts<STATE> {
        Parts {
            handle: Handle::new(self.swm, self.state),
            movable_functions: MovableFunctions::new(),
            fixed_functions: FixedFunctions::new(),
        }
    }

    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> pac::SWM0 {
        self.swm
    }
}

/// The main API for the switch matrix (SWM)
///
/// Provides access to all types that make up the SWM API. Please refer to the
/// [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts<STATE> {
    /// Handle to the switch matrix
    pub handle: Handle<STATE>,

    /// Movable functions
    pub movable_functions: MovableFunctions,

    /// Fixed functions
    pub fixed_functions: FixedFunctions,
}

/// Handle to the SWM peripheral
///
/// Can be used to enable and disable the switch matrix. It is also required by
/// other parts of the API to synchronize access the the underlying registers,
/// wherever this is required.
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [module documentation]: index.html
pub struct Handle<State = init_state::Enabled> {
    swm: pac::SWM0,
    _state: State,
}

impl<STATE> Handle<STATE> {
    pub(crate) fn new(swm: pac::SWM0, state: STATE) -> Self {
        Handle { swm, _state: state }
    }
}

impl Handle<init_state::Disabled> {
    /// Enable the switch matrix
    ///
    /// This method is only available, if `SWM` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `SWM` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(
        self,
        syscon: &mut syscon::Handle,
    ) -> Handle<init_state::Enabled> {
        syscon.enable_clock(&self.swm);

        Handle {
            swm: self.swm,
            _state: init_state::Enabled(()),
        }
    }
}

impl Handle<init_state::Enabled> {
    /// Disable the switch matrix
    ///
    /// The switch matrix retains it's configuration while disabled, but
    /// doesn't allow modifications
    ///
    /// This method is only available, if `SWM` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `SWM` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> Handle<init_state::Disabled> {
        syscon.disable_clock(&self.swm);

        Handle {
            swm: self.swm,
            _state: init_state::Disabled,
        }
    }
}

impl<T, F, O, Is> AssignFunction<F, Input> for Pin<T, pin_state::Swm<O, Is>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Input>,
{
    type Assigned = Pin<T, pin_state::Swm<O, (Is,)>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty: self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Is> AssignFunction<F, Output> for Pin<T, pin_state::Swm<(), Is>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Output>,
{
    type Assigned = Pin<T, pin_state::Swm<((),), Is>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty: self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, O, Is> UnassignFunction<F, Input>
    for Pin<T, pin_state::Swm<O, (Is,)>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Input>,
{
    type Unassigned = Pin<T, pin_state::Swm<O, Is>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty: self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Is> UnassignFunction<F, Output> for Pin<T, pin_state::Swm<((),), Is>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Output>,
{
    type Unassigned = Pin<T, pin_state::Swm<(), Is>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty: self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F> AssignFunction<F, Analog> for Pin<T, pin_state::Swm<(), ()>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Analog>,
{
    type Assigned = Pin<T, pin_state::Analog>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty: self.ty,
            state: pin_state::Analog,
        }
    }
}

/// A fixed or movable function that can be assigned to a pin
///
/// The type parameter `T` identifies the fixed or movable function that an
/// instance of `Function` controls. The other type paramter, `State`, tracks
/// whether this function is assigned to a pin, and which pin it is assigned to.
pub struct Function<T, State> {
    ty: T,
    _state: State,
}

impl<T> Function<T, state::Unassigned> {
    /// Assign this function to a pin
    ///
    /// This method is only available if a number of requirements are met:
    /// - `Function` must be in the [`Unassigned`] state, as a function can only
    ///   be assigned to one pin.
    /// - The [`Pin`] must be in the SWM state ([`pin_state::Swm`]). See
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
    /// use lpc82x_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let mut swm = p.SWM.split();
    ///
    /// // Assign output function to a pin
    /// let (u0_txd, pio0_0) = swm.movable_functions.u0_txd.assign(
    ///     swm.pins.pio0_0.into_swm_pin(),
    ///     &mut swm.handle,
    /// );
    ///
    /// // Assign input function to the same pin
    /// let (u1_rxd, pio0_0) = swm.movable_functions.u1_rxd.assign(
    ///     pio0_0,
    ///     &mut swm.handle,
    /// );
    /// ```
    ///
    /// [`Unassigned`]: state/struct.Unassigned.html
    pub fn assign<P, S>(
        mut self,
        mut pin: Pin<P, S>,
        swm: &mut Handle,
    ) -> (
        Function<T, state::Assigned<P>>,
        <Pin<P, S> as AssignFunction<T, T::Kind>>::Assigned,
    )
    where
        T: FunctionTrait<P>,
        P: PinTrait,
        S: PinState,
        Pin<P, S>: AssignFunction<T, T::Kind>,
    {
        self.ty.assign(&mut pin.ty, swm);

        let function = Function {
            ty: self.ty,
            _state: state::Assigned(PhantomData),
        };

        (function, pin.assign())
    }
}

impl<T, P> Function<T, state::Assigned<P>> {
    /// Unassign this function from a pin
    ///
    /// This method is only available if a number of requirements are met:
    /// - The function must be assigned to the provided pin. This means
    ///   `Function` must be in the [`Assigned`] state, and the type parameter
    ///   of [`Assigned`] must indicate that the function is assigned to the
    ///   same pin that is provided as an argument.
    /// - The [`Pin`] must be in the SWM state ([`pin_state::Swm`]), and the
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
    /// # use lpc82x_hal::Peripherals;
    /// #
    /// # let p = Peripherals::take().unwrap();
    /// #
    /// # let mut swm = p.SWM.split();
    /// #
    /// # // Assign output function to a pin
    /// # let (u0_txd, pio0_0) = swm.movable_functions.u0_txd.assign(
    /// #     swm.pins.pio0_0.into_swm_pin(),
    /// #     &mut swm.handle,
    /// # );
    /// #
    /// // U0_TXD must have been previously assigned to the pin, or the
    /// // following code will not compile. See documentation of
    /// // `Function::assign`.
    /// let (u0_txd, pio0_0) = u0_txd.unassign(pio0_0, &mut swm.handle);
    /// ```
    ///
    /// [`Assigned`]: state/struct.Assigned.html
    pub fn unassign<S>(
        mut self,
        mut pin: Pin<P, S>,
        swm: &mut Handle,
    ) -> (
        Function<T, state::Unassigned>,
        <Pin<P, S> as UnassignFunction<T, T::Kind>>::Unassigned,
    )
    where
        T: FunctionTrait<P>,
        P: PinTrait,
        S: PinState,
        Pin<P, S>: UnassignFunction<T, T::Kind>,
    {
        self.ty.unassign(&mut pin.ty, swm);

        let function = Function {
            ty: self.ty,
            _state: state::Unassigned,
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
pub trait FunctionTrait<P: PinTrait> {
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

/// Implemented for types that designate whether a function is input or output
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait FunctionKind {}

/// Designates an SWM function as an input function
pub struct Input;
impl FunctionKind for Input {}

/// Designates an SWM function as an output function
pub struct Output;
impl FunctionKind for Output {}

/// Designates an SWM function as an analog function
pub struct Analog;
impl FunctionKind for Analog {}

/// Internal trait used to assign functions to pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Function::assign`] for the public API that uses this
/// trait.
pub trait AssignFunction<Function, Kind> {
    /// The type of the pin after the function has been assigned
    type Assigned;

    /// Internal method for assigning a function to a pin
    fn assign(self) -> Self::Assigned;
}

/// Internal trait used to unassign functions from pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Function::unassign`] for the public API that uses this
/// trait.
pub trait UnassignFunction<Function, Kind> {
    /// The type of the pin after the function has been unassigned
    type Unassigned;

    /// Internal method for unassigning a function from a pin
    fn unassign(self) -> Self::Unassigned;
}

macro_rules! movable_functions {
    (
        $(
            $field:ident,
            $type:ident,
            $kind:ident,
            $reg_name:ident,
            $reg_field:ident;
        )*
    ) => {
        /// Provides access to all movable functions
        ///
        /// This struct is part of [`swm::Parts`].
        ///
        /// [`swm::Parts`]: struct.Parts.html
        #[allow(missing_docs)]
        pub struct MovableFunctions {
            $(pub $field: Function<$type, state::Unassigned>,)*
        }

        impl MovableFunctions {
            fn new() -> Self {
                MovableFunctions {
                    $($field: Function {
                        ty    : $type(()),
                        _state: state::Unassigned,
                    },)*
                }
            }
        }


        $(
            /// Represents a movable function
            ///
            /// Movable functions can be accessed via the field
            /// `movable_functions` of [`swm::Parts`].
            ///
            /// [`swm::Parts`]: struct.Parts.html
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_0 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_1 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_2 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_3 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_4 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_5 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_6 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_7 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_8 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_9 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_10);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_11);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_12);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_13);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_14);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_15);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_16);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_17);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_18);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_19);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_20);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_21);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_22);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_23);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_24);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_25);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_26);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_27);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_28);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO0_29);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO0_30);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO0_31);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_0 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_1 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_2 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_3 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_4 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_5 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_6 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_7 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_8 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_9 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_10);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_11);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_12);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_13);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_14);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_15);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_16);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_17);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_18);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_19);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_20);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_21);
        )*
    }
}

macro_rules! impl_function {
    (
        $type:ident,
        $kind:ident,
        $reg_name:ident,
        $reg_field:ident,
        $pin:ident
    ) => {
        impl FunctionTrait<pins::$pin> for $type {
            type Kind = $kind;

            fn assign(&mut self, _pin: &mut pins::$pin, swm: &mut Handle) {
                swm.swm.$reg_name.modify(|_, w| unsafe {
                    w.$reg_field()
                        .bits(pins::$pin::ID | (pins::$pin::PORT as u8) << 5)
                });
            }

            fn unassign(&mut self, _pin: &mut pins::$pin, swm: &mut Handle) {
                swm.swm
                    .$reg_name
                    .modify(|_, w| unsafe { w.$reg_field().bits(0xff) });
            }
        }
    };
}

#[cfg(feature = "82x")]
movable_functions!(
    u0_txd       , U0_TXD       , Output, pinassign0 , u0_txd_o;
    u0_rxd       , U0_RXD       , Input , pinassign0 , u0_rxd_i;
    u0_rts       , U0_RTS       , Output, pinassign0 , u0_rts_o;
    u0_cts       , U0_CTS       , Input , pinassign0 , u0_cts_i;
    u0_sclk      , U0_SCLK      , Output, pinassign1 , u0_sclk_io;
    u1_txd       , U1_TXD       , Output, pinassign1 , u1_txd_o;
    u1_rxd       , U1_RXD       , Input , pinassign1 , u1_rxd_i;
    u1_rts       , U1_RTS       , Output, pinassign1 , u1_rts_o;
    u1_cts       , U1_CTS       , Input , pinassign2 , u1_cts_i;
    u1_sclk      , U1_SCLK      , Output, pinassign2 , u1_sclk_io;
    u2_txd       , U2_TXD       , Output, pinassign2 , u2_txd_o;
    u2_rxd       , U2_RXD       , Input , pinassign2 , u2_rxd_i;
    u2_rts       , U2_RTS       , Output, pinassign3 , u2_rts_o;
    u2_cts       , U2_CTS       , Input , pinassign3 , u2_cts_i;
    u2_sclk      , U2_SCLK      , Output, pinassign3 , u2_sclk_io;
    spi0_sck     , SPI0_SCK     , Output, pinassign3 , spi0_sck_io;
    spi0_mosi    , SPI0_MOSI    , Output, pinassign4 , spi0_mosi_io;
    spi0_miso    , SPI0_MISO    , Output, pinassign4 , spi0_miso_io;
    spi0_ssel0   , SPI0_SSEL0   , Output, pinassign4 , spi0_ssel0_io;
    spi0_ssel1   , SPI0_SSEL1   , Output, pinassign4 , spi0_ssel1_io;
    spi0_ssel2   , SPI0_SSEL2   , Output, pinassign5 , spi0_ssel2_io;
    spi0_ssel3   , SPI0_SSEL3   , Output, pinassign5 , spi0_ssel3_io;
    spi1_sck     , SPI1_SCK     , Output, pinassign5 , spi1_sck_io;
    spi1_mosi    , SPI1_MOSI    , Output, pinassign5 , spi1_mosi_io;
    spi1_miso    , SPI1_MISO    , Output, pinassign6 , spi1_miso_io;
    spi1_ssel0   , SPI1_SSEL0   , Output, pinassign6 , spi1_ssel0_io;
    spi1_ssel1   , SPI1_SSEL1   , Output, pinassign6 , spi1_ssel1_io;
    sct_pin0     , SCT_PIN0     , Input , pinassign6 , sct_pin0_i;
    sct_pin1     , SCT_PIN1     , Input , pinassign7 , sct_pin1_i;
    sct_pin2     , SCT_PIN2     , Input , pinassign7 , sct_pin2_i;
    sct_pin3     , SCT_PIN3     , Input , pinassign7 , sct_pin3_i;
    sct_out0     , SCT_OUT0     , Output, pinassign7 , sct_out0_o;
    sct_out1     , SCT_OUT1     , Output, pinassign8 , sct_out1_o;
    sct_out2     , SCT_OUT2     , Output, pinassign8 , sct_out2_o;
    sct_out3     , SCT_OUT3     , Output, pinassign8 , sct_out3_o;
    sct_out4     , SCT_OUT4     , Output, pinassign8 , sct_out4_o;
    sct_out5     , SCT_OUT5     , Output, pinassign9 , sct_out5_o;
    i2c1_sda     , I2C1_SDA     , Output, pinassign9 , i2c1_sda_io;
    i2c1_scl     , I2C1_SCL     , Output, pinassign9 , i2c1_scl_io;
    i2c2_sda     , I2C2_SDA     , Output, pinassign9 , i2c2_sda_io;
    i2c2_scl     , I2C2_SCL     , Output, pinassign10, i2c2_scl_io;
    i2c3_sda     , I2C3_SDA     , Output, pinassign10, i2c3_sda_io;
    i2c3_scl     , I2C3_SCL     , Output, pinassign10, i2c3_scl_io;
    adc_pintrig0 , ADC_PINTRIG0 , Input , pinassign10, adc_pintrig0_i;
    acd_pintrig1 , ADC_PINTRIG1 , Input , pinassign11, adc_pintrig1_i;
    acmp_o       , ACMP_O       , Output, pinassign11, acmp_o_o;
    clkout       , CLKOUT       , Output, pinassign11, clkout_o;
    gpio_int_bmat, GPIO_INT_BMAT, Output, pinassign11, gpio_int_bmat_o;
);

#[cfg(feature = "845")]
movable_functions!(
    u0_txd       , U0_TXD       , Output, pinassign0 , u0_txd_o;
    u0_rxd       , U0_RXD       , Input , pinassign0 , u0_rxd_i;
    u0_rts       , U0_RTS       , Output, pinassign0 , u0_rts_o;
    u0_cts       , U0_CTS       , Input , pinassign0 , u0_cts_i;
    u0_sclk      , U0_SCLK      , Output, pinassign1 , u0_sclk_io;
    u1_txd       , U1_TXD       , Output, pinassign1 , u1_txd_o;
    u1_rxd       , U1_RXD       , Input , pinassign1 , u1_rxd_i;
    u1_rts       , U1_RTS       , Output, pinassign1 , u1_rts_o;
    u1_cts       , U1_CTS       , Input , pinassign2 , u1_cts_i;
    u1_sclk      , U1_SCLK      , Output, pinassign2 , u1_sclk_io;
    u2_txd       , U2_TXD       , Output, pinassign2 , u2_txd_o;
    u2_rxd       , U2_RXD       , Input , pinassign2 , u2_rxd_i;
    u2_rts       , U2_RTS       , Output, pinassign3 , u2_rts_o;
    u2_cts       , U2_CTS       , Input , pinassign3 , u2_cts_i;
    u2_sclk      , U2_SCLK      , Output, pinassign3 , u2_sclk_io;
    u3_txd       , U3_TXD       , Output, pinassign11, uart3_txd;
    u3_rxd       , U3_RXD       , Input , pinassign12, uart3_rxd;
    u3_sclk      , U3_SCLK      , Output, pinassign12, uart3_sclk;
    u4_txd       , U4_TXD       , Output, pinassign12, uart4_txd;
    u4_rxd       , U4_RXD       , Input , pinassign12, uart4_rxd;
    u4_sclk      , U4_SCLK      , Output, pinassign13, uart4_sclk;
    spi0_sck     , SPI0_SCK     , Output, pinassign3 , spi0_sck_io;
    spi0_mosi    , SPI0_MOSI    , Output, pinassign4 , spi0_mosi_io;
    spi0_miso    , SPI0_MISO    , Output, pinassign4 , spi0_miso_io;
    spi0_ssel0   , SPI0_SSEL0   , Output, pinassign4 , spi0_ssel0_io;
    spi0_ssel1   , SPI0_SSEL1   , Output, pinassign4 , spi0_ssel1_io;
    spi0_ssel2   , SPI0_SSEL2   , Output, pinassign5 , spi0_ssel2_io;
    spi0_ssel3   , SPI0_SSEL3   , Output, pinassign5 , spi0_ssel3_io;
    spi1_sck     , SPI1_SCK     , Output, pinassign5 , spi1_sck_io;
    spi1_mosi    , SPI1_MOSI    , Output, pinassign5 , spi1_mosi_io;
    spi1_miso    , SPI1_MISO    , Output, pinassign6 , spi1_miso_io;
    spi1_ssel0   , SPI1_SSEL0   , Output, pinassign6 , spi1_ssel0_io;
    spi1_ssel1   , SPI1_SSEL1   , Output, pinassign6 , spi1_ssel1_io;
    sct_pin0     , SCT_PIN0     , Input , pinassign6 , sct0_gpio_in_a_i;
    sct_pin1     , SCT_PIN1     , Input , pinassign7 , sct0_gpio_in_b_i;
    sct_pin2     , SCT_PIN2     , Input , pinassign7 , sct0_gpio_in_c_i;
    sct_pin3     , SCT_PIN3     , Input , pinassign7 , sct0_gpio_in_d_i;
    sct_out0     , SCT_OUT0     , Output, pinassign7 , sct_out0_o;
    sct_out1     , SCT_OUT1     , Output, pinassign8 , sct_out1_o;
    sct_out2     , SCT_OUT2     , Output, pinassign8 , sct_out2_o;
    sct_out3     , SCT_OUT3     , Output, pinassign8 , sct_out3_o;
    sct_out4     , SCT_OUT4     , Output, pinassign8 , sct_out4_o;
    sct_out5     , SCT_OUT5     , Output, pinassign9 , sct_out5_o;
    sct_out6     , SCT_OUT6     , Output, pinassign9 , sct_out6_o;
    i2c1_sda     , I2C1_SDA     , Output, pinassign9 , i2c1_sda_io;
    i2c1_scl     , I2C1_SCL     , Output, pinassign9 , i2c1_scl_io;
    i2c2_sda     , I2C2_SDA     , Output, pinassign10, i2c2_sda_io;
    i2c2_scl     , I2C2_SCL     , Output, pinassign10, i2c2_scl_io;
    i2c3_sda     , I2C3_SDA     , Output, pinassign10, i2c3_sda_io;
    i2c3_scl     , I2C3_SCL     , Output, pinassign10, i2c3_scl_io;
    acmp_o       , ACMP_O       , Output, pinassign11, comp0_out_o;
    clkout       , CLKOUT       , Output, pinassign11, clkout_o;
    gpio_int_bmat, GPIO_INT_BMAT, Output, pinassign11, gpio_int_bmat_o;
    t0_mat0      , T0_MAT0      , Output, pinassign13, t0_mat0;
    t0_mat1      , T0_MAT1      , Output, pinassign13, t0_mat1;
    t0_mat2      , T0_MAT2      , Output, pinassign13, t0_mat2;
    t0_mat3      , T0_MAT3      , Output, pinassign14, t0_mat3;
    t0_cap0      , T0_CAP0      , Output, pinassign14, t0_cap0;
    t0_cap1      , T0_CAP1      , Output, pinassign14, t0_cap1;
    t0_cap2      , T0_CAP2      , Output, pinassign14, t0_cap2;
);

macro_rules! fixed_functions {
    ($(
        $type:ident,
        $kind:ident,
        $register:ident,
        $field:ident,
        $pin:ident,
        $default_state:ty;
    )*) => {
        /// Provides access to all fixed functions
        ///
        /// This struct is part of [`swm::Parts`].
        ///
        /// [`swm::Parts`]: struct.Parts.html
        #[allow(missing_docs)]
        pub struct FixedFunctions {
            $(pub $field: Function<$type, $default_state>,)*
        }

        impl FixedFunctions {
            fn new() -> Self {
                FixedFunctions {
                    $($field: Function {
                        ty    : $type(()),
                        _state: state::State::new(),
                    },)*
                }
            }
        }


        $(
            /// Represents a fixed function
            ///
            /// Fixed functions can be accessed via the field `fixed_functions`
            /// of [`swm::Parts`].
            ///
            /// [`swm::Parts`]: struct.Parts.html
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl FunctionTrait<pins::$pin> for $type {
                type Kind = $kind;


                fn assign(&mut self, _: &mut pins::$pin, swm : &mut Handle) {
                    swm.swm.$register.modify(|_, w| w.$field().clear_bit());
                }

                fn unassign(&mut self, _: &mut pins::$pin, swm : &mut Handle)
                {
                    swm.swm.$register.modify(|_, w| w.$field().set_bit());
                }
            }
        )*
    }
}

#[cfg(feature = "82x")]
fixed_functions!(
    ACMP_I1 , Input , pinenable0, acmp_i1 , PIO0_0 , state::Unassigned;
    ACMP_I2 , Input , pinenable0, acmp_i2 , PIO0_1 , state::Unassigned;
    ACMP_I3 , Input , pinenable0, acmp_i3 , PIO0_14, state::Unassigned;
    ACMP_I4 , Input , pinenable0, acmp_i4 , PIO0_23, state::Unassigned;
    SWCLK   , Output, pinenable0, swclk   , PIO0_3 , state::Assigned<pins::PIO0_3>;
    SWDIO   , Output, pinenable0, swdio   , PIO0_2 , state::Assigned<pins::PIO0_2>;
    XTALIN  , Input , pinenable0, xtalin  , PIO0_8 , state::Unassigned;
    XTALOUT , Output, pinenable0, xtalout , PIO0_9 , state::Unassigned;
    RESETN  , Input , pinenable0, resetn  , PIO0_5 , state::Assigned<pins::PIO0_5>;
    CLKIN   , Input , pinenable0, clkin   , PIO0_1 , state::Unassigned;
    VDDCMP  , Input , pinenable0, vddcmp  , PIO0_6 , state::Unassigned;
    I2C0_SDA, Output, pinenable0, i2c0_sda, PIO0_11, state::Unassigned;
    I2C0_SCL, Output, pinenable0, i2c0_scl, PIO0_10, state::Unassigned;
    ADC_0   , Analog, pinenable0, adc_0   , PIO0_7 , state::Unassigned;
    ADC_1   , Analog, pinenable0, adc_1   , PIO0_6 , state::Unassigned;
    ADC_2   , Analog, pinenable0, adc_2   , PIO0_14, state::Unassigned;
    ADC_3   , Analog, pinenable0, adc_3   , PIO0_23, state::Unassigned;
    ADC_4   , Analog, pinenable0, adc_4   , PIO0_22, state::Unassigned;
    ADC_5   , Analog, pinenable0, adc_5   , PIO0_21, state::Unassigned;
    ADC_6   , Analog, pinenable0, adc_6   , PIO0_20, state::Unassigned;
    ADC_7   , Analog, pinenable0, adc_7   , PIO0_19, state::Unassigned;
    ADC_8   , Analog, pinenable0, adc_8   , PIO0_18, state::Unassigned;
    ADC_9   , Analog, pinenable0, adc_9   , PIO0_17, state::Unassigned;
    ADC_10  , Analog, pinenable0, adc_10  , PIO0_13, state::Unassigned;
    ADC_11  , Analog, pinenable0, adc_11  , PIO0_4 , state::Unassigned;
);

#[cfg(feature = "845")]
fixed_functions!(
    ACMP_I1 , Input , pinenable0, acmp_i1 , PIO0_0 , state::Unassigned;
    ACMP_I2 , Input , pinenable0, acmp_i2 , PIO0_1 , state::Unassigned;
    ACMP_I3 , Input , pinenable0, acmp_i3 , PIO0_14, state::Unassigned;
    ACMP_I4 , Input , pinenable0, acmp_i4 , PIO0_23, state::Unassigned;
    SWCLK   , Output, pinenable0, swclk   , PIO0_3 , state::Assigned<pins::PIO0_3>;
    SWDIO   , Output, pinenable0, swdio   , PIO0_2 , state::Assigned<pins::PIO0_2>;
    XTALIN  , Input , pinenable0, xtalin  , PIO0_8 , state::Unassigned;
    XTALOUT , Output, pinenable0, xtalout , PIO0_9 , state::Unassigned;
    RESETN  , Input , pinenable0, resetn  , PIO0_5 , state::Assigned<pins::PIO0_5>;
    CLKIN   , Input , pinenable0, clkin   , PIO0_1 , state::Unassigned;
    VDDCMP  , Input , pinenable0, vddcmp  , PIO0_6 , state::Unassigned;
    I2C0_SDA, Output, pinenable0, i2c0_sda, PIO0_11, state::Unassigned;
    I2C0_SCL, Output, pinenable0, i2c0_scl, PIO0_10, state::Unassigned;
    ADC_0   , Analog, pinenable0, adc_0   , PIO0_7 , state::Unassigned;
    ADC_1   , Analog, pinenable0, adc_1   , PIO0_6 , state::Unassigned;
    ADC_2   , Analog, pinenable0, adc_2   , PIO0_14, state::Unassigned;
    ADC_3   , Analog, pinenable0, adc_3   , PIO0_23, state::Unassigned;
    ADC_4   , Analog, pinenable0, adc_4   , PIO0_22, state::Unassigned;
    ADC_5   , Analog, pinenable0, adc_5   , PIO0_21, state::Unassigned;
    ADC_6   , Analog, pinenable0, adc_6   , PIO0_20, state::Unassigned;
    ADC_7   , Analog, pinenable0, adc_7   , PIO0_19, state::Unassigned;
    ADC_8   , Analog, pinenable0, adc_8   , PIO0_18, state::Unassigned;
    ADC_9   , Analog, pinenable0, adc_9   , PIO0_17, state::Unassigned;
    ADC_10  , Analog, pinenable0, adc_10  , PIO0_13, state::Unassigned;
    ADC_11  , Analog, pinenable0, adc_11  , PIO0_4 , state::Unassigned;
    DACOUT0 , Analog, pinenable0, dacout0 , PIO0_17, state::Unassigned;
    DACOUT1 , Analog, pinenable0, dacout1 , PIO0_29, state::Unassigned;
    CAPT_X0 , Analog, pinenable0, capt_x0 , PIO0_31, state::Unassigned;
    CAPT_X1 , Analog, pinenable0, capt_x1 , PIO1_0 , state::Unassigned;
    CAPT_X2 , Analog, pinenable0, capt_x2 , PIO1_1 , state::Unassigned;
    CAPT_X3 , Analog, pinenable0, capt_x3 , PIO1_2 , state::Unassigned;
    CAPT_X4 , Analog, pinenable1, capt_x4 , PIO1_3 , state::Unassigned;
    CAPT_X5 , Analog, pinenable1, capt_x5 , PIO1_4 , state::Unassigned;
    CAPT_X6 , Analog, pinenable1, capt_x6 , PIO1_5 , state::Unassigned;
    CAPT_X7 , Analog, pinenable1, capt_x7 , PIO1_6 , state::Unassigned;
    CAPT_X8 , Analog, pinenable1, capt_x8 , PIO1_7 , state::Unassigned;
    CAPT_YL , Analog, pinenable1, capt_yl , PIO1_8 , state::Unassigned;
    CAPT_YH , Analog, pinenable1, capt_yh , PIO1_8 , state::Unassigned;
);

/// Contains types that indicate the state of fixed or movable functions
pub mod state {
    use core::marker::PhantomData;

    /// Implemented by types that indicate the state of SWM functions
    ///
    /// This trait is implemented by types that indicate the state of SWM
    /// functions. It exists only to document which types those are. The user
    /// should not need to implement this trait, nor use it directly.
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
}
