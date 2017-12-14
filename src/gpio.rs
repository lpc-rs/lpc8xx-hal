//! APIs for General Purpose I/O (GPIO)
//!
//! See user manual, chapter 9.


use lpc82x;

use ::{
    syscon,
    SWM,
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
pub struct GPIO<'gpio, State: InitState = init_state::Initialized> {
    gpio  : &'gpio lpc82x::GPIO_PORT,
    _state: State,
}

impl<'gpio> GPIO<'gpio, init_state::Unknown> {
    pub(crate) fn new(gpio: &'gpio lpc82x::GPIO_PORT) -> Self {
        GPIO {
            gpio  : gpio,
            _state: init_state::Unknown,
        }
    }

    /// Initialize GPIO
    pub fn init(mut self, syscon: &mut syscon::Api)
        -> GPIO<'gpio, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.gpio);
        syscon.clear_reset(&mut self.gpio);

        GPIO {
            gpio  : self.gpio,
            _state: init_state::Initialized,
        }
    }
}

impl<'gpio> GPIO<'gpio> {
    /// Provides access to all pins
    pub fn pins(&self) -> Pins {
        Pins::new(self)
    }
}


/// Represents a pin
///
/// This trait is implemented by all types that represent a pin. HAL users
/// shouldn't need to implement this trait themselves.
///
/// It also should not be necessary for HAL users to use the methods of this
/// trait directly, unless compensating for missing pieces of HAL functionality.
/// Ideally, there should be higher-level peripheral methods that take pins as
/// parameters and use the methods of this trait to take care of the low-level
/// details.
pub trait Pin {
    /// Returns a number that identifies the pin
    ///
    /// This is `0` for [`PIO0_0`], `1` for [`PIO0_1`] and so forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    fn id() -> u8;

    /// Returns the pin's mask
    ///
    /// This is `0x00000001` for [`PIO0_0`], `0x00000002` for [`PIO0_1`] and so
    /// forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    fn mask() -> u32;
}


macro_rules! pins {
    ($($field:ident, $type:ident, $id:expr;)*) => {
        /// Provides access to all pins
        #[allow(missing_docs)]
        pub struct Pins<'gpio> {
            $(pub $field: $type<'gpio>,)*
        }

        impl<'gpio> Pins<'gpio> {
            pub(crate) fn new(gpio: &'gpio GPIO<'gpio>) -> Self {
                Pins {
                    $($field: $type(gpio),)*
                }
            }
        }


        $(
            /// Represents the pin this struct is named after
            ///
            /// # Limitations
            ///
            /// Currently, nothing prevents users of this HAL from creating any
            /// number of instances of this struct and using them for all kinds
            /// of purposes. Until this shortcoming is rectified, it is your own
            /// responsibility to make sure you are using the pin correctly.
            #[allow(non_camel_case_types)]
            pub struct $type<'gpio>(&'gpio GPIO<'gpio>);

            impl<'gpio> Pin for $type<'gpio> {
                fn id() -> u8 {
                    $id
                }

                fn mask() -> u32 {
                    0x1 << $id
                }
            }

            impl<'gpio> $type<'gpio> {
                /// Sets pin direction to output
                ///
                /// Disables the fixed function of the given pin (thus making it
                /// available for GPIO) and sets the GPIO direction to output.
                pub fn set_pin_to_output(&mut self, swm: &mut SWM) {
                    use swm::PinExt;
                    Self::disable_fixed_functions(swm);

                    self.0.gpio.dirset0.write(|w|
                        unsafe { w.dirsetp().bits(Self::mask()) }
                    )
                }

                /// Set pin output to HIGH
                pub fn set_high(&mut self) {
                    self.0.gpio.set0.write(|w|
                        unsafe { w.setp().bits(Self::mask()) }
                    )
                }

                /// Set pin output to LOW
                pub fn set_low(&mut self) {
                    self.0.gpio.clr0.write(|w|
                        unsafe { w.clrp().bits(Self::mask()) }
                    );
                }
            }
        )*
    }
}

pins!(
    pio0_0 , PIO0_0 , 0x00;
    pio0_1 , PIO0_1 , 0x01;
    pio0_2 , PIO0_2 , 0x02;
    pio0_3 , PIO0_3 , 0x03;
    pio0_4 , PIO0_4 , 0x04;
    pio0_5 , PIO0_5 , 0x05;
    pio0_6 , PIO0_6 , 0x06;
    pio0_7 , PIO0_7 , 0x07;
    pio0_8 , PIO0_8 , 0x08;
    pio0_9 , PIO0_9 , 0x09;
    pio0_10, PIO0_10, 0x0a;
    pio0_11, PIO0_11, 0x0b;
    pio0_12, PIO0_12, 0x0c;
    pio0_13, PIO0_13, 0x0d;
    pio0_14, PIO0_14, 0x0e;
    pio0_15, PIO0_15, 0x0f;
    pio0_16, PIO0_16, 0x10;
    pio0_17, PIO0_17, 0x11;
    pio0_18, PIO0_18, 0x12;
    pio0_19, PIO0_19, 0x13;
    pio0_20, PIO0_20, 0x14;
    pio0_21, PIO0_21, 0x15;
    pio0_22, PIO0_22, 0x16;
    pio0_23, PIO0_23, 0x17;
    pio0_24, PIO0_24, 0x18;
    pio0_25, PIO0_25, 0x19;
    pio0_26, PIO0_26, 0x1a;
    pio0_27, PIO0_27, 0x1b;
    pio0_28, PIO0_28, 0x1c;
);
