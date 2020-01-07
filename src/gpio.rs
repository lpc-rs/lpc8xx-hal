//! API for General Purpose I/O (GPIO)
//!
//! The entry point to this API is [`GPIO`]. It can be used to initialize the
//! peripheral, and is required by instances of [`Pin`] for GPIO functionality.
//! All [`Pin`] instances live in the [`swm`] module.
//!
//! The GPIO peripheral is described in the user manual, chapter 9.
//!
//! # Examples
//!
//! Initialize a GPIO pin and set its output to HIGH:
//!
//! ``` no_run
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::Peripherals;
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let swm = p.SWM.split();
//!
//! let pio0_12 = swm.pins.pio0_12
//!     .into_gpio_pin(&p.GPIO)
//!     .into_output()
//!     .set_high();
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [`swm`]: ../swm/index.html
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/lpc82x-hal/examples

use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin};
use void::Void;

use crate::{
    init_state, pac,
    swm::{pin_state, Pin, PinTrait},
    syscon,
};

/// Interface to the GPIO peripheral
///
/// Controls the GPIO peripheral. Can be used to enable, disable, or free the
/// peripheral. For GPIO-functionality directly related to pins, please refer
/// to [`Pin`].
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct GPIO<State = init_state::Enabled> {
    pub(crate) gpio: pac::GPIO,
    _state: State,
}

impl GPIO<init_state::Enabled> {
    /// Create an enabled gpio peripheral
    ///
    /// This method creates an `GPIO` instance that it assumes is already in the
    /// [`Enabled`] state. It's up to the caller to verify this assumption.
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub unsafe fn new_enabled(gpio: pac::GPIO) -> Self {
        GPIO {
            gpio: gpio,
            _state: init_state::Enabled(()),
        }
    }
}

impl GPIO<init_state::Disabled> {
    /// Create an disabled gpio peripheral
    ///
    /// This method creates an `GPIO` instance that it assumes is in the
    /// [`Disabled`] state. As it's only possible to enable a [`Disabled`] `GPIO`
    /// instance, it's also safe to pass an already [`Enabled`] instance.
    ///
    /// [`Disabled`]: ../init_state/struct.Enabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn new(gpio: pac::GPIO) -> Self {
        GPIO {
            gpio: gpio,
            _state: init_state::Disabled,
        }
    }
    /// Enable the GPIO peripheral
    ///
    /// This method is only available, if `GPIO` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `GPIO` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(mut self, syscon: &mut syscon::Handle) -> GPIO<init_state::Enabled> {
        syscon.enable_clock(&mut self.gpio);

        GPIO {
            gpio: self.gpio,
            _state: init_state::Enabled(()),
        }
    }
}

impl GPIO<init_state::Enabled> {
    /// Disable the GPIO peripheral
    ///
    /// This method is only available, if `GPIO` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `GPIO` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(mut self, syscon: &mut syscon::Handle) -> GPIO<init_state::Disabled> {
        syscon.disable_clock(&mut self.gpio);

        GPIO {
            gpio: self.gpio,
            _state: init_state::Disabled,
        }
    }
}

impl<State> GPIO<State> {
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
    pub fn free(self) -> pac::GPIO {
        self.gpio
    }
}

impl<'gpio, T, D> Pin<T, pin_state::Gpio<'gpio, D>>
where
    T: PinTrait,
    D: direction::NotOutput,
{
    /// Set pin direction to output
    ///
    /// This method is only available, if the pin is in the GPIO state and the
    /// pin is not already in output mode, i.e. the pin direction is input or
    /// unknown. You can enter the GPIO state using [`Pin::into_gpio_pin`].
    ///
    /// Consumes the pin instance and returns a new instance that is in output
    /// mode, making the methods to set the output level available.
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc82x_hal::prelude::*;
    /// use lpc82x_hal::Peripherals;
    ///
    /// let p = Peripherals::take().unwrap();
    ///
    /// let swm = p.SWM.split();
    ///
    /// // Transition pin into GPIO state, then set it to output
    /// let mut pin = swm.pins.pio0_12
    ///     .into_gpio_pin(&p.GPIO)
    ///     .into_output();
    ///
    /// // Output level can now be controlled
    /// pin.set_high();
    /// pin.set_low();
    /// ```
    pub fn into_output(self) -> Pin<T, pin_state::Gpio<'gpio, direction::Output>> {
        self.state.dirset[T::PORT].write(|w| unsafe { w.dirsetp().bits(T::MASK) });

        Pin {
            ty: self.ty,

            state: pin_state::Gpio {
                dirset: self.state.dirset,
                pin: self.state.pin,
                set: self.state.set,
                clr: self.state.clr,

                _direction: direction::Output,
            },
        }
    }
}

impl<'gpio, T> OutputPin for Pin<T, pin_state::Gpio<'gpio, direction::Output>>
where
    T: PinTrait,
{
    type Error = Void;

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
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.state.set[T::PORT].write(|w| unsafe { w.setp().bits(T::MASK) }))
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
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.state.clr[T::PORT].write(|w| unsafe { w.clrp().bits(T::MASK) }))
    }
}

impl<'gpio, T> StatefulOutputPin for Pin<T, pin_state::Gpio<'gpio, direction::Output>>
where
    T: PinTrait,
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
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.state.pin[T::PORT].read().port().bits() & T::MASK == T::MASK)
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
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.state.pin[T::PORT].read().port().bits() & T::MASK == T::MASK)
    }
}

/// Contains types to indicate the direction of GPIO pins
///
/// Please refer to [`Pin`] for documentation on how these types are used.
pub mod direction {
    /// Implemented by types that indicate GPIO pin direction
    ///
    /// The [`Gpio`] type uses this trait as a bound for its type parameter.
    /// This is done for documentation purposes, to clearly show which types can
    /// be used for this parameter. Other than that, this trait should not be
    /// relevant to users of this crate.
    ///
    /// [`Gpio`]: ../../swm/pin_state/struct.Gpio.html
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
    /// [`Gpio`]: ../../swm/pin_state/struct.Gpio.html
    /// [`Pin`]: ../../swm/struct.Pin.html
    pub struct Unknown;
    impl Direction for Unknown {}

    /// Marks a GPIO pin as being configured for input
    ///
    /// This type is used as a type parameter of [`Gpio`], which in turn is used
    /// as a type parameter of [`Pin`]. Please refer to the documentation of
    /// [`Pin`] to see how this type is used.
    ///
    /// [`Gpio`]: ../../swm/pin_state/struct.Gpio.html
    /// [`Pin`]: ../../swm/struct.Pin.html
    pub struct Input;
    impl Direction for Input {}

    /// Marks a GPIO pin as being configured for output
    ///
    /// This type is used as a type parameter of [`Gpio`], which in turn is used
    /// as a type parameter of [`Pin`]. Please refer to the documentation of
    /// [`Pin`] to see how this type is used.
    ///
    /// [`Gpio`]: ../../swm/pin_state/struct.Gpio.html
    /// [`Pin`]: ../../swm/struct.Pin.html
    pub struct Output;
    impl Direction for Output {}

    /// Marks a direction as not being output (i.e. being unknown or input)
    ///
    /// This is a helper trait used only to prevent some code duplication in
    /// [`Pin`] by allowing `impl` blocks to be defined precisely. It should not
    /// be relevant to users of this crate.
    ///
    /// [`Pin`]: ../../swm/struct.Pin.html
    pub trait NotOutput: Direction {}

    impl NotOutput for Unknown {}
    impl NotOutput for Input {}
}
