//! APIs for General Purpose I/O (GPIO)
//!
//! See user manual, chapter 9.


use embedded_hal::digital::OutputPin;
use lpc82x;

use init_state::{
    self,
    InitState,
};
use swm::{
    self,
    movable_function,
    FixedFunction,
};
use syscon;

use self::direction::Direction;
use self::pin_state::PinState;


/// Interface to general-purpose I/O (GPIO)
///
/// This API expects to be the sole owner of the GPIO peripheral. Don't use
/// [`lpc82x::GPIO_PORT`] directly, unless you know what you're doing.
///
/// [`lpc82x::GPIO_PORT`]: ../../lpc82x/struct.GPIO_PORT.html
pub struct GPIO<'gpio> {
    /// Handle for the GPIO peripheral
    pub handle: Handle<'gpio, init_state::Unknown,>,

    /// All pins that can be used for GPIO or other functions
    pub pins: Pins,
}

impl<'gpio> GPIO<'gpio> {
    pub(crate) fn new(gpio: &'gpio lpc82x::GPIO_PORT) -> Self {
        GPIO {
            handle: Handle {
                gpio  : gpio,
                _state: init_state::Unknown,
            },
            pins: Pins::new(),
        }
    }
}


/// Handle for the GPIO peripheral
pub struct Handle<'gpio, State: InitState = init_state::Initialized> {
    gpio  : &'gpio lpc82x::GPIO_PORT,
    _state: State,
}

impl<'gpio> Handle<'gpio, init_state::Unknown> {
    /// Initialize GPIO
    pub fn init(mut self, syscon: &mut syscon::Api)
        -> Handle<'gpio, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.gpio);
        syscon.clear_reset(&mut self.gpio);

        Handle {
            gpio  : self.gpio,
            _state: init_state::Initialized,
        }
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
}


macro_rules! pins {
    ($($field:ident, $type:ident, $id:expr;)*) => {
        /// Provides access to all pins
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
            /// Identifies the pin this struct is named after
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl PinName for $type {
                const ID  : u8  = $id;
                const MASK: u32 = 0x1 << $id;
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


/// A pin that can be used for GPIO, fixed functions, or movable functions
pub struct Pin<T: PinName, S: PinState> {
    ty   : T,
    state: S,
}

impl<T> Pin<T, pin_state::Unknown> where T: PinName {
    /// Enable the fixed function on this pin
    ///
    /// # Limitations
    ///
    /// This method can be used to enable a fixed function for a pin that is
    /// currently used for something else. The HAL user needs to make sure that
    /// the fixed function doesn't conflict with any other uses of the pin.
    pub fn enable_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: FixedFunction<Pin=T>
    {
        function.enable(&mut self.ty, swm);
    }

    /// Disable the fixed function on this pin
    ///
    /// # Limitations
    ///
    /// This method can be used to disable a fixed function while other code
    /// relies on that fixed function being enabled. The HAL user needs to make
    /// sure not to use this method in any way that breaks other code.
    pub fn disable_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: FixedFunction<Pin=T>
    {
        function.disable(&mut self.ty, swm);
    }

    /// Assign a movable function to the pin
    ///
    /// # Limitations
    ///
    /// This method can be used to assign a movable function to pins that are
    /// currently used for something else. The HAL user needs to make sure that
    /// this assignment doesn't conflict with any other uses of the pin.
    pub fn assign_function<F>(&mut self, function: &mut F, swm: &mut swm::Api)
        where F: movable_function::Assign
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
        where F: movable_function::Unassign
    {
        function.unassign::<T>(&mut self.ty, swm);
    }

    /// Makes this pin available for GPIO
    ///
    /// # Limitations
    ///
    /// This method doesn't disable any fixed functions or unsassign any
    /// movable functions. Before calling this function, the user must manually
    /// disable or unassign any active functions on this pin.
    pub fn as_gpio_pin<'gpio>(self, gpio: &'gpio Handle<'gpio>)
        -> Pin<T, pin_state::Gpio<'gpio, direction::Unknown>>
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
}

impl<'gpio, T, D> Pin<T, pin_state::Gpio<'gpio, D>>
    where
        T: PinName,
        D: Direction + direction::NotOutput,
{
    /// Sets pin direction to output
    pub fn as_output(self)
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
    where T: PinName
{
    fn is_high(&self) -> bool {
        self.state.pin0.read().port().bits() & T::MASK == T::MASK
    }

    fn is_low(&self) -> bool {
        !self.state.pin0.read().port().bits() & T::MASK == T::MASK
    }

    /// Set pin output to HIGH
    fn set_high(&mut self) {
        self.state.set0.write(|w|
            unsafe { w.setp().bits(T::MASK) }
        )
    }

    /// Set pin output to LOW
    fn set_low(&mut self) {
        self.state.clr0.write(|w|
            unsafe { w.clrp().bits(T::MASK) }
        );
    }
}


/// Contains types that mark pin states
pub mod pin_state {
    use lpc82x::gpio_port::{
        CLR0,
        DIRSET0,
        PIN0,
        SET0,
    };

    use super::direction::Direction;


    /// Implemented by types that indicate pin state
    ///
    /// This type is used as a trait bound for type parameters that indicate a
    /// pin's state. HAL users should never need to implement this trait, nor
    /// use it directly.
    pub trait PinState {}


    /// Marks a pin's state as being unknown
    ///
    /// As we can't know what happened to the hardware before the HAL was
    /// initializized, this is the initial state of all pins.
    pub struct Unknown;

    impl PinState for Unknown {}


    /// Marks a pin as being assigned to general-purpose I/O
    pub struct Gpio<'gpio, D: Direction> {
        pub(crate) dirset0: &'gpio DIRSET0,
        pub(crate) pin0   : &'gpio PIN0,
        pub(crate) set0   : &'gpio SET0,
        pub(crate) clr0   : &'gpio CLR0,

        pub(crate) _direction: D,
    }

    impl<'gpio, D> PinState for Gpio<'gpio, D> where D: Direction {}
}


/// Contains types that mark the direction of GPIO pins
pub mod direction {
    /// Implemented by types that indicate GPIO pin direction
    pub trait Direction {}

    /// Marks a pin's GPIO direction as being unknown
    ///
    /// As we can't know what happened to the hardware before the HAL was
    /// initialized, this is the initial state.
    pub struct Unknown;
    impl Direction for Unknown {}

    /// Marks a GPIO pin as being configured for input
    pub struct Input;
    impl Direction for Input {}

    /// Marks a GPIO pin as being configured for output
    pub struct Output;
    impl Direction for Output {}


    /// Marks a direction as being unknown or input (i.e. not being output)
    ///
    /// This is a helper trait used to more precisely define `impl` blocks. It
    /// is of no concern to users of this crate.
    pub trait NotOutput: Direction {}

    impl NotOutput for Unknown {}
    impl NotOutput for Input {}
}
