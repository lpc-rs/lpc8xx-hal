use crate::{
    gpio::{direction, GpioPin, Level},
    init_state,
};

use super::{
    gen::Token,
    state::{self, State},
    traits::Trait,
};

/// Main API for controlling pins
///
/// `Pin` has two type parameters:
/// - `T`, to indicate which specific pin this instance of `Pin` represents
///   ([`PIO0_0`], [`PIO0_1`], and so on).
/// - `S`, to indicate which state the represented pin is currently in.
///
/// A pin instance can be in one of the following states:
/// - [`state::Unused`], to indicate that the pin is currently not used.
/// - [`state::Swm`], to indicate that the pin is available for switch
///   matrix function assignment.
/// - [`state::Analog`], to indicate that the pin is being used for analog
///   input.
///
/// A pin that is in the GPIO state is represented by its own struct,
/// [`GpioPin`].
///
/// # State Management
///
/// All pins start out in their initial state, as defined in the user manual. To
/// prevent the user from making a mistake, only the methods that induce a valid
/// state transition are available. Code that tries to call a method that would
/// cause an invalid state transition will simply not compile:
///
/// ``` no_run
/// # use lpc8xx_hal::Peripherals;
/// #
/// # let mut p = Peripherals::take().unwrap();
/// #
/// # let mut syscon = p.SYSCON.split();
/// # let mut swm = p.SWM.split();
/// #
/// # #[cfg(feature = "82x")]
/// # let mut swm_handle = swm.handle;
/// # #[cfg(feature = "845")]
/// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
/// #
/// // Assign a function to a pin
/// let (clkout, pio0_12) = swm.movable_functions.clkout.assign(
///     p.pins.pio0_12.into_swm_pin(),
///     &mut swm_handle,
/// );
///
/// // As long as a function is assigned, we can't use the pin for general-
/// // purpose I/O. Therefore the following method call would cause a compile-
/// // time error.
/// // let pio0_12 = pio0_12.into_input_pin(&p.GPIO);
/// ```
///
/// To use the pin in the above example for GPIO, we first have to unassign the
/// movable function and transition the pin to the unused state:
///
/// ``` no_run
/// # use lpc8xx_hal::Peripherals;
/// #
/// # let mut p = Peripherals::take().unwrap();
/// #
/// # let mut syscon = p.SYSCON.split();
/// # let mut swm = p.SWM.split();
/// #
/// # #[cfg(feature = "82x")]
/// # let mut swm_handle = swm.handle;
/// # #[cfg(feature = "845")]
/// # let mut swm_handle = swm.handle.enable(&mut syscon.handle);
/// #
/// # let (clkout, pio0_12) = swm.movable_functions.clkout.assign(
/// #     p.pins.pio0_12.into_swm_pin(),
/// #     &mut swm_handle,
/// # );
/// #
/// # #[cfg(feature = "82x")]
/// # let gpio = p.GPIO;
/// # #[cfg(feature = "845")]
/// # let gpio = p.GPIO.enable(&mut syscon.handle);
///
/// let (clkout, pio0_12) = clkout.unassign(pio0_12, &mut swm_handle);
/// let pio0_12 = pio0_12.into_unused_pin();
///
/// // Now we can transition the pin into a GPIO state.
/// let pio0_12 = pio0_12.into_input_pin(gpio.tokens.pio0_12);
/// ```
///
/// # General Purpose I/O
///
/// All pins can be used for general-purpose I/O (GPIO), meaning they can be
/// used for reading digital input signals and writing digital output signals.
/// To set up a pin for GPIO use, you need to call [`Pin::into_input_pin`] or
/// [`Pin::into_output_pin`] when it is in its unused state.
///
/// This will return a [`GpioPin`], which provides the GPIO API.
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
/// # use lpc8xx_hal::Peripherals;
/// #
/// # let p = Peripherals::take().unwrap();
/// #
/// let pin = p.pins.pio0_12
///     .into_swm_pin();
///
/// // Functions can be assigned now using the SWM API
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
/// to the [SWM API] for more information on how to do that.
///
/// # Analog Input
///
/// To use a pin for analog input, you need to assign an ADC function:
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
/// // Transition pin to ADC state
/// let (adc_2, pio0_14) = swm.fixed_functions.adc_2.assign(
///     p.pins.pio0_14.into_swm_pin(),
///     &mut swm_handle,
/// );
/// ```
///
/// [`PIO0_0`]: struct.PIO0_0.html
/// [`PIO0_1`]: struct.PIO0_1.html
/// [`state::Unused`]: state/struct.Unused.html
/// [`state::Gpio`]: state/struct.Gpio.html
/// [`state::Swm`]: state/struct.Swm.html
/// [`state::Analog`]: state/struct.Analog.html
/// [`Pin::into_input_pin`]: struct.Pin.html#method.into_input_pin
/// [`Pin::into_output_pin`]: struct.Pin.html#method.into_output_pin
/// [`GpioPin`]: ../gpio/struct.GpioPin.html
/// [`Pin::into_swm_pin`]: struct.Pin.html#method.into_swm_pin
/// [SWM API]: ../swm/index.html
pub struct Pin<T: Trait, S: State> {
    pub(crate) ty: T,
    pub(crate) _state: S,
}

/// Marks the current directin of a Dynamic Pin.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DynamicPinDirection {
    /// Pin is currently Input
    Input,

    /// Pin is currently Output
    Output,
}

impl<T> Pin<T, state::Unused>
where
    T: Trait,
{
    /// Transition pin to GPIO input mode
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes this `Pin` instance and returns an instance of [`GpioPin`],
    /// which provides access to all GPIO functions.
    ///
    /// This method requires a GPIO token from the [`GPIO`] struct, to ensure
    /// that the GPIO peripheral is enabled, and stays enabled while the pin is
    /// in the GPIO mode.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc8xx_hal::prelude::*;
    /// use lpc8xx_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let mut syscon = p.SYSCON.split();
    /// let swm = p.SWM.split();
    ///
    /// #[cfg(feature = "82x")]
    /// let gpio = p.GPIO;
    /// #[cfg(feature = "845")]
    /// let gpio = p.GPIO.enable(&mut syscon.handle);
    ///
    /// // Transition pin into GPIO state, then set it to output
    /// let mut pin = p.pins.pio0_12
    ///     .into_input_pin(gpio.tokens.pio0_12);
    ///
    /// // Input level can now be read
    /// if pin.is_high() {
    ///     // The pin is high
    /// } else {
    ///     // The pin is low
    /// }
    /// ```
    ///
    /// [State Management]: #state-management
    /// [`GpioPin`]: ../gpio/struct.GpioPin.html
    /// [`GPIO`]: ../gpio/struct.GPIO.html
    pub fn into_input_pin(
        self,
        token: Token<T, init_state::Enabled>,
    ) -> GpioPin<T, direction::Input> {
        GpioPin::new(token, ())
    }

    /// Transition pin to GPIO output mode
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes this `Pin` instance and returns an instance of [`GpioPin`],
    /// which provides access to all GPIO functions.
    ///
    /// This method requires a GPIO token from the [`GPIO`] struct, to ensure
    /// that the GPIO peripheral is enabled, and stays enabled while the pin is
    /// in the GPIO mode.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc8xx_hal::{
    ///     prelude::*,
    ///     Peripherals,
    ///     gpio,
    /// };
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let mut syscon = p.SYSCON.split();
    /// let swm = p.SWM.split();
    ///
    /// #[cfg(feature = "82x")]
    /// let gpio = p.GPIO;
    /// #[cfg(feature = "845")]
    /// let gpio = p.GPIO.enable(&mut syscon.handle);
    ///
    /// // Transition pin into GPIO state, then set it to output
    /// let mut pin = p.pins.pio0_12.into_output_pin(
    ///     gpio.tokens.pio0_12,
    ///     gpio::Level::Low,
    /// );
    ///
    /// // Output level can now be controlled
    /// pin.set_high();
    /// pin.set_low();
    /// ```
    ///
    /// [State Management]: #state-management
    /// [`GpioPin`]: ../gpio/struct.GpioPin.html
    /// [`GPIO`]: ../gpio/struct.GPIO.html
    pub fn into_output_pin(
        self,
        token: Token<T, init_state::Enabled>,
        initial: Level,
    ) -> GpioPin<T, direction::Output> {
        GpioPin::new(token, initial)
    }

    /// Transition pin to Dynamic mode, i.e. GPIO direction switchable at runtime
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes this `Pin` instance and returns an instance of [`GpioPin`],
    /// which provides access to all GPIO functions.
    ///
    /// This method requires a GPIO token from the [`GPIO`] struct, to ensure
    /// that the GPIO peripheral is enabled, and stays enabled while the pin is
    /// in the GPIO mode.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc8xx_hal::{
    ///     prelude::*,
    ///     Peripherals,
    ///     gpio,
    ///     pins
    /// };
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let mut syscon = p.SYSCON.split();
    /// let swm = p.SWM.split();
    ///
    /// #[cfg(feature = "82x")]
    /// let gpio = p.GPIO;
    /// #[cfg(feature = "845")]
    /// let gpio = p.GPIO.enable(&mut syscon.handle);
    ///
    /// // Transition pin into GPIO state, then set it to output
    /// let mut pin = p.pins.pio0_12.into_dynamic_pin(
    ///     gpio.tokens.pio0_12,
    ///     gpio::Level::Low,
    ///     pins::DynamicPinDirection::Input,
    /// );
    ///
    /// // Direction can now be switched
    /// pin.switch_to_input();
    ///
    /// // in/output pin functions are available while pin has the matching direction
    /// let is_high = pin.is_high();
    /// let is_low = pin.is_low();
    ///
    /// pin.switch_to_output(gpio::Level::Low);
    /// pin.set_high();
    /// pin.set_low();
    ///
    /// // pin direction can be queried
    /// let is_input = pin.direction_is_input();
    /// let is_output = pin.direction_is_output();
    /// ```
    ///
    /// [State Management]: #state-management
    /// [`GpioPin`]: ../gpio/struct.GpioPin.html
    /// [`GPIO`]: ../gpio/struct.GPIO.html
    pub fn into_dynamic_pin(
        self,
        token: Token<T, init_state::Enabled>,
        level: Level,
        direction: DynamicPinDirection,
    ) -> GpioPin<T, direction::Dynamic> {
        GpioPin::new(token, (level, direction))
    }

    /// Transition pin to SWM mode
    ///
    /// This method is only available while the pin is in the unused state. Code
    /// that attempts to call this method while the pin is in any other state
    /// will not compile. See [State Management] for more information on
    /// managing pin states.
    ///
    /// Consumes this pin instance and returns a new instance that is in the SWM
    /// state, making this pin available for switch matrix function assignment.
    ///
    /// Please refer to the [SWM API] to learn more about SWM function
    /// assignment.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc8xx_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let pin = p.pins.pio0_12
    ///     .into_swm_pin();
    ///
    /// // `pin` is now ready for function assignment
    /// ```
    ///
    /// [State Management]: #state-management
    /// [SWM API]: ../swm/index.html
    pub fn into_swm_pin(self) -> Pin<T, state::Swm<(), ()>> {
        Pin {
            ty: self.ty,
            _state: state::Swm::new(),
        }
    }
}

impl<T> Pin<T, state::Swm<(), ()>>
where
    T: Trait,
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
    pub fn into_unused_pin(self) -> Pin<T, state::Unused> {
        Pin {
            ty: self.ty,
            _state: state::Unused,
        }
    }
}
