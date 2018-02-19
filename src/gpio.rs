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
    AdcFunction,
    InputFunction,
    OutputFunction,
};
use swm::fixed_function::{
    self,
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
pub struct Handle<'gpio, State: InitState = init_state::Enabled> {
    gpio  : &'gpio lpc82x::GPIO_PORT,
    _state: State,
}

impl<'gpio> Handle<'gpio, init_state::Unknown> {
    /// Initialize GPIO
    pub fn init(mut self, syscon: &mut syscon::Handle)
        -> Handle<'gpio, init_state::Enabled>
    {
        syscon.enable_clock(&mut self.gpio);
        syscon.clear_reset(&mut self.gpio);

        Handle {
            gpio  : self.gpio,
            _state: init_state::Enabled,
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


/// A pin that can be used for GPIO, fixed functions, or movable functions
pub struct Pin<T: PinName, S: PinState> {
    ty   : T,
    state: S,
}

impl<T> Pin<T, pin_state::Unknown> where T: PinName {
    /// Affirm that the pin is in its default state
    ///
    /// By calling this method, the user promises that the pin is in its default
    /// states. For most pins, this means that the pin is unused, but some pins
    /// are initially assigned to the switch matrix.
    ///
    /// Calling this method is safe, if the pin state has not been changed since
    /// the microcontroller was initialized. If the pin state has been changed
    /// since then, the user must change it back to the pin's initial state
    /// before calling this function.
    pub unsafe fn affirm_default_state(self) -> Pin<T, T::DefaultState> {
        Pin {
            ty   : self.ty,
            state: T::INITIAL_STATE,
        }
    }
}

impl<T> Pin<T, pin_state::Unused> where T: PinName {
    /// Makes the pin available for the ADC
    ///
    /// This method enables the analog function for this pin via the switch
    /// matrix, but as of now, there is no HAL API to actually control the ADC.
    /// You can use this method to enable the analog function and make sure that
    /// no conflicting functions can be enabled for the pin. After that, you
    /// need to use the raw [`IOCON`] and [`ADC`] register mappings to actually
    /// do anything with it.
    ///
    /// [`IOCON`]: ../../lpc82x/constant.IOCON.html
    /// [`ADC`]: ../../lpc82x/constant.ADC.html
    pub fn as_adc_pin<F>(mut self, function: F, swm: &mut swm::Handle)
        -> (Pin<T, pin_state::Adc>, F::Enabled)
        where F: AdcFunction + FixedFunction<Pin=T> + fixed_function::Enable
    {
        let function = function.enable(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Adc,
        };

        (pin, function)
    }

    /// Makes this pin available for GPIO
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

    /// Makes this pin available for function assignment by the switch matrix
    pub fn as_swm_pin(self) -> Pin<T, pin_state::Swm<(), ()>> {
        Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
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

impl<T, Inputs> Pin<T, pin_state::Swm<(), Inputs>> where T: PinName {
    /// Enable the fixed function on this pin
    pub fn enable_output_function<F>(mut self,
            function: F,
            swm     : &mut swm::Handle,
        )
        -> (Pin<T, pin_state::Swm<((),), Inputs>>, F::Enabled)
        where F: OutputFunction + FixedFunction<Pin=T> + fixed_function::Enable
    {
        let function = function.enable(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }

    /// Assign a movable function to the pin
    pub fn assign_output_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<((),), Inputs>>, F::Assigned)
        where F: OutputFunction + movable_function::Assign<T>
    {
        let function = function.assign(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }
}

impl<T, Inputs> Pin<T, pin_state::Swm<((),), Inputs>> where T: PinName {
    /// Disable the fixed function on this pin
    pub fn disable_output_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<(), Inputs>>, F::Disabled)
        where F: OutputFunction + FixedFunction<Pin=T> + fixed_function::Disable
    {
        let function = function.disable(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }

    /// Unassign a movable function from the pin
    pub fn unassign_output_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<(), Inputs>>, F::Unassigned)
        where F: OutputFunction + movable_function::Unassign<T>
    {
        let function = function.unassign(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }
}

impl<T, Output, Inputs> Pin<T, pin_state::Swm<Output, Inputs>>
    where T: PinName
{
    /// Enable the fixed function on this pin
    pub fn enable_input_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<Output, (Inputs,)>>, F::Enabled)
        where F: InputFunction + FixedFunction<Pin=T> + fixed_function::Enable
    {
        let function = function.enable(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }

    /// Assign a movable function to the pin
    pub fn assign_input_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<Output, (Inputs,)>>, F::Assigned)
        where F: InputFunction + movable_function::Assign<T>
    {
        let function = function.assign(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }
}

impl<T, Output, Inputs> Pin<T, pin_state::Swm<Output, (Inputs,)>>
    where T: PinName
{
    /// Disable the fixed function on this pin
    pub fn disable_input_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<Output, Inputs>>, F::Disabled)
        where F: InputFunction + FixedFunction<Pin=T> + fixed_function::Disable
    {
        let function = function.disable(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }

    /// Unassign a movable function from the pin
    pub fn unassign_input_function<F>(mut self,
        function: F,
        swm     : &mut swm::Handle,
    )
        -> (Pin<T, pin_state::Swm<Output, Inputs>>, F::Unassigned)
        where F: InputFunction + movable_function::Unassign<T>
    {
        let function = function.unassign(&mut self.ty, swm);

        let pin = Pin {
            ty   : self.ty,
            state: pin_state::Swm::new(),
        };

        (pin, function)
    }
}

impl<T> Pin<T, pin_state::Swm<(), ()>> where T: PinName {
    /// Marks the pin as being unused
    pub fn as_unused_pin(self) -> Pin<T, pin_state::Unused> {
        Pin {
            ty   : self.ty,
            state: pin_state::Unused,
        }
    }
}


/// Contains types that mark pin states
pub mod pin_state {
    use core::marker::PhantomData;

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


    /// Marks the pin as being unused
    pub struct Unused;

    impl PinState for Unused {}


    /// Marks the pin as being assigned to the analog-to-digital converter
    pub struct Adc;

    impl PinState for Adc {}


    /// Marks a pin as being assigned to general-purpose I/O
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
    /// This type has type parameters that track whether output and input
    /// functions have been assigned to a pin:
    ///
    /// - `Output` tracks whether an output function has been assigned. The only
    ///   valid states are no output functions being assigned, or exactly one
    ///   output function being assigned.
    /// - `Inputs` tracks the number of assigned input functions. Any number of
    ///   input functions may be assigned at the same time.
    ///
    /// Both type parameters use nested tuples to count the number of assigned
    /// functions. The empty tuple (`()`) represents zero assigned functions,
    /// the empty tuple nested in another tuple (`((),)`) represents one
    /// function being assigned, and so forth. This is a bit of a hack, of
    /// course, but it will do until const generics become available.
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
