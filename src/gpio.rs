//! APIs for General Purpose I/O (GPIO)
//!
//! See user manual, chapter 9.


use lpc82x;

use ::{
    swm,
    Pin,
    Swm,
    Syscon,
};
use init_state::{
    self,
    InitState,
};


/// Interface to general-purpose I/O (GPIO)
///
/// This API expects to be the sole owner of the GPIO peripheral. Don't use
/// [`lpc82x::GPIO_PORT`] directly, unless you know what you're doing.
///
/// [`lpc82x::GPIO_PORT`]: ../../lpc82x/struct.GPIO_PORT.html
pub struct Gpio<'gpio, State: InitState = init_state::Initialized> {
    gpio  : &'gpio lpc82x::GPIO_PORT,
    _state: State,
}

impl<'gpio> Gpio<'gpio, init_state::Unknown> {
    pub(crate) fn new(gpio: &'gpio lpc82x::GPIO_PORT) -> Self {
        Gpio {
            gpio  : gpio,
            _state: init_state::Unknown,
        }
    }

    /// Initialize GPIO
    pub fn init(mut self, syscon: &mut Syscon)
        -> Gpio<'gpio, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.gpio);
        syscon.clear_reset(&mut self.gpio);

        Gpio {
            gpio  : self.gpio,
            _state: init_state::Initialized,
        }
    }
}

impl<'gpio> Gpio<'gpio> {
    /// Sets pin direction to output
    ///
    /// Disables the fixed function of the given pin (thus making it available
    /// for GPIO) and sets the GPIO direction to output.
    pub fn set_pin_to_output<P>(&mut self, swm: &mut Swm)
        where P: Pin + swm::PinExt
    {
        P::disable_fixed_functions(swm);

        self.gpio.dirset0.write(|w|
            unsafe { w.dirsetp().bits(P::mask()) }
        )
    }

    /// Set pin output to HIGH
    pub fn set_high<P>(&mut self) where P: Pin {
        self.gpio.set0.write(|w|
            unsafe { w.setp().bits(P::mask()) }
        )
    }

    /// Set pin output to LOW
    pub fn set_low<P>(&mut self) where P: Pin {
        self.gpio.clr0.write(|w|
            unsafe { w.clrp().bits(P::mask()) }
        );
    }
}
