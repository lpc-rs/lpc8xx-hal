//! API to control pins

use crate::gpio::{self, GPIO};

use self::pin_state::PinState;

/// Implemented by types that identify pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Pin`] for the public API used to control pins.
pub trait PinTrait {
    /// A number that indentifies the port
    ///
    /// This is `0` for [`PIO0_0`] and `1` for [`PIO1_0`]
    const PORT: usize;
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
        $port:expr,
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
        /// available on an LPC8xx part. Please make sure that you are aware of
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
                const PORT: usize = $port;
                const ID  : u8    = $id;
                const MASK: u32   = 0x1 << $id;
            }
        )*
    }
}

#[cfg(feature = "82x")]
pins!(
    pio0_0 , PIO0_0 , 0, 0x00, pin_state::Unused        , pin_state::Unused;
    pio0_1 , PIO0_1 , 0, 0x01, pin_state::Unused        , pin_state::Unused;
    pio0_2 , PIO0_2 , 0, 0x02, pin_state::Swm<((),), ()>, pin_state::Swm::new();
    pio0_3 , PIO0_3 , 0, 0x03, pin_state::Swm<((),), ()>, pin_state::Swm::new();
    pio0_4 , PIO0_4 , 0, 0x04, pin_state::Unused        , pin_state::Unused;
    pio0_5 , PIO0_5 , 0, 0x05, pin_state::Swm<(), ((),)>, pin_state::Swm::new();
    pio0_6 , PIO0_6 , 0, 0x06, pin_state::Unused        , pin_state::Unused;
    pio0_7 , PIO0_7 , 0, 0x07, pin_state::Unused        , pin_state::Unused;
    pio0_8 , PIO0_8 , 0, 0x08, pin_state::Unused        , pin_state::Unused;
    pio0_9 , PIO0_9 , 0, 0x09, pin_state::Unused        , pin_state::Unused;
    pio0_10, PIO0_10, 0, 0x0a, pin_state::Unused        , pin_state::Unused;
    pio0_11, PIO0_11, 0, 0x0b, pin_state::Unused        , pin_state::Unused;
    pio0_12, PIO0_12, 0, 0x0c, pin_state::Unused        , pin_state::Unused;
    pio0_13, PIO0_13, 0, 0x0d, pin_state::Unused        , pin_state::Unused;
    pio0_14, PIO0_14, 0, 0x0e, pin_state::Unused        , pin_state::Unused;
    pio0_15, PIO0_15, 0, 0x0f, pin_state::Unused        , pin_state::Unused;
    pio0_16, PIO0_16, 0, 0x10, pin_state::Unused        , pin_state::Unused;
    pio0_17, PIO0_17, 0, 0x11, pin_state::Unused        , pin_state::Unused;
    pio0_18, PIO0_18, 0, 0x12, pin_state::Unused        , pin_state::Unused;
    pio0_19, PIO0_19, 0, 0x13, pin_state::Unused        , pin_state::Unused;
    pio0_20, PIO0_20, 0, 0x14, pin_state::Unused        , pin_state::Unused;
    pio0_21, PIO0_21, 0, 0x15, pin_state::Unused        , pin_state::Unused;
    pio0_22, PIO0_22, 0, 0x16, pin_state::Unused        , pin_state::Unused;
    pio0_23, PIO0_23, 0, 0x17, pin_state::Unused        , pin_state::Unused;
    pio0_24, PIO0_24, 0, 0x18, pin_state::Unused        , pin_state::Unused;
    pio0_25, PIO0_25, 0, 0x19, pin_state::Unused        , pin_state::Unused;
    pio0_26, PIO0_26, 0, 0x1a, pin_state::Unused        , pin_state::Unused;
    pio0_27, PIO0_27, 0, 0x1b, pin_state::Unused        , pin_state::Unused;
    pio0_28, PIO0_28, 0, 0x1c, pin_state::Unused        , pin_state::Unused;
);

#[cfg(feature = "845")]
pins!(
    pio0_0 , PIO0_0 , 0, 0x00, pin_state::Unused        , pin_state::Unused;
    pio0_1 , PIO0_1 , 0, 0x01, pin_state::Unused        , pin_state::Unused;
    pio0_2 , PIO0_2 , 0, 0x02, pin_state::Swm<((),), ()>, pin_state::Swm::new();
    pio0_3 , PIO0_3 , 0, 0x03, pin_state::Swm<((),), ()>, pin_state::Swm::new();
    pio0_4 , PIO0_4 , 0, 0x04, pin_state::Unused        , pin_state::Unused;
    pio0_5 , PIO0_5 , 0, 0x05, pin_state::Swm<(), ((),)>, pin_state::Swm::new();
    pio0_6 , PIO0_6 , 0, 0x06, pin_state::Unused        , pin_state::Unused;
    pio0_7 , PIO0_7 , 0, 0x07, pin_state::Unused        , pin_state::Unused;
    pio0_8 , PIO0_8 , 0, 0x08, pin_state::Unused        , pin_state::Unused;
    pio0_9 , PIO0_9 , 0, 0x09, pin_state::Unused        , pin_state::Unused;
    pio0_10, PIO0_10, 0, 0x0a, pin_state::Unused        , pin_state::Unused;
    pio0_11, PIO0_11, 0, 0x0b, pin_state::Unused        , pin_state::Unused;
    pio0_12, PIO0_12, 0, 0x0c, pin_state::Unused        , pin_state::Unused;
    pio0_13, PIO0_13, 0, 0x0d, pin_state::Unused        , pin_state::Unused;
    pio0_14, PIO0_14, 0, 0x0e, pin_state::Unused        , pin_state::Unused;
    pio0_15, PIO0_15, 0, 0x0f, pin_state::Unused        , pin_state::Unused;
    pio0_16, PIO0_16, 0, 0x10, pin_state::Unused        , pin_state::Unused;
    pio0_17, PIO0_17, 0, 0x11, pin_state::Unused        , pin_state::Unused;
    pio0_18, PIO0_18, 0, 0x12, pin_state::Unused        , pin_state::Unused;
    pio0_19, PIO0_19, 0, 0x13, pin_state::Unused        , pin_state::Unused;
    pio0_20, PIO0_20, 0, 0x14, pin_state::Unused        , pin_state::Unused;
    pio0_21, PIO0_21, 0, 0x15, pin_state::Unused        , pin_state::Unused;
    pio0_22, PIO0_22, 0, 0x16, pin_state::Unused        , pin_state::Unused;
    pio0_23, PIO0_23, 0, 0x17, pin_state::Unused        , pin_state::Unused;
    pio0_24, PIO0_24, 0, 0x18, pin_state::Unused        , pin_state::Unused;
    pio0_25, PIO0_25, 0, 0x19, pin_state::Unused        , pin_state::Unused;
    pio0_26, PIO0_26, 0, 0x1a, pin_state::Unused        , pin_state::Unused;
    pio0_27, PIO0_27, 0, 0x1b, pin_state::Unused        , pin_state::Unused;
    pio0_28, PIO0_28, 0, 0x1c, pin_state::Unused        , pin_state::Unused;
    pio0_29, PIO0_29, 0, 0x1d, pin_state::Unused        , pin_state::Unused;
    pio0_30, PIO0_30, 0, 0x1e, pin_state::Unused        , pin_state::Unused;
    pio0_31, PIO0_31, 0, 0x1f, pin_state::Unused        , pin_state::Unused;
    pio1_0 , PIO1_0 , 1, 0x00, pin_state::Unused        , pin_state::Unused;
    pio1_1 , PIO1_1 , 1, 0x01, pin_state::Unused        , pin_state::Unused;
    pio1_2 , PIO1_2 , 1, 0x02, pin_state::Unused        , pin_state::Unused;
    pio1_3 , PIO1_3 , 1, 0x03, pin_state::Unused        , pin_state::Unused;
    pio1_4 , PIO1_4 , 1, 0x04, pin_state::Unused        , pin_state::Unused;
    pio1_5 , PIO1_5 , 1, 0x05, pin_state::Unused        , pin_state::Unused;
    pio1_6 , PIO1_6 , 1, 0x06, pin_state::Unused        , pin_state::Unused;
    pio1_7 , PIO1_7 , 1, 0x07, pin_state::Unused        , pin_state::Unused;
    pio1_8 , PIO1_8 , 1, 0x08, pin_state::Unused        , pin_state::Unused;
    pio1_9 , PIO1_9 , 1, 0x09, pin_state::Unused        , pin_state::Unused;
    pio1_10, PIO1_10, 1, 0x0a, pin_state::Unused        , pin_state::Unused;
    pio1_11, PIO1_11, 1, 0x0b, pin_state::Unused        , pin_state::Unused;
    pio1_12, PIO1_12, 1, 0x0c, pin_state::Unused        , pin_state::Unused;
    pio1_13, PIO1_13, 1, 0x0d, pin_state::Unused        , pin_state::Unused;
    pio1_14, PIO1_14, 1, 0x0e, pin_state::Unused        , pin_state::Unused;
    pio1_15, PIO1_15, 1, 0x0f, pin_state::Unused        , pin_state::Unused;
    pio1_16, PIO1_16, 1, 0x10, pin_state::Unused        , pin_state::Unused;
    pio1_17, PIO1_17, 1, 0x11, pin_state::Unused        , pin_state::Unused;
    pio1_18, PIO1_18, 1, 0x12, pin_state::Unused        , pin_state::Unused;
    pio1_19, PIO1_19, 1, 0x13, pin_state::Unused        , pin_state::Unused;
    pio1_20, PIO1_20, 1, 0x14, pin_state::Unused        , pin_state::Unused;
    pio1_21, PIO1_21, 1, 0x15, pin_state::Unused        , pin_state::Unused;
);

/// Main API for controlling pins
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
/// - [`pin_state::Analog`], to indicate that the pin is being used for analog
///   input
///
/// # State Management
///
/// All pins start out in their initial state, as defined in the user manual. To
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
/// need this feature, [please speak up](https://github.com/lpc-rs/lpc8xx-hal/issues/50).
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
/// topic, [please help us figure this out](https://github.com/lpc-rs/lpc8xx-hal/issues/44).
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
/// supported by this API. If you need this feature, [please let us know](https://github.com/lpc-rs/lpc8xx-hal/issues/51)!
///
/// As a wokraround, you can use the raw register mappings from the lpc82x-pac &
/// lpc845-pac crates, [`lpc82x::IOCON`] and [`lpc82x::ADC`], after you have put
/// the pin into the ADC state.
///
/// [`direction::Unknown`]: ../gpio/direction/struct.Unknown.html
/// [`direction::Input`]: ../gpio/direction/struct.Input.html
/// [`direction::Output`]: ../gpio/direction/struct.Output.html
/// [`lpc82x::IOCON`]: https://docs.rs/lpc82x-pac/0.7.*/lpc82x_pac/struct.IOCON.html
/// [`lpc82x::ADC`]: https://docs.rs/lpc82x-pac/0.7.*/lpc82x_pac/struct.ADC.html
pub struct Pin<T: PinTrait, S: PinState> {
    pub(crate) ty: T,
    pub(crate) state: S,
}

impl<T> Pin<T, pin_state::Unused>
where
    T: PinTrait,
{
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
    pub fn into_gpio_pin(
        self,
        gpio: &GPIO,
    ) -> Pin<T, pin_state::Gpio<gpio::direction::Unknown>> {
        // Isn't used for lpc845
        #[allow(unused_imports)]
        use core::slice;
        #[cfg(feature = "82x")]
        let registers = pin_state::GpioRegisters {
            dirset: slice::from_ref(&gpio.gpio.dirset0),
            dirclr: slice::from_ref(&gpio.gpio.dirclr0),
            pin: slice::from_ref(&gpio.gpio.pin0),
            set: slice::from_ref(&gpio.gpio.set0),
            clr: slice::from_ref(&gpio.gpio.clr0),
        };
        #[cfg(feature = "845")]
        let registers = pin_state::GpioRegisters {
            dirset: &gpio.gpio.dirset,
            dirclr: &gpio.gpio.dirclr,
            pin: &gpio.gpio.pin,
            set: &gpio.gpio.set,
            clr: &gpio.gpio.clr,
        };
        Pin {
            ty: self.ty,
            state: pin_state::Gpio {
                registers,
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
            ty: self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T> Pin<T, pin_state::Swm<(), ()>>
where
    T: PinTrait,
{
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
            ty: self.ty,
            state: pin_state::Unused,
        }
    }
}

/// Contains types that indicate pin states
///
/// Please refer to [`Pin`] for documentation about how these types are used.
pub mod pin_state {
    use core::marker::PhantomData;

    use crate::gpio::direction::Direction;
    #[cfg(feature = "845")]
    use crate::pac::gpio::{CLR, DIRCLR, DIRSET, PIN, SET};
    #[cfg(feature = "82x")]
    use crate::pac::gpio::{
        CLR0 as CLR, DIRCLR0 as DIRCLR, DIRSET0 as DIRSET, PIN0 as PIN,
        SET0 as SET,
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
    pub struct Analog;

    impl PinState for Analog {}

    /// Marks a [`Pin`]  as being assigned to general-purpose I/O
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Gpio<'gpio, D: Direction> {
        pub(crate) registers: GpioRegisters<'gpio>,
        pub(crate) _direction: D,
    }

    pub(crate) struct GpioRegisters<'gpio> {
        pub(crate) dirset: &'gpio [DIRSET],
        pub(crate) dirclr: &'gpio [DIRCLR],
        pub(crate) pin: &'gpio [PIN],
        pub(crate) set: &'gpio [SET],
        pub(crate) clr: &'gpio [CLR],
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
