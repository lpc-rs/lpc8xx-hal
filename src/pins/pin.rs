use crate::{
    gpio::{direction, GpioPin, Level},
    init_state,
};

use super::{
    gen::Token,
    state::{self, State},
    traits::PinTrait,
};

/// Main API for controlling pins
///
/// `Pin` has two type parameters:
/// - `T`, to indicate which specific pin this instance of `Pin` represents (so,
///   [`PIO0_0`], [`PIO0_1`], and so on)
/// - `S`, to indicate which state the represented pin is currently in
///
/// A pin instance can be in one of the following states:
/// - [`state::Unused`], to indicate that the pin is currently not used
/// - [`state::Gpio`], to indicate that the pin is being used for
///   general-purpose I/O
/// - [`state::Swm`], to indicate that the pin is available for switch
///   matrix function assignment
/// - [`state::Analog`], to indicate that the pin is being used for analog
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
/// [`PIO0_0`]: struct.PIO0_0.html
/// [`PIO0_1`]: struct.PIO0_1.html
/// [`state::Unused`]: state/struct.Unused.html
/// [`state::Gpio`]: state/struct.Gpio.html
/// [`state::Swm`]: state/struct.Swm.html
/// [`state::Analog`]: state/struct.Analog.html
/// [`direction::Unknown`]: ../gpio/direction/struct.Unknown.html
/// [`direction::Input`]: ../gpio/direction/struct.Input.html
/// [`direction::Output`]: ../gpio/direction/struct.Output.html
/// [`lpc82x::IOCON`]: https://docs.rs/lpc82x-pac/0.7.*/lpc82x_pac/struct.IOCON.html
/// [`lpc82x::ADC`]: https://docs.rs/lpc82x-pac/0.7.*/lpc82x_pac/struct.ADC.html
pub struct Pin<T: PinTrait, S: State> {
    pub(crate) ty: T,
    pub(crate) _state: S,
}

impl<T> Pin<T, state::Unused>
where
    T: PinTrait,
{
    /// Transition pin to GPIO input mode
    pub fn into_input_pin(
        self,
        token: Token<T, init_state::Enabled>,
    ) -> GpioPin<T, direction::Input> {
        GpioPin::new(token, ())
    }

    /// Transition pin to GPIO output mode
    pub fn into_output_pin(
        self,
        token: Token<T, init_state::Enabled>,
        initial: Level,
    ) -> GpioPin<T, direction::Output> {
        GpioPin::new(token, initial)
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
    pub fn into_swm_pin(self) -> Pin<T, state::Swm<(), ()>> {
        Pin {
            ty: self.ty,
            _state: state::Swm::new(),
        }
    }
}

impl<T> Pin<T, state::Swm<(), ()>>
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
    pub fn into_unused_pin(self) -> Pin<T, state::Unused> {
        Pin {
            ty: self.ty,
            _state: state::Unused,
        }
    }
}
