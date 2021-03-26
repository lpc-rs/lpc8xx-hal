//! API for General Purpose I/O (GPIO)
//!
//! The entry point to this API is [`GPIO`]. It can be used to initialize the
//! peripheral, and is required to convert instances of [`Pin`] to a
//! [`GpioPin`], which provides the core GPIO API.
//!
//! The GPIO peripheral is described in the following user manuals:
//! - LPC82x user manual, chapter 9
//! - LPC84x user manual, chapter 12
//!
//! # Examples
//!
//! Initialize a GPIO pin and set its output to HIGH:
//!
//! ``` no_run
//! use lpc8xx_hal::{
//!     prelude::*,
//!     Peripherals,
//!     gpio,
//! };
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut syscon = p.SYSCON.split();
//!
//! #[cfg(feature = "82x")]
//! let gpio = p.GPIO;
//! #[cfg(feature = "845")]
//! let gpio = p.GPIO.enable(&mut syscon.handle);
//!
//! let pio0_12 = p.pins.pio0_12.into_output_pin(
//!     gpio.tokens.pio0_12,
//!     gpio::Level::High,
//! );
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [`GPIO`]: struct.GPIO.html
//! [`Pin`]: ../pins/struct.Pin.html
//! [`GpioPin`]: struct.GpioPin.html
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/examples

use core::marker::PhantomData;

use embedded_hal::digital::v2::{
    InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin,
};
use embedded_hal_alpha::digital::{
    InputPin as InputPinAlpha, OutputPin as OutputPinAlpha,
    StatefulOutputPin as StatefulOutputPinAlpha,
    ToggleableOutputPin as ToggleableOutputPinAlpha,
};
use void::Void;

use crate::{init_state, pac, pins, syscon};

#[cfg(feature = "845")]
use crate::pac::gpio::{CLR, DIRCLR, DIRSET, NOT, PIN, SET};
#[cfg(feature = "82x")]
use crate::pac::gpio::{
    CLR0 as CLR, DIRCLR0 as DIRCLR, DIRSET0 as DIRSET, NOT0 as NOT,
    PIN0 as PIN, SET0 as SET,
};

use self::direction::{Direction, DynamicPinErr};

/// Interface to the GPIO peripheral
///
/// Controls the GPIO peripheral. Can be used to enable, disable, or free the
/// peripheral. For GPIO-functionality directly related to pins, please refer
/// to [`GpioPin`].
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`GpioPin`]: struct.GpioPin.html
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct GPIO<State = init_state::Enabled> {
    pub(crate) gpio: pac::GPIO,
    _state: PhantomData<State>,

    /// Tokens representing all pins
    ///
    /// Since the [`enable`] and [`disable`] methods consume `self`, they can
    /// only be called, if all tokens are available. This means, any tokens that
    /// have been moved out while the peripheral was enabled, prevent the
    /// peripheral from being disabled (unless those tokens are moved back into
    /// their original place).
    ///
    /// As using a pin for GPIO requires such a token, it is impossible to
    /// disable the GPIO peripheral while pins are used for GPIO.
    ///
    /// [`enable`]: #method.enable
    /// [`disable`]: #method.disable
    pub tokens: pins::Tokens<State>,
}

impl<State> GPIO<State> {
    pub(crate) fn new(gpio: pac::GPIO) -> Self {
        GPIO {
            gpio,
            _state: PhantomData,

            tokens: pins::Tokens::new(),
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
    pub fn free(self) -> pac::GPIO {
        self.gpio
    }
}

impl GPIO<init_state::Disabled> {
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
    pub fn enable(
        self,
        syscon: &mut syscon::Handle,
    ) -> GPIO<init_state::Enabled> {
        syscon.enable_clock(&self.gpio);

        // Only works, if all tokens are available.
        let tokens = self.tokens.switch_state();

        GPIO {
            gpio: self.gpio,
            _state: PhantomData,
            tokens,
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
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> GPIO<init_state::Disabled> {
        syscon.disable_clock(&self.gpio);

        // Only works, if all tokens are available.
        let tokens = self.tokens.switch_state();

        GPIO {
            gpio: self.gpio,
            _state: PhantomData,
            tokens,
        }
    }
}

/// A pin used for general purpose I/O (GPIO).
///
/// This struct is a wrapper around the representation of a specific pin `P`– it enables said pin
/// to be used as a GPIO pin.
///
/// You can get access to an instance of this struct by switching a pin to the
/// GPIO state, using [`Pin::into_input_pin`] or [`Pin::into_output_pin`].
///
/// # `embedded-hal` traits
/// - While in input mode
///   - [`embedded_hal::digital::v2::InputPin`] for reading the pin state
/// - While in output mode
///   - [`embedded_hal::digital::v2::OutputPin`] for setting the pin state
///   - [`embedded_hal::digital::v2::StatefulOutputPin`] for reading the pin output state
///   - [`embedded_hal::digital::v2::ToggleableOutputPin`] for toggling the pin state
///
/// [`Pin::into_input_pin`]: ../pins/struct.Pin.html#method.into_input_pin
/// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
/// [`embedded_hal::digital::v2::InputPin`]: #impl-InputPin
/// [`embedded_hal::digital::v2::OutputPin`]: #impl-OutputPin
/// [`embedded_hal::digital::v2::StatefulOutputPin`]: #impl-StatefulOutputPin
/// [`embedded_hal::digital::v2::ToggleableOutputPin`]: #impl-ToggleableOutputPin
pub struct GpioPin<P, D> {
    inner: P, // holds port, id and mask for this specific pin
    _direction: D,
}

impl<P, D> GpioPin<P, D>
where
    P: pins::Trait,
    D: Direction,
{
    pub(crate) fn new(inner: P, arg: D::SwitchArg) -> Self {
        // This is sound, as we only write to stateless registers, restricting
        // ourselves to the bit that belongs to the pin represented by `P`.
        // Since all other instances of `GpioPin` are doing the same, there are
        // no race conditions.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);
        let direction = D::switch(&registers, arg, &inner);

        Self {
            inner,
            _direction: direction,
        }
    }

    /// Get identifying information about this pin in the form of a [`pins::Trait`]
    ///
    /// [`pins::Trait`]: ../pins/trait.Trait.html
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Indicates wether the voltage at the pin is currently HIGH
    /// This is not accessible to the user to avoid confusion becauzse `is_high()`
    /// semantics differ depending on pin direction. It is only used to implement
    /// `is_high()` and `is_set_high()` respectively for the different direction types.
    pub(crate) fn is_high_inner(&self) -> bool {
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        is_high(&registers, self.inner())
    }
}

impl<P> GpioPin<P, direction::Input>
where
    P: pins::Trait,
{
    /// Set pin direction to output
    ///
    /// This method is only available while the pin is in input mode.
    ///
    /// Consumes the pin instance and returns a new instance that is in output
    /// mode, making the methods to set the output level available.
    pub fn into_output(self, initial: Level) -> GpioPin<P, direction::Output> {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        let direction =
            direction::Output::switch(&registers, initial, self.inner());

        GpioPin {
            inner: self.inner,
            _direction: direction,
        }
    }

    /// Set pin direction to dynamic (i.e. changeable at runtime)
    ///
    /// This method is only available when the pin is not already in dynamic mode.
    ///
    /// Consumes the pin instance and returns a new instance that is in dynamic
    /// mode, making the methods to change direction as well as read/set levels
    /// (depending on the current diection) available.
    pub fn into_dynamic(
        self,
        initial_level: Level,
        initial_direction: pins::DynamicPinDirection,
    ) -> GpioPin<P, direction::Dynamic> {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        // always switch to ensure initial level and direction are set correctly
        let new_direction = direction::Dynamic::switch(
            &registers,
            (initial_level, initial_direction),
            self.inner(),
        );

        GpioPin {
            inner: self.inner,
            _direction: new_direction,
        }
    }

    /// Indicates wether the voltage at the pin is currently HIGH
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to input.
    ///
    /// See [`Pin::into_input_pin`] and [`into_input`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_input_pin`]: ../pins/struct.Pin.html#method.into_input_pin
    /// [`into_input`]: #method.into_input
    pub fn is_high(&self) -> bool {
        self.is_high_inner()
    }

    /// Indicates wether the pin input is LOW
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to input.
    ///
    /// See [`Pin::into_input_pin`] and [`into_input`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_input_pin`]: ../pins/struct.Pin.html#method.into_input_pin
    /// [`into_input`]: #method.into_input
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Returns the current voltage level at this pin.
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to input.
    ///
    /// See [`Pin::into_input_pin`] and [`into_input`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_input_pin`]: ../pins/struct.Pin.html#method.into_input_pin
    /// [`into_input`]: #method.into_input
    pub fn get_level(&self) -> Level {
        Level::from_pin(&self)
    }
}

impl<P> GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    /// Set pin direction to input
    ///
    /// This method is only available while the pin is in output mode.
    ///
    /// Consumes the pin instance and returns a new instance that is in output
    /// mode, making the methods to set the output level available.
    pub fn into_input(self) -> GpioPin<P, direction::Input> {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        let direction = direction::Input::switch(&registers, (), &self.inner);

        GpioPin {
            inner: self.inner,
            _direction: direction,
        }
    }

    /// Set pin direction to dynamic (i.e. changeable at runtime)
    ///
    /// This method is only available when the pin is not already in dynamic mode.
    ///
    /// Consumes the pin instance and returns a new instance that is in dynamic
    /// mode, making the methods to change direction as well as read/set levels
    /// (depending on the current diection) available.
    pub fn into_dynamic(
        self,
        initial_level: Level,
        initial_direction: pins::DynamicPinDirection,
    ) -> GpioPin<P, direction::Dynamic> {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        // always switch to ensure initial level and direction are set correctly
        let new_direction = direction::Dynamic::switch(
            &registers,
            (initial_level, initial_direction),
            &self.inner,
        );

        GpioPin {
            inner: self.inner,
            _direction: new_direction,
        }
    }

    /// Set the pin output to HIGH
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to output.
    ///
    /// See [`Pin::into_output_pin`] and [`into_output`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
    /// [`into_output`]: #method.into_output
    pub fn set_high(&mut self) {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        set_high(&registers, self.inner());
    }

    /// Set the pin output to LOW
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to output.
    ///
    /// See [`Pin::into_output_pin`] and [`into_output`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
    /// [`into_output`]: #method.into_output
    pub fn set_low(&mut self) {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        set_low(&registers, self.inner());
    }

    /// Indicates whether the pin output is currently set to HIGH
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to output.
    ///
    /// See [`Pin::into_output_pin`] and [`into_output`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
    /// [`into_output`]: #method.into_output
    pub fn is_set_high(&self) -> bool {
        // This is sound, as we only read a bit from a register.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        is_high(&registers, self.inner())
    }

    /// Indicates whether the pin output is currently set to LOW
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to output.
    ///
    /// See [`Pin::into_output_pin`] and [`into_output`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
    /// [`into_output`]: #method.into_output
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Returns the level to which this pin is currently set
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to output.
    ///
    /// See [`Pin::into_output_pin`] and [`into_output`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
    /// [`into_output`]: #method.into_output
    pub fn get_set_level(&self) -> Level {
        match self.is_set_high() {
            true => Level::High,
            false => Level::Low,
        }
    }

    /// Toggle the pin output
    ///
    /// This method is only available, if two conditions are met:
    /// - The pin is in the GPIO state.
    /// - The pin direction is set to output.
    ///
    /// See [`Pin::into_output_pin`] and [`into_output`]. Unless both of these
    /// conditions are met, code trying to call this method will not compile.
    ///
    /// [`Pin::into_output_pin`]: ../pins/struct.Pin.html#method.into_output_pin
    /// [`into_output`]: #method.into_output
    pub fn toggle(&mut self) {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        registers.not[usize::from(self.inner().port())]
            .write(|w| unsafe { w.notp().bits(self.inner().mask()) });
    }
}

impl<P> GpioPin<P, direction::Dynamic>
where
    P: pins::Trait,
{
    /// Tell us whether this pin's direction is currently set to Output.
    pub fn direction_is_output(&self) -> bool {
        return self._direction.current_direction
            == pins::DynamicPinDirection::Output;
    }

    /// Tell us whether this pin's direction is currently set to Input.
    pub fn direction_is_input(&self) -> bool {
        return !self.direction_is_output();
    }

    /// Switch pin direction to input. If the pin is already an input pin, this does nothing.
    pub fn switch_to_input(&mut self) {
        if self._direction.current_direction == pins::DynamicPinDirection::Input
        {
            return;
        }

        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        // switch direction
        set_direction_input(&registers, self.inner());
        self._direction.current_direction = pins::DynamicPinDirection::Input;
    }

    /// Switch pin direction to output with output level set to `level`.
    /// If the pin is already an output pin, this function only switches its level to `level`.
    pub fn switch_to_output(&mut self, level: Level) {
        // First set the output level, before we switch the mode.
        match level {
            Level::High => self.set_high(),
            Level::Low => self.set_low(),
        }

        // we are already in output, nothing else to do
        if self._direction.current_direction
            == pins::DynamicPinDirection::Output
        {
            return;
        }

        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        // Now that the output level is configured, we can safely switch to
        // output mode, without risking an undesired signal between now and
        // the first call to `set_high`/`set_low`.
        set_direction_output(&registers, self.inner());
        self._direction.current_direction = pins::DynamicPinDirection::Output;
    }

    /// Set the pin level to High.
    /// Note that this will be executed regardless of the current pin direction.
    /// This enables you to set the initial pin level *before* switching to output
    pub fn set_high(&mut self) {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        set_high(&registers, self.inner());
    }

    /// Set the pin level to Low.
    /// Note that this will be executed regardless of the current pin direction.
    /// This enables you to set the initial pin level *before* switching to output
    pub fn set_low(&mut self) {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        set_low(&registers, self.inner());
    }

    /// Returns the current voltage level at this pin.
    /// This can be used when the pin is in any direction:
    ///
    /// If it is currently an Output pin, it indicates to which level the pin is set
    /// If it is currently an Input pin, it indicates the level currently present at this pin
    ///
    /// This method is only available, if the pin has been set to dynamic mode.
    /// See [`Pin::into_dynamic_pin`].
    /// Unless this condition is met, code trying to call this method will not compile.
    pub fn get_level(&self) -> Level {
        Level::from_pin(&self)
    }
}

impl<P> OutputPin for GpioPin<P, direction::Dynamic>
where
    P: pins::Trait,
{
    type Error = DynamicPinErr;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        // NOTE: this check is kind of redundant but since both `set_high()`s are public I
        // didn't want to either leave it out of `self.set_high()` or return an OK here
        // when there's really an error
        // (applies to all Dynamic Pin impls)
        match self._direction.current_direction {
            pins::DynamicPinDirection::Output => {
                // Call the inherent method defined above.
                Ok(self.set_high())
            }
            pins::DynamicPinDirection::Input => {
                Err(Self::Error::WrongDirection)
            }
        }
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        match self._direction.current_direction {
            pins::DynamicPinDirection::Output => {
                // Call the inherent method defined above.
                Ok(self.set_low())
            }
            pins::DynamicPinDirection::Input => {
                Err(Self::Error::WrongDirection)
            }
        }
    }
}

impl<P> StatefulOutputPin for GpioPin<P, direction::Dynamic>
where
    P: pins::Trait,
{
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        match self._direction.current_direction {
            pins::DynamicPinDirection::Output => {
                // Re-use level reading function
                self.is_set_high()
            }
            pins::DynamicPinDirection::Input => {
                Err(Self::Error::WrongDirection)
            }
        }
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        match self._direction.current_direction {
            pins::DynamicPinDirection::Output => {
                // Re-use level reading function
                self.is_set_low()
            }
            pins::DynamicPinDirection::Input => {
                Err(Self::Error::WrongDirection)
            }
        }
    }
}

impl<P> InputPin for GpioPin<P, direction::Dynamic>
where
    P: pins::Trait,
{
    type Error = DynamicPinErr;

    fn is_high(&self) -> Result<bool, Self::Error> {
        match self._direction.current_direction {
            pins::DynamicPinDirection::Output => {
                Err(Self::Error::WrongDirection)
            }
            pins::DynamicPinDirection::Input => {
                // Call the inherent method defined above.
                Ok(self.is_high_inner())
            }
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        match self._direction.current_direction {
            pins::DynamicPinDirection::Output => {
                Err(Self::Error::WrongDirection)
            }
            pins::DynamicPinDirection::Input => {
                // Call the inherent method defined above.
                Ok(!self.is_high_inner())
            }
        }
    }
}

impl<P> InputPin for GpioPin<P, direction::Input>
where
    P: pins::Trait,
{
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_high())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_low())
    }
}

impl<P> InputPinAlpha for GpioPin<P, direction::Input>
where
    P: pins::Trait,
{
    type Error = Void;

    fn try_is_high(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_high())
    }

    fn try_is_low(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_low())
    }
}

impl<P> OutputPin for GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    type Error = Void;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.set_high())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.set_low())
    }
}

impl<P> OutputPinAlpha for GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    type Error = Void;

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.set_high())
    }

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.set_low())
    }
}

impl<P> StatefulOutputPin for GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_set_high())
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_set_low())
    }
}

impl<P> StatefulOutputPinAlpha for GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    fn try_is_set_high(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_set_high())
    }

    fn try_is_set_low(&self) -> Result<bool, Self::Error> {
        // Call the inherent method defined above.
        Ok(self.is_set_low())
    }
}

impl<P> ToggleableOutputPin for GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    type Error = Void;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.toggle())
    }
}

impl<P> ToggleableOutputPinAlpha for GpioPin<P, direction::Output>
where
    P: pins::Trait,
{
    type Error = Void;

    fn try_toggle(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.toggle())
    }
}

/// The voltage level of a pin
#[derive(Debug, Copy, Clone)]
pub enum Level {
    /// High voltage
    High,

    /// Low voltage
    Low,
}

impl Level {
    fn from_pin<P: pins::Trait, D: Direction>(pin: &GpioPin<P, D>) -> Self {
        match pin.is_high_inner() {
            true => Level::High,
            false => Level::Low,
        }
    }
}

fn set_high(registers: &Registers, inner: &impl pins::Trait) {
    registers.set[usize::from(inner.port())]
        .write(|w| unsafe { w.setp().bits(inner.mask()) });
}

fn set_low(registers: &Registers, inner: &impl pins::Trait) {
    registers.clr[usize::from(inner.port())]
        .write(|w| unsafe { w.clrp().bits(inner.mask()) });
}

fn is_high(registers: &Registers, inner: &impl pins::Trait) -> bool {
    registers.pin[usize::from(inner.port())]
        .read()
        .port()
        .bits()
        & inner.mask()
        == inner.mask()
}

// For internal use only.
// Use the direction helpers of `GpioPin<P, direction::Output>` and `GpioPin<P, direction::Dynamic>`
// instead.
fn set_direction_output(registers: &Registers, inner: &impl pins::Trait) {
    registers.dirset[usize::from(inner.port())]
        .write(|w| unsafe { w.dirsetp().bits(inner.mask()) });
}

// For internal use only.
// Use the direction helpers of `GpioPin<P, direction::Input>` and `GpioPin<P, direction::Dynamic>`
// instead.
fn set_direction_input(registers: &Registers, inner: &impl pins::Trait) {
    registers.dirclr[usize::from(inner.port())]
        .write(|w| unsafe { w.dirclrp().bits(inner.mask()) });
}

/// This is an internal type that should be of no concern to users of this crate
pub struct Registers<'gpio> {
    dirset: &'gpio [DIRSET],
    dirclr: &'gpio [DIRCLR],
    pin: &'gpio [PIN],
    set: &'gpio [SET],
    clr: &'gpio [CLR],
    not: &'gpio [NOT],
}

impl<'gpio> Registers<'gpio> {
    /// Create a new instance of `Registers` from the provided register block
    ///
    /// If the reference to `RegisterBlock` is not exclusively owned by the
    /// caller, accessing all registers is still completely race-free, as long
    /// as the following rules are upheld:
    /// - Never write to `pin`, only use it for reading.
    /// - For all other registers, only set bits that no other callers are
    ///   setting.
    fn new(gpio: &'gpio pac::gpio::RegisterBlock) -> Self {
        #[cfg(feature = "82x")]
        {
            use core::slice;

            Self {
                dirset: slice::from_ref(&gpio.dirset0),
                dirclr: slice::from_ref(&gpio.dirclr0),
                pin: slice::from_ref(&gpio.pin0),
                set: slice::from_ref(&gpio.set0),
                clr: slice::from_ref(&gpio.clr0),
                not: slice::from_ref(&gpio.not0),
            }
        }

        #[cfg(feature = "845")]
        Self {
            dirset: &gpio.dirset,
            dirclr: &gpio.dirclr,
            pin: &gpio.pin,
            set: &gpio.set,
            clr: &gpio.clr,
            not: &gpio.not,
        }
    }
}

/// Contains types to indicate the direction of GPIO pins
///
/// Please refer to [`GpioPin`] for documentation on how these types are used.
///
/// [`GpioPin`]: ../struct.GpioPin.html
pub mod direction {
    use crate::pins;

    use super::{Level, Registers};

    /// Implemented by types that indicate GPIO pin direction
    ///
    /// The [`GpioPin`] type uses this trait as a bound for its type parameter.
    /// This is done for documentation purposes, to clearly show which types can
    /// be used for this parameter. Other than that, this trait should not be
    /// relevant to users of this crate.
    ///
    /// [`GpioPin`]: ../struct.GpioPin.html
    pub trait Direction {
        /// The argument of the `switch` method
        type SwitchArg;

        /// Switch a pin to this direction
        ///
        /// This method is for internal use only. Any changes to it won't be
        /// considered breaking changes.
        fn switch<P: pins::Trait>(
            _: &Registers,
            _: Self::SwitchArg,
            _: &P,
        ) -> Self;
    }

    /// Marks a GPIO pin as being configured for input
    ///
    /// This type is used as a type parameter of [`GpioPin`]. Please refer to
    /// the documentation there to see how this type is used.
    ///
    /// [`GpioPin`]: ../struct.GpioPin.html
    pub struct Input(());

    impl Direction for Input {
        type SwitchArg = ();

        fn switch<P: pins::Trait>(
            registers: &Registers,
            _: Self::SwitchArg,
            inner: &P,
        ) -> Self {
            super::set_direction_input(registers, inner);
            Self(())
        }
    }

    /// Marks a GPIO pin as being configured for output
    ///
    /// This type is used as a type parameter of [`GpioPin`]. Please refer to
    /// the documentation there to see how this type is used.
    ///
    /// [`GpioPin`]: ../struct.GpioPin.html
    pub struct Output(());

    impl Direction for Output {
        type SwitchArg = Level;

        fn switch<P: pins::Trait>(
            registers: &Registers,
            initial: Level,
            inner: &P,
        ) -> Self {
            // First set the output level, before we switch the mode.
            match initial {
                Level::High => super::set_high(registers, inner),
                Level::Low => super::set_low(registers, inner),
            }

            // Now that the output level is configured, we can safely switch to
            // output mode, without risking an undesired signal between now and
            // the first call to `set_high`/`set_low`.
            super::set_direction_output(&registers, inner);

            Self(())
        }
    }

    /// Marks a GPIO pin as being run-time configurable for in/output
    /// Initial direction is Output
    ///
    /// This type is used as a type parameter of [`GpioPin`]. Please refer to
    /// the documentation there to see how this type is used.
    ///
    /// [`GpioPin`]: ../struct.GpioPin.html
    pub struct Dynamic {
        pub(super) current_direction: pins::DynamicPinDirection,
    }

    /// Error that can be thrown by operations on a Dynamic pin
    #[derive(Copy, Clone, Debug)]
    pub enum DynamicPinErr {
        /// you called a function that is not applicable to the pin's current direction
        WrongDirection,
    }

    impl Direction for Dynamic {
        type SwitchArg = (Level, pins::DynamicPinDirection);

        fn switch<P: pins::Trait>(
            registers: &Registers,
            initial: Self::SwitchArg,
            inner: &P,
        ) -> Self {
            let (level, current_direction) = initial;

            // First set the output level, before we switch the mode.
            match level {
                Level::High => super::set_high(registers, inner),
                Level::Low => super::set_low(registers, inner),
            }

            match current_direction {
                pins::DynamicPinDirection::Input => {
                    // Now that the output level is configured, we can safely switch to
                    // output mode, without risking an undesired signal between now and
                    // the first call to `set_high`/`set_low`.
                    super::set_direction_input(registers, inner);
                }
                pins::DynamicPinDirection::Output => {
                    // Now that the output level is configured, we can safely switch to
                    // output mode, without risking an undesired signal between now and
                    // the first call to `set_high`/`set_low`.
                    super::set_direction_output(registers, inner);
                }
            }

            Self { current_direction }
        }
    }
}
