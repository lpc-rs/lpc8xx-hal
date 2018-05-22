//! API for General Purpose I/O (GPIO)
//!
//! The entry point for this API is [`GPIO`]. [`GPIO`] provides access to the
//! [`gpio::Handle`], which you can use to initialize the GPIO peripheral, and
//! to instances of [`Pin`], which allow you to configure pins.
//!
//! The GPIO peripheral is described in the user manual, chapter 9.
//!
//! # Examples
//!
//! Initialize a GPIO pin and set its output to HIGH:
//!
//! ``` no_run
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//!
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::{
//!     GPIO,
//!     SYSCON,
//! };
//!
//! let mut peripherals = lpc82x::Peripherals::take().unwrap();
//!
//! let     gpio   = GPIO::new(peripherals.GPIO_PORT);
//! let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
//!
//! let gpio_handle = gpio.handle.enable(&mut syscon.handle);
//!
//! let pio0_12 = unsafe { gpio.pins.pio0_12.affirm_default_state() }
//!     .into_gpio_pin(&gpio_handle)
//!     .into_output()
//!     .set_high();
//! ```
//!
//! Assign a pin to the switch matrix and enable a fixed function:
//!
//! ``` no_run
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//!
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::{
//!     GPIO,
//!     SWM,
//!     SYSCON,
//! };
//!
//! let mut peripherals = lpc82x::Peripherals::take().unwrap();
//!
//! let     gpio   = GPIO::new(peripherals.GPIO_PORT);
//! let     swm    = SWM::new(peripherals.SWM);
//! let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
//!
//! let mut swm_handle = swm.handle.enable(&mut syscon.handle);
//!
//! let vddcmp = unsafe {
//!     swm.fixed_functions.vddcmp.affirm_default_state()
//! };
//! let pio0_6 = unsafe { gpio.pins.pio0_6.affirm_default_state() }
//!     .into_swm_pin();
//! vddcmp.assign(pio0_6, &mut swm_handle);
//! ```
//!
//! [`GPIO`]: struct.GPIO.html
//! [`Peripherals`]: ../struct.Peripherals.html
//! [`gpio::Handle`]: struct.Handle.html
//! [`Pin`]: struct.Pin.html
//! [`lpc82x::GPIO_PORT`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.GPIO_PORT.html


use embedded_hal::digital::{
    OutputPin,
    StatefulOutputPin,
};

use init_state::{
    self,
    InitState,
};
use raw;
use swm;
use syscon;

use self::pin_state::PinState;


/// Entry point to the GPIO API
///
/// This struct provides access to all types that make up the GPIO API, namely
/// [`gpio::Handle`], which can be used to initialize the GPIO peripheral, and
/// [`Pins`], which provides access to all pins.
///
/// Please refer to the [module documentation] for more information.
///
/// [`gpio::Handle`]: struct.Handle.html
/// [`Pins`]: struct.Pins.html
/// [module documentation]: index.html
pub struct GPIO {
    /// The handle to the GPIO peripheral
    pub handle: Handle<init_state::Unknown,>,

    /// The pins that can be used for GPIO or other functions
    pub pins: Pins,
}

impl GPIO {
    /// Create an instance of `GPIO`
    pub fn new(gpio: raw::GPIO_PORT) -> Self {
        GPIO {
            handle: Handle {
                gpio  : gpio,
                _state: init_state::Unknown,
            },
            pins: Pins::new(),
        }
    }
}


/// The handle to the GPIO peripheral
///
/// This handle can be used to initialize the GPIO peripheral. It has a type
/// parameter to track whether the peripheral has been initialized.
///
/// Please refer to the [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Handle<State: InitState = init_state::Enabled> {
    gpio  : raw::GPIO_PORT,
    _state: State,
}

impl<'gpio, State> Handle<State> where State: init_state::NotEnabled {
    /// Enable the GPIO peripheral
    ///
    /// Enables the clock and clears the peripheral reset for the GPIO
    /// peripheral.
    ///
    /// This method is only available, if `gpio::Handle` is not already in the
    /// [`Enabled`] state. Code that attempts to call this method when the GPIO
    /// peripheral is already enabled will not compile.
    ///
    /// Consumes this instance of `gpio::Handle` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(mut self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Enabled>
    {
        syscon.enable_clock(&mut self.gpio);
        syscon.clear_reset(&mut self.gpio);

        Handle {
            gpio  : self.gpio,
            _state: init_state::Enabled,
        }
    }
}

impl<State> Handle<State> where State: init_state::NotDisabled {
    /// Disable the GPIO peripheral
    ///
    /// This method is only available, if `gpio::Handle` is not already in the
    /// [`Disabled`] state. Code that attempts to call this method when the GPIO
    /// peripheral is already disabled will not compile.
    ///
    /// Consumes this instance of `gpio::Handle` and returns another instance
    /// that has its `State` type parameter set to [`Disabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(mut self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Disabled>
    {
        syscon.disable_clock(&mut self.gpio);

        Handle {
            gpio  : self.gpio,
            _state: init_state::Disabled,
        }
    }
}


/// Represents a specific pin
///
/// This trait is implemented by all types that represent a specific pin.
/// Instances of those types are not directly accessible by the user, but the
/// types show up as a type parameter on [`Pin`].
///
/// HAL users shouldn't need to implement this trait themselves. Nor should they
/// need to use it directly, unless compensating for missing functionality.
/// Therefore any changes to this trait won't be considered breaking changes.
///
/// [`Pin`]: struct.Pin.html
pub trait PinTrait {
    /// The default state of the pin after microcontroller initialization
    type DefaultState: PinState;

    /// A number that identifies the pin
    ///
    /// This is `0` for [`PIO0_0`], `1` for [`PIO0_1`] and so forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    const ID: u8;

    /// The pin's bit mask
    ///
    /// This is `0x00000001` for [`PIO0_0`], `0x00000002` for [`PIO0_1`] and so
    /// forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    const MASK: u32;

    /// The initial value of the pin state after microcontroller initialization
    const INITIAL_STATE: Self::DefaultState;
}


macro_rules! pins {
    ($(
        $field:ident,
        $type:ident,
        $id:expr,
        $default_state:ty,
        $default_state_val:expr;
    )*) => {
        /// Provides access to all pins
        ///
        /// Has one field for each pin. Please refer to the documentation of
        /// [`Pin`] to learn more about how to use them.
        ///
        /// This struct is a part of [`GPIO`].
        ///
        /// # Limitations
        ///
        /// This struct currently provides access to all pins that can be
        /// available on an LPC82x part. Please make sure that you are aware of
        /// which pins are actually available on your specific part, and only
        /// use those.
        ///
        /// [`Pin`]: struct.Pin.html
        /// [`GPIO`]: struct.GPIO.html
        #[allow(missing_docs)]
        pub struct Pins {
            $(pub $field: Pin<$type, pin_state::Unknown>,)*
        }

        impl Pins {
            fn new() -> Self {
                Pins {
                    $(
                        $field: Pin {
                            ty   : $type(()),
                            state: pin_state::Unknown,
                        },
                    )*
                }
            }
        }


        $(
            /// Identifies a specific pin
            ///
            /// Users of this crate usually won't have direct access to this
            /// struct, as it is owned by an instance of [`Pin`]. You can use it
            /// in type signatures, to indicate that you need a [`Pin`] instance
            /// representing a specific pin.
            ///
            /// Please refer to the documentation of [`Pin`] for more
            /// information.
            ///
            /// [`Pin`]: struct.Pin.html
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl PinTrait for $type {
                type DefaultState = $default_state;

                const ID  : u8  = $id;
                const MASK: u32 = 0x1 << $id;

                const INITIAL_STATE: Self::DefaultState = $default_state_val;
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


/// Provides access to pin functionality
///
/// `Pin` has two type parameters:
/// - `T`, to indicate which specific pin this instance of `Pin` represents (so,
///   [`PIO0_0`], [`PIO0_1`], and so on)
/// - `S`, to indicate which state the represented pin is currently in
///
/// A pin instance can be in one of the following states:
/// - [`pin_state::Unknown`], to indicate that the current state of the pin is
///   not known
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
/// All pins start out in the [`pin_state::Unknown`] state, as the HAL API can't
/// know what happened to them before the API was initialized. To start using
/// them, you need to promise the API that they are still in their default
/// state, using [`affirm_default_state`].
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::GPIO;
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let gpio = GPIO::new(peripherals.GPIO_PORT);
/// #
/// use lpc82x_hal::gpio::{
///     PIO0_12,
///     pin_state,
///     Pin,
/// };
///
/// // The pin starts out in the unknown state
/// let pin: Pin<PIO0_12, pin_state::Unknown> = gpio.pins.pio0_12;
///
/// // After we promise we didn't mess with the pin, the API knows it's unused
/// let pin: Pin<PIO0_12, pin_state::Unused> =
///     unsafe { pin.affirm_default_state() };
/// ```
///
/// Once the API knows the pin's state, we can use its methods to configure it.
/// To prevent us from making mistakes, only the methods that induce a valid
/// state transition are available. Code that tries to call a method that would
/// cause an invalid state transition will simply not compile.
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::{
/// #     GPIO,
/// #     SWM,
/// #     SYSCON,
/// # };
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
/// # let     swm    = SWM::new(peripherals.SWM);
/// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
/// #
/// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
/// #
/// // Reassure the API that the pin is in its default state, i.e. unused.
/// let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() };
///
/// // Assign a movable function to this pin
/// let clkout = unsafe {
///     swm.movable_functions.clkout.affirm_default_state()
/// };
/// let (_, pin) = clkout.assign(pin.into_swm_pin(), &mut swm_handle);
///
/// // As long as the movable function is assigned, we can't use the pin for
/// // general-purpose I/O. Therefore the following method call would cause a
/// // compile-time error.
/// // pin.into_gpio_pin(&gpio);
/// ```
///
/// To use the pin in the above example for GPIO, we first have to unassign the
/// movable function and transition the pin to the unused state using
/// [`into_unused_pin`].
///
/// # General Purpose I/O
///
/// All pins can be used for general-purpose I/O (GPIO), meaning they can be
/// used for reading digital input signals and writing digital output signals.
/// To set up a pin for GPIO use, you need to call [`into_gpio_pin`] when it is
/// in its unused state.
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::{
/// #     GPIO,
/// #     SWM,
/// #     SYSCON,
/// # };
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
/// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
/// #
/// // To use general-purpose I/O, we need to enable the GPIO peripheral. The
/// // call to `into_gpio_pin` below enforces this by requiring a reference to
/// // an enabled GPIO handle.
/// let gpio_handle = gpio.handle.enable(&mut syscon.handle);
///
/// // Affirm that pin is unused, then transition to the GPIO state
/// let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() }
///     .into_gpio_pin(&gpio_handle);
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
/// The same logic that applies to the overall pin state applies to the GPIO
/// state, too: We can't know what happened to the pin configuration before the
/// API was initialized, so we start out with [`direction::Unknown`].
///
/// To use a pin, that we previously configured for GPIO (see example above),
/// for digital output, we need to set the pin direction using [`into_output`].
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::{
/// #     GPIO,
/// #     SYSCON,
/// # };
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
/// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
/// #
/// # let gpio_handle = gpio.handle.enable(&mut syscon.handle);
/// #
/// # let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() }
/// #     .into_gpio_pin(&gpio_handle);
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
/// Although this functionality belongs to a different peripheral, the switch
/// matrix (see [`swm`] module and chapter 7 in the user manual), it is highly
/// relevant to the functioning of other pin features, and is therefore part of
/// this API.
///
/// Before you can assign any functions to a pin, you need to transition it from
/// the unused state to the SWM state using [`into_swm_pin`].
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::GPIO;
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let gpio = GPIO::new(peripherals.GPIO_PORT);
/// #
/// // Affirm that the pin is unused, then transition to the SWM state
/// let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() }
///     .into_swm_pin();
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
/// Once a pin is in the SWM state, you can use the various function-related
/// methods to enable the pin's fixed function, or assign movable functions.
/// Please refer to the documentation of the [`swm`] module to learn how to
/// access the fixed and movable functions, and transition them into the right
/// state.
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::{
/// #     GPIO,
/// #     SWM,
/// #     SYSCON,
/// # };
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
/// # let     swm    = SWM::new(peripherals.SWM);
/// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
/// #
/// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
/// #
/// # let xtalout = unsafe {
/// #     swm.fixed_functions.xtalout.affirm_default_state()
/// # };
/// # let u0_rxd = unsafe {
/// #     swm.movable_functions.u0_rxd.affirm_default_state()
/// # };
/// # let u1_rxd = unsafe {
/// #     swm.movable_functions.u1_rxd.affirm_default_state()
/// # };
/// # let u0_txd = unsafe {
/// #     swm.movable_functions.u0_txd.affirm_default_state()
/// # };
/// #
/// // Put PIO0_9 into the SWM state
/// let pin = unsafe { gpio.pins.pio0_9.affirm_default_state() }
///     .into_swm_pin();
///
/// // Enable this pin's fixed function, which is an output function.
/// let (xtalout, pin) = xtalout.assign(pin, &mut swm_handle);
///
/// // Now we can assign various input functions in addition.
/// let (_, pin) = u0_rxd.assign(pin, &mut swm_handle);
/// let (_, pin) = u1_rxd.assign(pin, &mut swm_handle);
///
/// // We can't assign another output function. The next line won't compile.
/// // let (_, pin) = u0_txd.assign(pin, &mut swm_handle);
///
/// // Once we disabled the currently enabled output function, we can assign
/// // another output function.
/// let (pin, _) = pin.unassign_function(xtalout, &mut swm_handle);
/// let (_, pin) = u0_txd.assign(pin, &mut swm_handle);
/// ```
///
/// # Analog Input
///
/// To use a pin for analog input, you need to assign an ADC function.
///
/// ``` no_run
/// # extern crate lpc82x;
/// # extern crate lpc82x_hal;
/// #
/// # use lpc82x_hal::{
/// #     GPIO,
/// #     SWM,
/// #     SYSCON,
/// # };
/// #
/// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
/// #
/// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
/// # let     swm    = SWM::new(peripherals.SWM);
/// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
/// #
/// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
/// #
/// # let adc_2 = unsafe {
/// #     swm.fixed_functions.adc_2.affirm_default_state()
/// # };
/// #
/// // Transition pin into ADC state
/// let pio0_14 = unsafe { gpio.pins.pio0_14.affirm_default_state() }
///     .into_swm_pin();
/// adc_2.assign(pio0_14, &mut swm_handle);
/// ```
///
/// Using the pin for analog input once it is in the ADC state is currently not
/// supported by this API. If you need this feature, [please let us know](https://github.com/braun-robotics/rust-lpc82x-hal/issues/51)!
///
/// As a woraround, you can use the raw register mappings from the lpc82x crate,
/// [`lpc82x::IOCON`] and [`lpc82x::ADC`], after you have put the pin into the
/// ADC state.
///
/// [`PIO0_0`]: struct.PIO0_0.html
/// [`PIO0_1`]: struct.PIO0_1.html
/// [`pin_state::Unknown`]: pin_state/struct.Unknown.html
/// [`pin_state::Unused`]: pin_state/struct.Unused.html
/// [`pin_state::Adc`]: pin_state/struct.Adc.html
/// [`pin_state::Gpio`]: pin_state/struct.Gpio.html
/// [`pin_state::Swm`]: pin_state/struct.Swm.html
/// [`affirm_default_state`]: #method.affirm_default_state
/// [`into_unused_pin`]: #method.into_unused_pin
/// [`into_gpio_pin`]: #method.into_gpio_pin
/// [`direction::Unknown`]: direction/struct.Unknown.html
/// [`direction::Input`]: direction/struct.Input.html
/// [`direction::Output`]: direction/struct.Output.html
/// [`into_output`]: #method.into_output
/// [`swm`]: ../swm/index.html
/// [`into_swm_pin`]: #method.into_swm_pin
/// [`lpc82x::IOCON`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.IOCON.html
/// [`lpc82x::ADC`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.ADC.html
pub struct Pin<T: PinTrait, S: PinState> {
    pub(crate) ty   : T,
               state: S,
}

impl<T> Pin<T, pin_state::Unknown> where T: PinTrait {
    /// Affirm that the pin is in its default state
    ///
    /// This method is only available, if the pin is in the unknown state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile.
    ///
    /// By calling this method, you promise that the pin's configuration has not
    /// been changed from its default. For most pins, this means that the pin is
    /// unused, but some pins are initially assigned to the switch matrix. This
    /// method consumes the current pin instance and returns a new instance
    /// whose state matches the pin's default configuration.
    ///
    /// Unless you have changed the pin's configuration before initializing the
    /// HAL API, or have called some code that might have changed the
    /// configuration, this method is safe to call.
    ///
    /// # Safety
    ///
    /// You MUST NOT call this method, if the pin configuration has been changed
    /// from its default configuration. You can call this method again, after
    /// you restore the pin to its default configuration.
    ///
    /// Calling this method while the pin's configuration deviates from the
    /// default will create a `Pin` instance whose state doesn't match the
    /// actual pin configuration. This can lead to any number of problems.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// # extern crate lpc82x;
    /// # extern crate lpc82x_hal;
    /// #
    /// # use lpc82x_hal::{
    /// #     GPIO,
    /// #     SWM,
    /// #     SYSCON,
    /// # };
    /// #
    /// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
    /// #
    /// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
    /// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
    /// # let mut swm    = SWM::new(peripherals.SWM);
    /// #
    /// # let swclk = unsafe {
    /// #     swm.fixed_functions.swclk.affirm_default_state()
    /// # };
    /// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
    /// #
    /// // These pins are in the unknown state. As long as that's the case, we
    /// // can't do anything useful with them.
    /// let pio0_3  = gpio.pins.pio0_3;
    /// let pio0_12 = gpio.pins.pio0_12;
    ///
    /// // Since we didn't change the pin configuration, nor called any code
    /// // that did, we can safely affirm that the pins are in their default
    /// // state.
    /// let pio0_3  = unsafe { pio0_3.affirm_default_state()  };
    /// let pio0_12 = unsafe { pio0_12.affirm_default_state() };
    ///
    /// // PIO0_12 happens to be unused by default, which means it is ready to
    /// // be transitioned into another state now. However, PIO0_3 has its fixed
    /// // function enabled by default. If we want to use it for something else,
    /// // we need to transition it into the unused state before we can do so.
    /// let pio0_3 = pio0_3
    ///     .unassign_function(swclk, &mut swm_handle)
    ///     .0 // also returns output function; we're only interested in pin
    ///     .into_unused_pin();
    /// ```
    pub unsafe fn affirm_default_state(self) -> Pin<T, T::DefaultState> {
        Pin {
            ty   : self.ty,
            state: T::INITIAL_STATE,
        }
    }
}

impl<T> Pin<T, pin_state::Unused> where T: PinTrait {
    /// Transition this pin instance to the GPIO state
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes the pin instance and returns a new instance that is in the GPIO
    /// state, allowing you to use the pin for general-purpose I/O. As long as
    /// the pin is in the GPIO state, it needs the GPIO peripheral to be enabled
    /// to function correctly. To statically guarantee that this is the case,
    /// this method takes a shared reference to [`gpio::Handle`], which the pin
    /// keeps around until it leaves the GPIO state.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// # extern crate lpc82x;
    /// # extern crate lpc82x_hal;
    /// #
    /// # use lpc82x_hal::{
    /// #     GPIO,
    /// #     SYSCON,
    /// # };
    /// #
    /// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
    /// #
    /// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
    /// #
    /// let gpio        = GPIO::new(peripherals.GPIO_PORT);
    /// let gpio_handle = gpio.handle.enable(&mut syscon.handle);
    ///
    /// let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() }
    ///     .into_gpio_pin(&gpio_handle);
    ///
    /// // `pin` is now available for general-purpose I/O
    /// ```
    ///
    /// [State Management]: #state-management
    /// [`gpio::Handle`]: struct.Handle.html
    pub fn into_gpio_pin(self, gpio: &Handle)
        -> Pin<T, pin_state::Gpio<direction::Unknown>>
    {
        Pin {
            ty   : self.ty,
            state: pin_state::Gpio {
                dirset0: &gpio.gpio.dirset0,
                pin0   : &gpio.gpio.pin0,
                set0   : &gpio.gpio.set0,
                clr0   : &gpio.gpio.clr0,

                _direction: direction::Unknown,
            },
        }
    }

    /// Transition this pin instance to to the SWM state
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes the pin instance and returns a new instance that is in the SWM
    /// state, making this pin available for switch matrix function assignment.
    /// 
    /// Make this pin available for function assignment by the switch matrix.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// # extern crate lpc82x;
    /// # extern crate lpc82x_hal;
    /// #
    /// # use lpc82x_hal::GPIO;
    /// #
    /// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
    /// #
    /// let gpio = GPIO::new(peripherals.GPIO_PORT);
    ///
    /// let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() }
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

impl<'gpio, T, D> Pin<T, pin_state::Gpio<'gpio, D>>
    where
        T: PinTrait,
        D: direction::NotOutput,
{
    /// Sets pin direction to output
    ///
    /// This method is only available, if the pin is in the GPIO state and the
    /// pin is not already in output mode, i.e. the pin direction is input or
    /// unknown. You can enter the GPIO state using [`into_gpio_pin`].
    ///
    /// Consumes the pin instance and returns a new instance that is in output
    /// mode, making the methods to set the output level available.
    ///
    /// **NOTE**: There are some doubts about whether this is the right API
    /// design. [Please leave your feedback](https://github.com/braun-robotics/rust-lpc82x-hal/issues/53),
    /// if you have anything to say about this.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// # extern crate lpc82x;
    /// # extern crate lpc82x_hal;
    /// #
    /// # use lpc82x_hal::{
    /// #     GPIO,
    /// #     SYSCON,
    /// # };
    /// #
    /// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
    /// #
    /// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
    /// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
    /// #
    /// # let gpio_handle = gpio.handle.enable(&mut syscon.handle);
    /// #
    /// # let pin = unsafe { gpio.pins.pio0_12.affirm_default_state() }
    /// #     .into_gpio_pin(&gpio_handle);
    /// #
    /// use lpc82x_hal::prelude::*;
    ///
    /// // Assumes the pin is already in the GPIO state
    /// let mut pin = pin.into_output();
    ///
    /// // Output level can now be set
    /// pin.set_high();
    /// pin.set_low();
    /// ```
    ///
    /// [`into_gpio_pin`]: #method.into_gpio_pin
    pub fn into_output(self)
        -> Pin<T, pin_state::Gpio<'gpio, direction::Output>>
    {
        self.state.dirset0.write(|w|
            unsafe { w.dirsetp().bits(T::MASK) }
        );

        Pin {
            ty: self.ty,

            state: pin_state::Gpio {
                dirset0: self.state.dirset0,
                pin0   : self.state.pin0,
                set0   : self.state.set0,
                clr0   : self.state.clr0,

                _direction: direction::Output,
            }
        }
    }
}

impl<'gpio, T> OutputPin for Pin<T, pin_state::Gpio<'gpio, direction::Output>>
    where T: PinTrait
{
    /// Set the pin output to HIGH
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state. Use [`into_gpio_pin`] to achieve this.
    /// - The pin direction is set to output. See [`into_output`].
    ///
    /// Unless both of these conditions are met, code trying to call this method
    /// will not compile.
    ///
    /// [`into_gpio_pin`]: #method.into_gpio_pin
    /// [`into_output`]: #method.into_output
    fn set_high(&mut self) {
        self.state.set0.write(|w|
            unsafe { w.setp().bits(T::MASK) }
        )
    }

    /// Set the pin output to LOW
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state. Use [`into_gpio_pin`] to achieve this.
    /// - The pin direction is set to output. See [`into_output`].
    ///
    /// Unless both of these conditions are met, code trying to call this method
    /// will not compile.
    ///
    /// [`into_gpio_pin`]: #method.into_gpio_pin
    /// [`into_output`]: #method.into_output
    fn set_low(&mut self) {
        self.state.clr0.write(|w|
            unsafe { w.clrp().bits(T::MASK) }
        );
    }
}

impl<'gpio, T> StatefulOutputPin
    for Pin<T, pin_state::Gpio<'gpio, direction::Output>>
    where T: PinTrait
{
    /// Indicates whether the pin output is currently set to HIGH
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state. Use [`into_gpio_pin`] to achieve this.
    /// - The pin direction is set to output. See [`into_output`].
    ///
    /// Unless both of these conditions are met, code trying to call this method
    /// will not compile.
    ///
    /// [`into_gpio_pin`]: #method.into_gpio_pin
    /// [`into_output`]: #method.into_output
    fn is_set_high(&self) -> bool {
        self.state.pin0.read().port().bits() & T::MASK == T::MASK
    }

    /// Indicates whether the pin output is currently set to LOW
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state. Use [`into_gpio_pin`] to achieve this.
    /// - The pin direction is set to output. See [`into_output`].
    ///
    /// Unless both of these conditions are met, code trying to call this method
    /// will not compile.
    ///
    /// [`into_gpio_pin`]: #method.into_gpio_pin
    /// [`into_output`]: #method.into_output
    fn is_set_low(&self) -> bool {
        !self.state.pin0.read().port().bits() & T::MASK == T::MASK
    }
}

impl<T, State> Pin<T, State>
    where
        T    : PinTrait,
        State: PinState,
{
    /// Unassign a movable output function from this pin
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the SWM state. Use [`into_swm_pin`] to achieve this.
    /// - An output function, either fixed or movable, is enabled on or assigned
    ///   to this pin. Please refer to [`swm::OutputFunction`] to learn which
    ///   fixed and movable functions are output functions.
    ///
    /// Unless both of these conditions are met, code trying to call this method
    /// will not compile.
    ///
    /// Consumes the pin instance and an instance of the movable function, and
    /// returns a tuple containing
    /// - a new pin instance, its type state indicating that no output function
    ///   is enabled; and
    /// - a new instance of the movable function, its state indicating that it
    ///   is not assigned to any pin. Please refer to the [`swm`] module to
    ///   learn more about movable function states.
    ///
    /// Even though this method is available, if any output function is enabled
    /// on this pin, it only accepts a movable function as a parameter, whose
    /// state indicates that it is assigned to this specific pin. Code that
    /// tries to unassign a movable function that isn't assigned to this pin
    /// will not compile.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// # extern crate lpc82x;
    /// # extern crate lpc82x_hal;
    /// #
    /// # use lpc82x_hal::{
    /// #     GPIO,
    /// #     SWM,
    /// #     SYSCON,
    /// # };
    /// #
    /// # let mut peripherals = lpc82x::Peripherals::take().unwrap();
    /// #
    /// # let     gpio   = GPIO::new(peripherals.GPIO_PORT);
    /// # let     swm    = SWM::new(peripherals.SWM);
    /// # let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
    /// #
    /// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
    /// #
    /// # let pio0_9 = unsafe { gpio.pins.pio0_9.affirm_default_state() };
    /// # let pio0_9 = pio0_9.into_swm_pin();
    /// #
    /// # let u0_txd = unsafe {
    /// #     swm.movable_functions.u0_txd.affirm_default_state()
    /// # };
    /// #
    /// # let (u0_txd, pio0_9) = u0_txd.assign(
    /// #     pio0_9,
    /// #     &mut swm_handle,
    /// # );
    /// #
    /// // Assumes that U0_TXD is assigned to PIO0_9
    /// let (pio0_9, u0_txd) = pio0_9.unassign_function(
    ///     u0_txd,
    ///     &mut swm_handle,
    /// );
    ///
    /// // Both PIO0_9 and U0_TXD are now available again
    /// ```
    ///
    /// [`into_swm_pin`]: #method.into_swm_pin
    /// [`swm::OutputFunction`]: ../swm/trait.OutputFunction.html
    /// [`swm`]: ../swm/index.html
    pub fn unassign_function<F, K>(mut self,
        function: swm::Function<F, swm::state::Assigned<T>>,
        swm     : &mut swm::Handle,
    )
        -> (
            <Self as swm::UnassignFunction<F, K>>::Unassigned,
            swm::Function<F, swm::state::Unassigned>,
        )
        where
            Self: swm::UnassignFunction<F, K>,
            F   : swm::FunctionTrait<T, Kind=K>,
            K   : swm::FunctionKind,
    {
        use swm::UnassignFunction;

        let function = function.unassign(&mut self.ty, swm);

        (self.unassign(), function)
    }
}

impl<T> Pin<T, pin_state::Swm<(), ()>> where T: PinTrait {
    /// Transitions this pin instance from the SWM state to the unused state
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the SWM state.
    /// - No input or output functions, fixed or movable, are assigned to this
    ///   pin.
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


impl<T, F, Output, Inputs> swm::AssignFunction<F, swm::Input>
    for Pin<T, pin_state::Swm<Output, Inputs>>
    where
        T: PinTrait,
        F: swm::FunctionTrait<T, Kind=swm::Input>,
{
    type Assigned = Pin<T, pin_state::Swm<Output, (Inputs,)>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Inputs> swm::AssignFunction<F, swm::Output>
    for Pin<T, pin_state::Swm<(), Inputs>>
    where
        T: PinTrait,
        F: swm::FunctionTrait<T, Kind=swm::Output>,
{
    type Assigned = Pin<T, pin_state::Swm<((),), Inputs>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Output, Inputs> swm::UnassignFunction<F, swm::Input>
     for Pin<T, pin_state::Swm<Output, (Inputs,)>>
     where
        T: PinTrait,
        F: swm::FunctionTrait<T, Kind=swm::Output>,
{
    type Unassigned = Pin<T, pin_state::Swm<Output, Inputs>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F, Inputs> swm::UnassignFunction<F, swm::Output>
     for Pin<T, pin_state::Swm<((),), Inputs>>
     where
        T: PinTrait,
        F: swm::FunctionTrait<T, Kind=swm::Output>,
{
    type Unassigned = Pin<T, pin_state::Swm<(), Inputs>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        }
    }
}

impl<T, F> swm::AssignFunction<F, swm::Adc>
    for Pin<T, pin_state::Swm<(), ()>>
    where
        T: PinTrait,
        F: swm::FunctionTrait<T, Kind=swm::Adc>,
{
    type Assigned = Pin<T, pin_state::Adc>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty   : self.ty,
            state: pin_state::Adc,
        }
    }
}


/// Contains types to indicate pin states
///
/// Please refer to [`Pin`] for documentation on how these types are used.
///
/// [`Pin`]: ../struct.Pin.html
pub mod pin_state {
    use core::marker::PhantomData;

    use raw::gpio_port::{
        CLR0,
        DIRSET0,
        PIN0,
        SET0,
    };

    use super::direction::Direction;


    /// Implemented by types that indicate pin state
    ///
    /// [`Pin`] uses this type as a trait bound for the type parameter that
    /// indicates the pin's state. This is done for the purpose of
    /// documentation, to show which states a pin can be in. Other than that,
    /// this trait should not be relevant to users of this crate.
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub trait PinState {}


    /// Marks a pin's state as being unknown
    ///
    /// As the HAL API can't know what happened to the hardware before the HAL
    /// was initializized, this is the initial state of all pins.
    ///
    /// Please refer to [`Pin`] to see how this type is used.
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Unknown;

    impl PinState for Unknown {}


    /// Marks the pin as being unused
    ///
    /// Please refer to [`Pin`] to see how this type is used.
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Unused;

    impl PinState for Unused {}


    /// Marks the pin as being assigned to the analog-to-digital converter
    ///
    /// Please refer to [`Pin`] to see how this type is used.
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub struct Adc;

    impl PinState for Adc {}


    /// Marks a pin as being assigned to general-purpose I/O
    ///
    /// Please refer to [`Pin`] to see how this type is used.
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


    /// Marks a ping as being available for switch matrix function assigment
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
    /// Please refer to [`Pin`] for more information on how this type is used.
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


/// Contains types to indicate the direction of GPIO pins
///
/// Please refer to [`Pin`] for documentation on how these types are used.
///
/// [`Pin`]: ../struct.Pin.html
pub mod direction {
    /// Implemented by types that indicate GPIO pin direction
    ///
    /// The [`Gpio`] type uses this trait as a bound for its type parameter.
    /// This is done for documentation purposes, to clearly show which types can
    /// be used for this parameter. Other than that, this trait should not be
    /// relevant to users of this crate.
    ///
    /// [`Gpio`]: ../pin_state/struct.Gpio.html
    pub trait Direction {}

    /// Marks a GPIO pin's direction as being unknown
    ///
    /// This type is used as a type parameter of [`Gpio`], which in turn is used
    /// as a type parameter of [`Pin`]. Please refer to the documentation of
    /// [`Pin`] to see how this type is used.
    ///
    /// As we can't know what happened to the hardware before the HAL was
    /// initialized, this is the initial state of GPIO pins.
    ///
    /// [`Gpio`]: ../pin_state/struct.Gpio.html
    /// [`Pin`]: ../struct.Pin.html
    pub struct Unknown;
    impl Direction for Unknown {}

    /// Marks a GPIO pin as being configured for input
    ///
    /// This type is used as a type parameter of [`Gpio`], which in turn is used
    /// as a type parameter of [`Pin`]. Please refer to the documentation of
    /// [`Pin`] to see how this type is used.
    ///
    /// [`Gpio`]: ../pin_state/struct.Gpio.html
    /// [`Pin`]: ../struct.Pin.html
    pub struct Input;
    impl Direction for Input {}

    /// Marks a GPIO pin as being configured for output
    ///
    /// This type is used as a type parameter of [`Gpio`], which in turn is used
    /// as a type parameter of [`Pin`]. Please refer to the documentation of
    /// [`Pin`] to see how this type is used.
    ///
    /// [`Gpio`]: ../pin_state/struct.Gpio.html
    /// [`Pin`]: ../struct.Pin.html
    pub struct Output;
    impl Direction for Output {}


    /// Marks a direction as not being output (i.e. being unknown or input)
    ///
    /// This is a helper trait used only to prevent some code duplication in
    /// [`Pin`] by allowing `impl` blocks to be defined precisely. It should not
    /// be relevant to users of this crate.
    ///
    /// [`Pin`]: ../struct.Pin.html
    pub trait NotOutput: Direction {}

    impl NotOutput for Unknown {}
    impl NotOutput for Input {}
}
