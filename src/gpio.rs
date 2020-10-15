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

use crate::{
    init_state, pac,
    pins::{self, Token},
    syscon,
};

#[cfg(feature = "845")]
use crate::pac::gpio::{CLR, DIRCLR, DIRSET, NOT, PIN, SET};
#[cfg(feature = "82x")]
use crate::pac::gpio::{
    CLR0 as CLR, DIRCLR0 as DIRCLR, DIRSET0 as DIRSET, NOT0 as NOT,
    PIN0 as PIN, SET0 as SET,
};

use self::direction::Direction;

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

/// A pin used for general purpose I/O (GPIO)
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
pub struct GpioPin<T, D> {
    token: pins::Token<T, init_state::Enabled>,
    _direction: D,
}

impl<T, D> GpioPin<T, D>
where
    T: pins::Trait,
    D: Direction,
{
    pub(crate) fn new(
        token: Token<T, init_state::Enabled>,
        arg: D::SwitchArg,
    ) -> Self {
        // This is sound, as we only write to stateless registers, restricting
        // ourselves to the bit that belongs to the pin represented by `T`.
        // Since all other instances of `GpioPin` are doing the same, there are
        // no race conditions.
        let gpio = unsafe { &*pac::GPIO::ptr() };

        let registers = Registers::new(gpio);
        let direction = D::switch::<T>(&registers, arg);

        Self {
            token,
            _direction: direction,
        }
    }
}

impl<T> GpioPin<T, direction::Input>
where
    T: pins::Trait,
{
    /// Set pin direction to output
    ///
    /// This method is only available while the pin is in input mode.
    ///
    /// Consumes the pin instance and returns a new instance that is in output
    /// mode, making the methods to set the output level available.
    pub fn into_output(self, initial: Level) -> GpioPin<T, direction::Output> {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        let direction = direction::Output::switch::<T>(&registers, initial);

        GpioPin {
            token: self.token,
            _direction: direction,
        }
    }

    /// Indicates wether the pin input is HIGH
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
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        registers.pin[T::PORT].read().port().bits() & T::MASK == T::MASK
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
}

impl<T> GpioPin<T, direction::Output>
where
    T: pins::Trait,
{
    /// Set pin direction to input
    ///
    /// This method is only available while the pin is in output mode.
    ///
    /// Consumes the pin instance and returns a new instance that is in output
    /// mode, making the methods to set the output level available.
    pub fn into_input(self) -> GpioPin<T, direction::Input> {
        // This is sound, as we only do a stateless write to a bit that no other
        // `GpioPin` instance writes to.
        let gpio = unsafe { &*pac::GPIO::ptr() };
        let registers = Registers::new(gpio);

        let direction = direction::Input::switch::<T>(&registers, ());

        GpioPin {
            token: self.token,
            _direction: direction,
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

        set_high::<T>(&registers);
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

        set_low::<T>(&registers);
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

        registers.pin[T::PORT].read().port().bits() & T::MASK == T::MASK
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

        registers.not[T::PORT].write(|w| unsafe { w.notp().bits(T::MASK) });
    }
}

impl<T> InputPin for GpioPin<T, direction::Input>
where
    T: pins::Trait,
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

impl<T> InputPinAlpha for GpioPin<T, direction::Input>
where
    T: pins::Trait,
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

impl<T> OutputPin for GpioPin<T, direction::Output>
where
    T: pins::Trait,
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

impl<T> OutputPinAlpha for GpioPin<T, direction::Output>
where
    T: pins::Trait,
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

impl<T> StatefulOutputPin for GpioPin<T, direction::Output>
where
    T: pins::Trait,
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

impl<T> StatefulOutputPinAlpha for GpioPin<T, direction::Output>
where
    T: pins::Trait,
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

impl<T> ToggleableOutputPin for GpioPin<T, direction::Output>
where
    T: pins::Trait,
{
    type Error = Void;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.toggle())
    }
}

impl<T> ToggleableOutputPinAlpha for GpioPin<T, direction::Output>
where
    T: pins::Trait,
{
    type Error = Void;

    fn try_toggle(&mut self) -> Result<(), Self::Error> {
        // Call the inherent method defined above.
        Ok(self.toggle())
    }
}

/// The voltage level of a pin
#[derive(Debug)]
pub enum Level {
    /// High voltage
    High,

    /// Low voltage
    Low,
}

fn set_high<T: pins::Trait>(registers: &Registers) {
    registers.set[T::PORT].write(|w| unsafe { w.setp().bits(T::MASK) });
}

fn set_low<T: pins::Trait>(registers: &Registers) {
    registers.clr[T::PORT].write(|w| unsafe { w.clrp().bits(T::MASK) });
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
        fn switch<T: pins::Trait>(_: &Registers, _: Self::SwitchArg) -> Self;
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

        fn switch<T: pins::Trait>(
            registers: &Registers,
            _: Self::SwitchArg,
        ) -> Self {
            registers.dirclr[T::PORT]
                .write(|w| unsafe { w.dirclrp().bits(T::MASK) });
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

        fn switch<T: pins::Trait>(
            registers: &Registers,
            initial: Level,
        ) -> Self {
            // First set the output level, before we switch the mode.
            match initial {
                Level::High => super::set_high::<T>(registers),
                Level::Low => super::set_low::<T>(registers),
            }

            // Now that the output level is configured, we can safely switch to
            // output mode, without risking an undesired signal between now and
            // the first call to `set_high`/`set_low`.
            registers.dirset[T::PORT]
                .write(|w| unsafe { w.dirsetp().bits(T::MASK) });

            Self(())
        }
    }
}
