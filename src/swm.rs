//! APIs for the switch matrix (SWM)
//!
//! The entry point to this API is [`SWM`]. Please refer to [`SWM`]'s
//! documentation for additional information.
//!
//! The switch matrix is described in the user manual, chapter 7.

use core::marker::PhantomData;

use gpio::{
    self,
    GPIO,
};
use init_state::{
    self,
    InitState,
};
use raw;
use syscon;

use self::pin_state::PinState;


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
pub struct SWM {
    swm: raw::SWM,
}

impl SWM {
    pub(crate) fn new(swm: raw::SWM) -> Self {
        SWM { swm }
    }

    /// Splits the SWM API into its component parts
    ///
    /// This is the regular way to access the SWM API. It exists as an explicit
    /// step, as it's no longer possible to gain access to the raw peripheral
    /// using [`SWM::free`] after you've called this method.
    pub fn split(self) -> Parts {
        Parts {
            handle           : Handle::new(self.swm),
            pins             : Pins::new(),
            movable_functions: MovableFunctions::new(),
            fixed_functions  : FixedFunctions::new(),
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
    /// [open an issue]: https://github.com/braun-robotics/rust-lpc82x-hal/issues
    pub fn free(self) -> raw::SWM {
        self.swm
    }
}


/// The main API for the switch matrix (SWM)
///
/// Provides access to all types that make up the SWM API. Please refer to the
/// [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts {
    /// Handle to the switch matrix
    pub handle: Handle<init_state::Enabled>,

    /// Pins that can be used for GPIO or other functions
    pub pins: Pins,

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
pub struct Handle<State: InitState = init_state::Enabled> {
    swm   : raw::SWM,
    _state: State,
}

impl Handle<init_state::Enabled> {
    pub(crate) fn new(swm: raw::SWM) -> Self {
        Handle {
            swm   : swm,
            _state: init_state::Enabled,
        }
    }
}

impl Handle<init_state::Disabled> {
    /// Enable the switch matrix
    ///
    /// Enables the clock for the SWM peripheral.
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
    pub fn enable(mut self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Enabled>
    {
        syscon.enable_clock(&mut self.swm);

        Handle {
            swm   : self.swm,
            _state: init_state::Enabled,
        }
    }
}

impl Handle<init_state::Enabled> {
    /// Disable the switch matrix
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
    pub fn disable(mut self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Disabled>
    {
        syscon.disable_clock(&mut self.swm);

        Handle {
            swm   : self.swm,
            _state: init_state::Disabled,
        }
    }
}


/// Implemented by types that identify pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Pin`] for the public API used to control pins.
pub trait PinTrait {
    /// A number that identifies the pin
    ///
    /// This is `0` for [`PIO0_0`], `1` for [`PIO0_1`] and so forth.
    const ID: u8;

    /// The pin's bit mask
    ///
    /// This is `0x00000001` for [`PIO0_0`], `0x00000002` for [`PIO0_1`],
    /// `0x00000004` for [`PIO0_2`], and so forth.
    const MASK: u32;
}


macro_rules! pins {
    ($(
        $field:ident,
        $type:ident,
        $id:expr,
        $default_state_ty:ty,
        $default_state_val:expr;
    )*) => {
        /// Provides access to all pins
        ///
        /// This struct is a part of [`swm::Parts`].
        ///
        /// # Limitations
        ///
        /// This struct currently provides access to all pins that can be
        /// available on an LPC82x part. Please make sure that you are aware of
        /// which pins are actually available on your specific part, and only
        /// use those.
        ///
        /// [`swm::Parts`]: struct.Parts.html
        #[allow(missing_docs)]
        pub struct Pins {
            $(pub $field: Pin<$type, $default_state_ty>,)*
        }

        impl Pins {
            pub(crate) fn new() -> Self {
                Pins {
                    $(
                        $field: Pin {
                            ty   : $type(()),
                            state: $default_state_val,
                        },
                    )*
                }
            }
        }


        $(
            /// Identifies a specific pin
            ///
            /// Pins can be accessed via the field `pins` of [`swm::Parts`].
            ///
            /// [`swm::Parts`]: struct.Parts.html
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl PinTrait for $type {
                const ID  : u8  = $id;
                const MASK: u32 = 0x1 << $id;
            }
        )*
    }
}

pins!(
    pio0_0 , PIO0_0 , 0x00, pin_state::Unused        , pin_state::Unused;
    pio0_1 , PIO0_1 , 0x01, pin_state::Unused        , pin_state::Unused;
    pio0_2 , PIO0_2 , 0x02, pin_state::Swm<((),), ()>, pin_state::Swm::new();
    pio0_3 , PIO0_3 , 0x03, pin_state::Swm<((),), ()>, pin_state::Swm::new();
    pio0_4 , PIO0_4 , 0x04, pin_state::Unused        , pin_state::Unused;
    pio0_5 , PIO0_5 , 0x05, pin_state::Swm<(), ((),)>, pin_state::Swm::new();
    pio0_6 , PIO0_6 , 0x06, pin_state::Unused        , pin_state::Unused;
    pio0_7 , PIO0_7 , 0x07, pin_state::Unused        , pin_state::Unused;
    pio0_8 , PIO0_8 , 0x08, pin_state::Unused        , pin_state::Unused;
    pio0_9 , PIO0_9 , 0x09, pin_state::Unused        , pin_state::Unused;
    pio0_10, PIO0_10, 0x0a, pin_state::Unused        , pin_state::Unused;
    pio0_11, PIO0_11, 0x0b, pin_state::Unused        , pin_state::Unused;
    pio0_12, PIO0_12, 0x0c, pin_state::Unused        , pin_state::Unused;
    pio0_13, PIO0_13, 0x0d, pin_state::Unused        , pin_state::Unused;
    pio0_14, PIO0_14, 0x0e, pin_state::Unused        , pin_state::Unused;
    pio0_15, PIO0_15, 0x0f, pin_state::Unused        , pin_state::Unused;
    pio0_16, PIO0_16, 0x10, pin_state::Unused        , pin_state::Unused;
    pio0_17, PIO0_17, 0x11, pin_state::Unused        , pin_state::Unused;
    pio0_18, PIO0_18, 0x12, pin_state::Unused        , pin_state::Unused;
    pio0_19, PIO0_19, 0x13, pin_state::Unused        , pin_state::Unused;
    pio0_20, PIO0_20, 0x14, pin_state::Unused        , pin_state::Unused;
    pio0_21, PIO0_21, 0x15, pin_state::Unused        , pin_state::Unused;
    pio0_22, PIO0_22, 0x16, pin_state::Unused        , pin_state::Unused;
    pio0_23, PIO0_23, 0x17, pin_state::Unused        , pin_state::Unused;
    pio0_24, PIO0_24, 0x18, pin_state::Unused        , pin_state::Unused;
    pio0_25, PIO0_25, 0x19, pin_state::Unused        , pin_state::Unused;
    pio0_26, PIO0_26, 0x1a, pin_state::Unused        , pin_state::Unused;
    pio0_27, PIO0_27, 0x1b, pin_state::Unused        , pin_state::Unused;
    pio0_28, PIO0_28, 0x1c, pin_state::Unused        , pin_state::Unused;
);


/// Main API to control for controlling pins
///
/// `Pin` has two type parameters:
/// - `T`, to indicate which specific pin this instance of `Pin` represents (so,
///   [`PIO0_0`], [`PIO0_1`], and so on)
/// - `S`, to indicate which state the represented pin is currently in
///
/// A pin instance can be in one of the following states:
/// - [`pin_state::Unused`], to indicate that the pin is currently not used
/// - [`pin_state::Gpio`], to indicate that the pin is being used for
///   general-purpose I/O
/// - [`pin_state::Swm`], to indicate that the pin is available for switch
///   matrix function assignment
/// - [`pin_state::Adc`], to indicate that the pin is being used for analog
///   input
///
/// # State Management
///
/// All pins start out their initial state, as defined in the user manual. To
/// prevent us from making mistakes, only the methods that induce a valid state
/// transition are available. Code that tries to call a method that would cause
/// an invalid state transition will simply not compile:
///
/// ``` no_run
/// # use lpc82x_hal::Peripherals;
/// #
/// # let mut p = Peripherals::take().unwrap();
/// #
/// # let mut swm = p.SWM.split();
/// #
/// // Assign a function to a pin
/// let (clkout, pio0_12) = swm.movable_functions.clkout.assign(
///     swm.pins.pio0_12.into_swm_pin(),
///     &mut swm.handle,
/// );
///
/// // As long as the function is assigned, we can't use the pin for
/// // general-purpose I/O. Therefore the following method call would cause a
/// // compile-time error.
/// // let pio0_12 = pio0_12.into_gpio_pin(&p.GPIO);
/// ```
///
/// To use the pin in the above example for GPIO, we first have to unassign the
/// movable function and transition the pin to the unused state:
///
/// ``` no_run
/// # use lpc82x_hal::Peripherals;
/// #
/// # let mut p = Peripherals::take().unwrap();
/// #
/// # let mut swm = p.SWM.split();
/// #
/// # let (clkout, pio0_12) = swm.movable_functions.clkout.assign(
/// #     swm.pins.pio0_12.into_swm_pin(),
/// #     &mut swm.handle,
/// # );
/// #
/// let (clkout, pio0_12) = clkout.unassign(pio0_12, &mut swm.handle);
/// let pio0_12 = pio0_12.into_unused_pin();
///
/// // Now we can transition the pin into the GPIO state.
/// let pio0_12 = pio0_12.into_gpio_pin(&p.GPIO);
/// ```
///
/// # General Purpose I/O
///
/// All pins can be used for general-purpose I/O (GPIO), meaning they can be
/// used for reading digital input signals and writing digital output signals.
/// To set up a pin for GPIO use, you need to call [`Pin::into_gpio_pin`] when
/// it is in its unused state.
///
/// ``` no_run
/// use lpc82x_hal::Peripherals;
///
/// let mut p = Peripherals::take().unwrap();
///
/// let mut swm = p.SWM.split();
///
/// // The pin takes a shared reference to `GPIO`, which it keeps around as long
/// // as the pin is in the GPIO state. This ensures the GPIO peripheral can't
/// // be disabled while we're still using the pin for GPIO.
/// let pin = swm.pins.pio0_12.into_gpio_pin(&p.GPIO);
/// ```
///
/// Now `pin` is in the GPIO state. The GPIO state has the following sub-states:
/// - [`direction::Unknown`], to indicate that the current GPIO configuration is
///   not known
/// - [`direction::Input`], to indicate that the pin is configured for digital
///   input
/// - [`direction::Output`], to indicate that the pin is configured for digital
///   output
///
/// To use a pin, that we previously configured for GPIO (see example above),
/// for digital output, we need to set the pin direction using
/// [`Pin::into_output`].
///
/// ``` no_run
/// # use lpc82x_hal::Peripherals;
/// #
/// # let p = Peripherals::take().unwrap();
/// #
/// # let mut swm = p.SWM.split();
/// #
/// # let pin = swm.pins.pio0_12
/// #     .into_gpio_pin(&p.GPIO);
/// #
/// use lpc82x_hal::prelude::*;
///
/// // Configure pin for digital output. This assumes that the pin is currently
/// // in the GPIO state.
/// let mut pin = pin.into_output();
///
/// // Now we can change the output signal as we like.
/// pin.set_high();
/// pin.set_low();
/// ```
///
/// Using pins for digital input is currently not supported by the API. If you
/// need this feature, [please speak up](https://github.com/braun-robotics/rust-lpc82x-hal/issues/50).
///
/// # Fixed and Movable Functions
///
/// Besides general-purpose I/O, pins can be used for a number of more
/// specialized functions. Some of those can be used only on one specific pin
/// (fixed functions), others can be assigned to any pin (movable functions).
///
/// Before you can assign any functions to a pin, you need to transition it from
/// the unused state to the SWM state using [`Pin::into_swm_pin`].
///
/// ``` no_run
/// # use lpc82x_hal::Peripherals;
/// #
/// # let p = Peripherals::take().unwrap();
/// #
/// # let swm = p.SWM.split();
/// #
/// let pin = swm.pins.pio0_12
///     .into_swm_pin();
///
/// // Functions can be assigned now using the methods on `Function`
/// ```
///
/// As mentioned above, a function can be fixed or movable. But there is also
/// another distinction: Functions can be input or output functions. Any number
/// of input functions can be assigned to a pin at the same time, but at most
/// one output function can be assigned to a pin at once (see user manual,
/// section 7.4). These rules are enforced by the API at compile time.
///
/// **NOTE:** There is some uncertainty about whether those rules treat GPIO as
/// just another kind of function, or if they don't apply to it. Currently, this
/// API treats GPIO as something entirely different from the switch matrix
/// functions, which may be too restrictive. If you have any insight on this
/// topic, [please help us figure this out](https://github.com/braun-robotics/rust-lpc82x-hal/issues/44).
///
/// Once a pin is in the SWM state, you can assign functions to it. Please refer
/// to [`Function`] for more information on how to do that.
///
/// # Analog Input
///
/// To use a pin for analog input, you need to assign an ADC function:
///
/// ``` no_run
/// use lpc82x_hal::Peripherals;
///
/// let p = Peripherals::take().unwrap();
///
/// let mut swm = p.SWM.split();
///
/// // Transition pin into ADC state
/// let (adc_2, pio0_14) = swm.fixed_functions.adc_2.assign(
///     swm.pins.pio0_14.into_swm_pin(),
///     &mut swm.handle,
/// );
/// ```
///
/// Using the pin for analog input once it is in the ADC state is currently not
/// supported by this API. If you need this feature, [please let us know](https://github.com/braun-robotics/rust-lpc82x-hal/issues/51)!
///
/// As a woraround, you can use the raw register mappings from the lpc82x crate,
/// [`lpc82x::IOCON`] and [`lpc82x::ADC`], after you have put the pin into the
/// ADC state.
///
/// [`direction::Unknown`]: ../gpio/direction/struct.Unknown.html
/// [`direction::Input`]: ../gpio/direction/struct.Input.html
/// [`direction::Output`]: ../gpio/direction/struct.Output.html
/// [`lpc82x::IOCON`]: https://docs.rs/lpc82x/0.4.*/lpc82x/struct.IOCON.html
/// [`lpc82x::ADC`]: https://docs.rs/lpc82x/0.4.*/lpc82x/struct.ADC.html
pub struct Pin<T: PinTrait, S: PinState> {
    pub(crate) ty   : T,
    pub(crate) state: S,
}

impl<T> Pin<T, pin_state::Unused> where T: PinTrait {
    /// Transition pin to GPIO state
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes this pin instance and returns a new instance that is in the
    /// GPIO state, allowing you to use the pin for general-purpose I/O. As long
    /// as the pin is in the GPIO state, it needs the GPIO peripheral to be
    /// enabled to function correctly. To statically guarantee that this is the
    /// case, this method takes a shared reference to [`GPIO`], which the pin
    /// keeps around until it leaves the GPIO state.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc82x_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let swm = p.SWM.split();
    ///
    /// let pin = swm.pins.pio0_12
    ///     .into_gpio_pin(&p.GPIO);
    ///
    /// // `pin` is now available for general-purpose I/O
    /// ```
    ///
    /// [State Management]: #state-management
    pub fn into_gpio_pin(self, gpio: &GPIO)
        -> Pin<T, pin_state::Gpio<gpio::direction::Unknown>>
    {
        Pin {
            ty   : self.ty,
            state: pin_state::Gpio {
                dirset0: &gpio.gpio.dirset0,
                pin0   : &gpio.gpio.pin0,
                set0   : &gpio.gpio.set0,
                clr0   : &gpio.gpio.clr0,

                _direction: gpio::direction::Unknown,
            },
        }
    }

    /// Transition pin to SWM state
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes this pin instance and returns a new instance that is in the SWM
    /// state, making this pin available for switch matrix function assignment.
    ///
    /// Please refer [`Function`] to learn more about SWM function assignment.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc82x_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let swm = p.SWM.split();
    ///
    /// let pin = swm.pins.pio0_12
    ///     .into_swm_pin();
    ///
    /// // `pin` is now ready for function assignment
    /// ```
    ///
    /// [State Management]: #state-management
    pub fn into_swm_pin(self) -> Pin<T, pin_state::Swm<(), ()>> {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T> Pin<T, pin_state::Swm<(), ()>> where T: PinTrait {
    /// Transitions this pin from the SWM state to the unused state
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the SWM state.
    /// - No functions are assigned to this pin.
    ///
    /// Unless both of these conditions are met, code trying to call this method
    /// will not compile.
    ///
    /// Consumes the pin instance and returns a new pin instance, its type state
    /// indicating it is unused. This makes it possible to use the pin for
    /// something else. See [State Management] for more information on managing
    /// pin states.
    ///
    /// [State Management]: #state-management
    pub fn into_unused_pin(self) -> Pin<T, pin_state::Unused> {
        Pin {
            ty   : self.ty,
            state: pin_state::Unused,
        }
    }
}

impl<T, F, O, Is> AssignFunction<F, Input>
    for Pin<T, pin_state::Swm<O, Is>>
    where
        T: PinTrait,
        F: FunctionTrait<T, Kind=Input>,
{
    type Assigned = Pin<T, pin_state::Swm<O, (Is,)>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Is> AssignFunction<F, Output>
    for Pin<T, pin_state::Swm<(), Is>>
    where
        T: PinTrait,
        F: FunctionTrait<T, Kind=Output>,
{
    type Assigned = Pin<T, pin_state::Swm<((),), Is>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, O, Is> UnassignFunction<F, Input>
     for Pin<T, pin_state::Swm<O, (Is,)>>
     where
        T: PinTrait,
        F: FunctionTrait<T, Kind=Input>,
{
    type Unassigned = Pin<T, pin_state::Swm<O, Is>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Is> UnassignFunction<F, Output>
     for Pin<T, pin_state::Swm<((),), Is>>
     where
        T: PinTrait,
        F: FunctionTrait<T, Kind=Output>,
{
    type Unassigned = Pin<T, pin_state::Swm<(), Is>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F> AssignFunction<F, Adc>
    for Pin<T, pin_state::Swm<(), ()>>
    where
        T: PinTrait,
        F: FunctionTrait<T, Kind=Adc>,
{
    type Assigned = Pin<T, pin_state::Adc>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Adc,
        }
    }
}


/// Contains types that indicate pin states
///
/// Please refer to [`Pin`] for documentation about how these types are used.
pub mod pin_state {
    use core::marker::PhantomData;

    use gpio::direction::Direction;
    use raw::gpio_port::{
        CLR0,
        DIRSET0,
        PIN0,
        SET0,
    };


    /// Implemented by types that indicate pin state
    ///
    /// [`Pin`] uses this type as a trait bound for the type parameter that
    /// indicates the pin's state. This is done for the purpose of
    /// documentation, to show which states a pin can be in. Other than that,
    /// this trait should not be relevant to users of this crate.
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub trait PinState {}


    /// Marks a [`Pin`] as being unused
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Unused;

    impl PinState for Unused {}


    /// Marks a [`Pin`]  as being assigned to the analog-to-digital converter
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Adc;

    impl PinState for Adc {}


    /// Marks a [`Pin`]  as being assigned to general-purpose I/O
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Gpio<'gpio, D: Direction> {
        pub(crate) dirset0: &'gpio DIRSET0,
        pub(crate) pin0   : &'gpio PIN0,
        pub(crate) set0   : &'gpio SET0,
        pub(crate) clr0   : &'gpio CLR0,

        pub(crate) _direction: D,
    }

    impl<'gpio, D> PinState for Gpio<'gpio, D> where D: Direction {}


    /// Marks a [`Pin`]  as being available for switch matrix function assigment
    ///
    /// The type parameters of this struct track whether output and input
    /// functions have been assigned to a pin:
    ///
    /// - `Output` tracks whether an output function has been assigned. Zero or
    ///   one output functions can be assigned to a pin.
    /// - `Inputs` tracks the number of assigned input functions. Any number of
    ///   input functions can be assigned to a pin at the same time.
    ///
    /// Both type parameters use nested tuples to count the number of assigned
    /// functions. The empty tuple (`()`) represents zero assigned functions,
    /// the empty tuple nested in another tuple (`((),)`) represents one
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

    impl<Output, Inputs> PinState for Swm<Output, Inputs> {}
}


/// A fixed or movable function that can be assigned to a pin
///
/// The type parameter `T` identifies the fixed or movable function that an
/// instance of `Function` controls. The other type paramter, `State`, tracks
/// whether this function is assigned to a pin, and which pin it is assigned to.
pub struct Function<T, State> {
    ty    : T,
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
    pub fn assign<P, S>(mut self, mut pin: Pin<P, S>, swm: &mut Handle)
        -> (
            Function<T, state::Assigned<P>>,
            <Pin<P, S> as AssignFunction<T, T::Kind>>::Assigned,
        )
        where
            T        : FunctionTrait<P>,
            P        : PinTrait,
            S        : PinState,
            Pin<P, S>: AssignFunction<T, T::Kind>,
    {
        self.ty.assign(&mut pin.ty, swm);

        let function = Function {
            ty    : self.ty,
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
    pub fn unassign<S>(mut self, mut pin: Pin<P, S>, swm: &mut Handle)
        -> (
            Function<T, state::Unassigned>,
            <Pin<P, S> as UnassignFunction<T, T::Kind>>::Unassigned,
        )
        where
            T        : FunctionTrait<P>,
            P        : PinTrait,
            S        : PinState,
            Pin<P, S>: UnassignFunction<T, T::Kind>,
    {
        self.ty.unassign(&mut pin.ty, swm);

        let function = Function {
            ty    : self.ty,
            _state: state::Unassigned,
        };

        (function, pin.unassign())
    }
}


/// Implemented for all fixed and movable functions
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
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
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait FunctionKind {}

/// Designates an SWM function as an input function
pub struct Input;
impl FunctionKind for Input {}

/// Designates an SWM function as an output function
pub struct Output;
impl FunctionKind for Output {}

/// Designates an SWM function as an ADC function
pub struct Adc;
impl FunctionKind for Adc {}


/// Internal trait used to assign functions to pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
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
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
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
        impl FunctionTrait<$pin> for $type {
            type Kind = $kind;


            fn assign(&mut self, _pin: &mut $pin, swm : &mut Handle) {
                swm.swm.$reg_name.modify(|_, w|
                    unsafe { w.$reg_field().bits($pin::ID) }
                );
            }

            fn unassign(&mut self, _pin: &mut $pin, swm : &mut Handle) {
                swm.swm.$reg_name.modify(|_, w|
                    unsafe { w.$reg_field().bits(0xff) }
                );
            }
        }
    }
}

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
    sct_pin0     , SCT_PIN0     , Input , pinassign6 , sct_in0_i;
    sct_pin1     , SCT_PIN1     , Input , pinassign7 , sct_in1_i;
    sct_pin2     , SCT_PIN2     , Input , pinassign7 , sct_in2_i;
    sct_pin3     , SCT_PIN3     , Input , pinassign7 , sct_in3_i;
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


macro_rules! fixed_functions {
    ($(
        $type:ident,
        $kind:ident,
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

            impl FunctionTrait<$pin> for $type {
                type Kind = $kind;


                fn assign(&mut self, _: &mut $pin, swm : &mut Handle) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().clear_bit());
                }

                fn unassign(&mut self, _: &mut $pin, swm : &mut Handle)
                {
                    swm.swm.pinenable0.modify(|_, w| w.$field().set_bit());
                }
            }
        )*
    }
}

fixed_functions!(
    ACMP_I1 , Input , acmp_i1 , PIO0_0 , state::Unassigned;
    ACMP_I2 , Input , acmp_i2 , PIO0_1 , state::Unassigned;
    ACMP_I3 , Input , acmp_i3 , PIO0_14, state::Unassigned;
    ACMP_I4 , Input , acmp_i4 , PIO0_23, state::Unassigned;
    SWCLK   , Output, swclk   , PIO0_3 , state::Assigned<PIO0_3>;
    SWDIO   , Output, swdio   , PIO0_2 , state::Assigned<PIO0_2>;
    XTALIN  , Input , xtalin  , PIO0_8 , state::Unassigned;
    XTALOUT , Output, xtalout , PIO0_9 , state::Unassigned;
    RESETN  , Input , resetn  , PIO0_5 , state::Assigned<PIO0_5>;
    CLKIN   , Input , clkin   , PIO0_1 , state::Unassigned;
    VDDCMP  , Input , vddcmp  , PIO0_6 , state::Unassigned;
    I2C0_SDA, Output, i2c0_sda, PIO0_11, state::Unassigned;
    I2C0_SCL, Output, i2c0_scl, PIO0_10, state::Unassigned;
    ADC_0   , Adc   , adc_0   , PIO0_7 , state::Unassigned;
    ADC_1   , Adc   , adc_1   , PIO0_6 , state::Unassigned;
    ADC_2   , Adc   , adc_2   , PIO0_14, state::Unassigned;
    ADC_3   , Adc   , adc_3   , PIO0_23, state::Unassigned;
    ADC_4   , Adc   , adc_4   , PIO0_22, state::Unassigned;
    ADC_5   , Adc   , adc_5   , PIO0_21, state::Unassigned;
    ADC_6   , Adc   , adc_6   , PIO0_20, state::Unassigned;
    ADC_7   , Adc   , adc_7   , PIO0_19, state::Unassigned;
    ADC_8   , Adc   , adc_8   , PIO0_18, state::Unassigned;
    ADC_9   , Adc   , adc_9   , PIO0_17, state::Unassigned;
    ADC_10  , Adc   , adc_10  , PIO0_13, state::Unassigned;
    ADC_11  , Adc   , adc_11  , PIO0_4 , state::Unassigned;
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
        fn new() -> Self { Unassigned }
    }


    /// Indicates that a function is assigned to a pin
    pub struct Assigned<Pin>(pub(crate) PhantomData<Pin>);

    impl<Pin> State for Assigned<Pin> {
        fn new() -> Self { Assigned(PhantomData) }
    }
}
