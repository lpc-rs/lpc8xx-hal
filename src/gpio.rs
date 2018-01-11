//! APIs for General Purpose I/O (GPIO)
//!
//! See user manual, chapter 9.


use lpc82x;

use init_state::{
    self,
    InitState,
};
use swm::{
    self,
    FixedFunction,
    MovableFunction,
};
use syscon;


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
    pub fn pins(&mut self) -> Pins {
        Pins::new(self)
    }
}


/// Represents a specific pin
///
/// This trait is implemented by all types that represent a specific pin. HAL
/// users shouldn't need to implement this trait themselves.
///
/// It also should not be necessary for HAL users to use the methods of this
/// trait directly, unless compensating for missing pieces of HAL functionality.
/// Ideally, there should be higher-level peripheral methods that take pins as
/// parameters and use the methods of this trait to take care of the low-level
/// details.
pub trait PinName {
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

    /// Disables all fixed functions for the given pin
    fn disable_fixed_functions(&mut self,
        _swm            : &mut swm::Api,
        _fixed_functions: &mut swm::FixedFunctions,
    )
        where Self: Sized;
}


macro_rules! pins {
    ($($field:ident, $type:ident, $id:expr $(, $fixed_function:ident)*;)*) => {
        /// Provides access to all pins
        #[allow(missing_docs)]
        pub struct Pins<'gpio> {
            $(pub $field: Pin<'gpio, $type>,)*
        }

        impl<'gpio> Pins<'gpio> {
            fn new(gpio: &'gpio GPIO<'gpio>) -> Self {
                Pins {
                    $($field: Pin { gpio: gpio, ty: $type(()) },)*
                }
            }
        }


        $(
            /// Identifies the pin this struct is named after
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl PinName for $type {
                const ID  : u8  = $id;
                const MASK: u32 = 0x1 << $id;

                fn disable_fixed_functions(&mut self,
                    _swm            : &mut swm::Api,
                    _fixed_functions: &mut swm::FixedFunctions,
                ) {
                    #[allow(unused_imports)]
                    use swm::FixedFunction;

                    $(_fixed_functions.$fixed_function.disable(_swm);)*
                }
            }
        )*
    }
}

pins!(
    pio0_0 , PIO0_0 , 0x00, acmp_i1;
    pio0_1 , PIO0_1 , 0x01, acmp_i2, clkin;
    pio0_2 , PIO0_2 , 0x02, swdio;
    pio0_3 , PIO0_3 , 0x03, swclk;
    pio0_4 , PIO0_4 , 0x04, adc_11;
    pio0_5 , PIO0_5 , 0x05, resetn;
    pio0_6 , PIO0_6 , 0x06, vddcmp, adc_1;
    pio0_7 , PIO0_7 , 0x07, adc_0;
    pio0_8 , PIO0_8 , 0x08, xtalin;
    pio0_9 , PIO0_9 , 0x09, xtalout;
    pio0_10, PIO0_10, 0x0a, i2c0_scl;
    pio0_11, PIO0_11, 0x0b, i2c0_sda;
    pio0_12, PIO0_12, 0x0c;
    pio0_13, PIO0_13, 0x0d, adc_10;
    pio0_14, PIO0_14, 0x0e, acmp_i3, adc_2;
    pio0_15, PIO0_15, 0x0f;
    pio0_16, PIO0_16, 0x10;
    pio0_17, PIO0_17, 0x11, adc_9;
    pio0_18, PIO0_18, 0x12, adc_8;
    pio0_19, PIO0_19, 0x13, adc_7;
    pio0_20, PIO0_20, 0x14, adc_6;
    pio0_21, PIO0_21, 0x15, adc_5;
    pio0_22, PIO0_22, 0x16, adc_4;
    pio0_23, PIO0_23, 0x17, acmp_i4, adc_3;
    pio0_24, PIO0_24, 0x18;
    pio0_25, PIO0_25, 0x19;
    pio0_26, PIO0_26, 0x1a;
    pio0_27, PIO0_27, 0x1b;
    pio0_28, PIO0_28, 0x1c;
);


/// A pin that can be used for GPIO, fixed functions, or movable functions
pub struct Pin<'gpio, T: PinName> {
    gpio: &'gpio GPIO<'gpio>,
    ty  : T,
}

impl<'gpio, T> Pin<'gpio, T> where T: PinName {
    /// Enable the fixed function on this pin
    ///
    /// # Limitations
    ///
    /// This method can be used to enable a fixed function for a pin that is
    /// currently used for something else. The HAL user needs to make sure that
    /// the fixed function doesn't conflict with any other uses of the pin.
    ///
    /// This method also doesn't check, whether the fixed function provided
    /// actually belongs to this pin, so it can be used to enable fixed
    /// functions of other pins. The user needs to make sure not to do that or,
    /// at the very least, be intentional about it.
    pub fn enable_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: FixedFunction
    {
        function.enable(swm);
    }

    /// Disable the fixed function on this pin
    ///
    /// # Limitations
    ///
    /// This method can be used to disable a fixed function while other code
    /// relies on that fixed function being enabled. The HAL user needs to make
    /// sure not to use this method in any way that breaks other code.
    ///
    /// This method also doesn't check, whether the fixed function provided
    /// actually belongs to this pin, so it can be used to disable fixed
    /// functions of other pins. The user needs to make sure not to do that or,
    /// at the very least, be intentional about it.
    pub fn disable_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: FixedFunction
    {
        function.disable(swm);
    }

    /// Assign a movable function to the pin
    ///
    /// # Limitations
    ///
    /// This method can be used to assign a movable function to pins that are
    /// currently used for something else. The HAL user needs to make sure that
    /// this assignment doesn't conflict with any other uses of the pin.
    pub fn assign_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: MovableFunction
    {
        function.assign::<T>(&mut self.ty, swm);
    }

    /// Unassign a movable function from the pin
    ///
    /// # Limitations
    ///
    /// This method can be used to unassign a movable function from a pin, while
    /// other parts of the code still rely on that function being assigned. The
    /// HAL user is responsible for making sure this method is used correctly.
    pub fn unassign_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: MovableFunction
    {
        function.unassign::<T>(&mut self.ty, swm);
    }

    /// Sets pin direction to output
    ///
    /// Disables the fixed function of the given pin (thus making it available
    /// for GPIO) and sets the GPIO direction to output.
    pub fn set_pin_to_output(&mut self,
        swm            : &mut swm::Api,
        fixed_functions: &mut swm::FixedFunctions,
    ) {
        self.ty.disable_fixed_functions(swm, fixed_functions);

        self.gpio.gpio.dirset0.write(|w|
            unsafe { w.dirsetp().bits(T::MASK) }
        )
    }

    /// Set pin output to HIGH
    pub fn set_high(&mut self) {
        self.gpio.gpio.set0.write(|w|
            unsafe { w.setp().bits(T::MASK) }
        )
    }

    /// Set pin output to LOW
    pub fn set_low(&mut self) {
        self.gpio.gpio.clr0.write(|w|
            unsafe { w.clrp().bits(T::MASK) }
        );
    }
}
