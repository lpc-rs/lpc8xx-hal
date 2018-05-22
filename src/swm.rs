//! APIs for the switch matrix (SWM)
//!
//! See user manual, chapter 7.


use core::marker::PhantomData;

use gpio::{
    PIO0_2,
    PIO0_3,
    PIO0_5,
    Pin,
    PinTrait,
};
use gpio::pin_state::PinState;
use init_state::{
    self,
    InitState,
};
use raw;
use syscon;


/// Interface to the switch matrix (SWM)
pub struct SWM {
    /// Main SWM API
    pub handle: Handle<init_state::Unknown>,

    /// Movable functions
    pub movable_functions: MovableFunctions,

    /// Fixed functions
    pub fixed_functions: FixedFunctions,
}

impl SWM {
    /// Create an instance of `SWM`
    pub fn new(swm: raw::SWM) -> Self {
        SWM {
            handle           : Handle::new(swm),
            movable_functions: MovableFunctions::new(),
            fixed_functions  : FixedFunctions::new(),
        }
    }
}


/// Main API of the SWM peripheral
pub struct Handle<State: InitState = init_state::Enabled> {
    swm   : raw::SWM,
    _state: State,
}

impl Handle<init_state::Unknown> {
    pub(crate) fn new(swm: raw::SWM) -> Self {
        Handle {
            swm   : swm,
            _state: init_state::Unknown,
        }
    }
}

impl<State> Handle<State> where State: init_state::NotEnabled {
    /// Enable the switch matrix
    ///
    /// This method is only available, if `swm::Handle` is not already in the
    /// [`Enabled`] state. Code that attempts to call this method when the
    /// switch matrix is already enabled will not compile.
    ///
    /// Consumes this instance of `swm::Handle` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(mut self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Enabled>
    {
        syscon.enable_clock(&mut self.swm);

        Handle {
            swm   : self.swm,
            _state: init_state::Enabled,
        }
    }
}

impl<State> Handle<State> where State: init_state::NotDisabled {
    /// Disable the switch matrix
    ///
    /// This method is only available, if `swm::Handle` is not already in the
    /// [`Disabled`] state. Code that attempts to call this method when the
    /// switch matrix is already disabled will not compile.
    ///
    /// Consumes this instance of `swm::Handle` and returns another instance
    /// that has its `State` type parameter set to [`Disabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(mut self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Disabled>
    {
        syscon.disable_clock(&mut self.swm);

        Handle {
            swm   : self.swm,
            _state: init_state::Disabled,
        }
    }
}


/// A movable function that can be assigned to any pin
pub struct Function<T, State> {
    ty    : T,
    _state: State,
}

impl<T> Function<T, state::Unknown> where T: DefaultState {
    /// Affirm that the movable function is in its default state
    ///
    /// By calling this method, the user promises that the movable function is
    /// in its default state. This is safe to do, if nothing has changed that
    /// state before the HAL has been initialized.
    ///
    /// If the movable function's state has been changed by any other means than
    /// the HAL API, then the user must use those means to return the movable
    /// function to its default state, as specified in the user manual, before
    /// calling this method.
    pub unsafe fn affirm_default_state(self) -> Function<T, T::DefaultState> {
        Function {
            ty    : self.ty,
            _state: state::State::new(),
        }
    }
}

impl<T> Function<T, state::Unassigned> {
    /// Assign the movable function to a pin
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::assign_input_function`] and [`Pin::assign_output_function`]
    /// instead.
    ///
    /// [`Pin::assign_input_function`]: ../gpio/struct.Pin.html#method.assign_input_function
    /// [`Pin::assign_output_function`]: ../gpio/struct.Pin.html#method.assign_output_function
    pub fn assign<P, S>(mut self, mut pin: Pin<P, S>, swm: &mut Handle)
        -> (
            Function<T, state::Assigned<P>>,
            <Pin<P, S> as AssignFunction<T, T::Kind>>::Assigned,
        )
        where
            T        : FunctionTrait<P>,
            P        : PinTrait,
            S        : PinState,
            Pin<P, S>: AssignFunction<T, T::Kind>,
    {
        self.ty.assign(&mut pin.ty, swm);

        let function = Function {
            ty    : self.ty,
            _state: state::Assigned(PhantomData),
        };

        (function, pin.assign())
    }
}

impl<T, P> Function<T, state::Assigned<P>> {
    /// Unassign the movable function
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::unassign_input_function`] and
    /// [`Pin::unassign_output_function`] instead.
    ///
    /// [`Pin::unassign_input_function`]: ../gpio/struct.Pin.html#method.unassign_input_function
    /// [`Pin::unassign_output_function`]: ../gpio/struct.Pin.html#method.unassign_input_function
    pub fn unassign(mut self, pin: &mut P, swm: &mut Handle)
        -> Function<T, state::Unassigned>
        where
            T: FunctionTrait<P>,
            P: PinTrait,
    {
        self.ty.unassign(pin, swm);

        Function {
            ty    : self.ty,
            _state: state::Unassigned,
        }
    }
}


/// Implemented by all functions
pub trait DefaultState {
    /// The default state of this function
    type DefaultState: state::State;
}


/// Implemented by all movable functions
pub trait FunctionTrait<P: PinTrait> {
    /// Whether this is an input or output function
    ///
    /// There are also bidirectional functions, but for our purposes, they are
    /// treated as output functions.
    type Kind: FunctionKind;


    /// Assigns the movable function to a pin
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::assign_input_function`] and [`Pin::assign_output_function`]
    /// instead.
    ///
    /// [`Pin::assign_input_function`]: ../gpio/struct.Pin.html#method.assign_input_function
    /// [`Pin::assign_output_function`]: ../gpio/struct.Pin.html#method.assign_output_function
    fn assign(&mut self, pin: &mut P, swm: &mut Handle);

    /// Unassign the movable function
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::unassign_input_function`] and
    /// [`Pin::unassign_output_function`] instead.
    ///
    /// [`Pin::unassign_input_function`]: ../gpio/struct.Pin.html#method.unassign_input_function
    /// [`Pin::unassign_output_function`]: ../gpio/struct.Pin.html#method.unassign_input_function
    fn unassign(&mut self, pin: &mut P, swm: &mut Handle);
}


/// Implemented for types that designate whether a function is input or output
pub trait FunctionKind {}

/// Designates an SWM function as an input function
pub struct Input;
impl FunctionKind for Input {}

/// Designates an SWM function as an output function
pub struct Output;
impl FunctionKind for Output {}

/// Designates an SWM function as an ADC function
pub struct Adc;
impl FunctionKind for Adc {}


/// Internal trait used to assign functions to pins
pub trait AssignFunction<Function, Kind> {
    /// The type of the pin after the function has been assigned
    type Assigned;

    /// Internal method for assigning a function to a pin
    fn assign(self) -> Self::Assigned;
}

/// Internal trait used to unassign functions from pins
pub trait UnassignFunction<Function, Kind> {
    /// The type of the pin after the function has been unassigned
    type Unassigned;

    /// Internal method for unassigning a function from a pin
    fn unassign(self) -> Self::Unassigned;
}


macro_rules! movable_functions {
    (
        $(
            $field:ident,
            $type:ident,
            $kind:ident,
            $reg_name:ident,
            $reg_field:ident;
        )*
    ) => {
        /// Provides access to all movable functions
        ///
        /// This struct is part of [`SWM`].
        ///
        /// [`SWM`]: struct.SWM.html
        #[allow(missing_docs)]
        pub struct MovableFunctions {
            $(pub $field: Function<$type, state::Unknown>,)*
        }

        impl MovableFunctions {
            fn new() -> Self {
                MovableFunctions {
                    $($field: Function {
                        ty    : $type(()),
                        _state: state::Unknown,
                    },)*
                }
            }
        }


        $(
            /// Represents a movable function
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl DefaultState for $type {
                type DefaultState = state::Unassigned;
            }

            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_0 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_1 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_2 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_3 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_4 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_5 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_6 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_7 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_8 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_9 );
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_10);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_11);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_12);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_13);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_14);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_15);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_16);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_17);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_18);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_19);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_20);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_21);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_22);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_23);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_24);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_25);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_26);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_27);
            impl_function!($type, $kind, $reg_name, $reg_field, PIO0_28);
        )*
    }
}

macro_rules! impl_function {
    (
        $type:ident,
        $kind:ident,
        $reg_name:ident,
        $reg_field:ident,
        $pin:ident
    ) => {
        impl FunctionTrait<::gpio::$pin> for $type {
            type Kind = $kind;


            fn assign(&mut self, _pin: &mut ::gpio::$pin, swm : &mut Handle) {
                swm.swm.$reg_name.modify(|_, w|
                    unsafe { w.$reg_field().bits(::gpio::$pin::ID) }
                );
            }

            fn unassign(&mut self, _pin: &mut ::gpio::$pin, swm : &mut Handle) {
                swm.swm.$reg_name.modify(|_, w|
                    unsafe { w.$reg_field().bits(0xff) }
                );
            }
        }
    }
}

movable_functions!(
    u0_txd       , U0_TXD       , Output, pinassign0 , u0_txd_o;
    u0_rxd       , U0_RXD       , Input , pinassign0 , u0_rxd_i;
    u0_rts       , U0_RTS       , Output, pinassign0 , u0_rts_o;
    u0_cts       , U0_CTS       , Input , pinassign0 , u0_cts_i;
    u0_sclk      , U0_SCLK      , Output, pinassign1 , u0_sclk_io;
    u1_txd       , U1_TXD       , Output, pinassign1 , u1_txd_o;
    u1_rxd       , U1_RXD       , Input , pinassign1 , u1_rxd_i;
    u1_rts       , U1_RTS       , Output, pinassign1 , u1_rts_o;
    u1_cts       , U1_CTS       , Input , pinassign2 , u1_cts_i;
    u1_sclk      , U1_SCLK      , Output, pinassign2 , u1_sclk_io;
    u2_txd       , U2_TXD       , Output, pinassign2 , u2_txd_o;
    u2_rxd       , U2_RXD       , Input , pinassign2 , u2_rxd_i;
    u2_rts       , U2_RTS       , Output, pinassign3 , u2_rts_o;
    u2_cts       , U2_CTS       , Input , pinassign3 , u2_cts_i;
    u2_sclk      , U2_SCLK      , Output, pinassign3 , u2_sclk_io;
    spi0_sck     , SPI0_SCK     , Output, pinassign3 , spi0_sck_io;
    spi0_mosi    , SPI0_MOSI    , Output, pinassign4 , spi0_mosi_io;
    spi0_miso    , SPI0_MISO    , Output, pinassign4 , spi0_miso_io;
    spi0_ssel0   , SPI0_SSEL0   , Output, pinassign4 , spi0_ssel0_io;
    spi0_ssel1   , SPI0_SSEL1   , Output, pinassign4 , spi0_ssel1_io;
    spi0_ssel2   , SPI0_SSEL2   , Output, pinassign5 , spi0_ssel2_io;
    spi0_ssel3   , SPI0_SSEL3   , Output, pinassign5 , spi0_ssel3_io;
    spi1_sck     , SPI1_SCK     , Output, pinassign5 , spi1_sck_io;
    spi1_mosi    , SPI1_MOSI    , Output, pinassign5 , spi1_mosi_io;
    spi1_miso    , SPI1_MISO    , Output, pinassign6 , spi1_miso_io;
    spi1_ssel0   , SPI1_SSEL0   , Output, pinassign6 , spi1_ssel0_io;
    spi1_ssel1   , SPI1_SSEL1   , Output, pinassign6 , spi1_ssel1_io;
    sct_pin0     , SCT_PIN0     , Input , pinassign6 , sct_in0_i;
    sct_pin1     , SCT_PIN1     , Input , pinassign7 , sct_in1_i;
    sct_pin2     , SCT_PIN2     , Input , pinassign7 , sct_in2_i;
    sct_pin3     , SCT_PIN3     , Input , pinassign7 , sct_in3_i;
    sct_out0     , SCT_OUT0     , Output, pinassign7 , sct_out0_o;
    sct_out1     , SCT_OUT1     , Output, pinassign8 , sct_out1_o;
    sct_out2     , SCT_OUT2     , Output, pinassign8 , sct_out2_o;
    sct_out3     , SCT_OUT3     , Output, pinassign8 , sct_out3_o;
    sct_out4     , SCT_OUT4     , Output, pinassign8 , sct_out4_o;
    sct_out5     , SCT_OUT5     , Output, pinassign9 , sct_out5_o;
    i2c1_sda     , I2C1_SDA     , Output, pinassign9 , i2c1_sda_io;
    i2c1_scl     , I2C1_SCL     , Output, pinassign9 , i2c1_scl_io;
    i2c2_sda     , I2C2_SDA     , Output, pinassign9 , i2c2_sda_io;
    i2c2_scl     , I2C2_SCL     , Output, pinassign10, i2c2_scl_io;
    i2c3_sda     , I2C3_SDA     , Output, pinassign10, i2c3_sda_io;
    i2c3_scl     , I2C3_SCL     , Output, pinassign10, i2c3_scl_io;
    adc_pintrig0 , ADC_PINTRIG0 , Input , pinassign10, adc_pintrig0_i;
    acd_pintrig1 , ADC_PINTRIG1 , Input , pinassign11, adc_pintrig1_i;
    acmp_o       , ACMP_O       , Output, pinassign11, acmp_o_o;
    clkout       , CLKOUT       , Output, pinassign11, clkout_o;
    gpio_int_bmat, GPIO_INT_BMAT, Output, pinassign11, gpio_int_bmat_o;
);


macro_rules! fixed_functions {
    ($(
        $type:ident,
        $kind:ident,
        $field:ident,
        $pin:ident,
        $default_state:ty;
    )*) => {
        /// Provides access to all fixed functions
        ///
        /// This struct is part of [`SWM`].
        ///
        /// [`SWM`]: struct.SWM.html
        #[allow(missing_docs)]
        pub struct FixedFunctions {
            $(pub $field: Function<$type, state::Unknown>,)*
        }

        impl FixedFunctions {
            fn new() -> Self {
                FixedFunctions {
                    $($field: Function {
                        ty    : $type(()),
                        _state: state::State::new(),
                    },)*
                }
            }
        }


        $(
            /// Represents a fixed function
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl DefaultState for $type {
                type DefaultState = $default_state;
            }

            impl FunctionTrait<::gpio::$pin> for $type {
                type Kind = $kind;


                fn assign(&mut self, _: &mut ::gpio::$pin, swm : &mut Handle) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().clear_bit());
                }

                fn unassign(&mut self, _: &mut ::gpio::$pin, swm : &mut Handle)
                {
                    swm.swm.pinenable0.modify(|_, w| w.$field().set_bit());
                }
            }
        )*
    }
}

fixed_functions!(
    ACMP_I1 , Input , acmp_i1 , PIO0_0 , state::Unassigned;
    ACMP_I2 , Input , acmp_i2 , PIO0_1 , state::Unassigned;
    ACMP_I3 , Input , acmp_i3 , PIO0_14, state::Unassigned;
    ACMP_I4 , Input , acmp_i4 , PIO0_23, state::Unassigned;
    SWCLK   , Output, swclk   , PIO0_3 , state::Assigned<PIO0_3>;
    SWDIO   , Output, swdio   , PIO0_2 , state::Assigned<PIO0_2>;
    XTALIN  , Input , xtalin  , PIO0_8 , state::Unassigned;
    XTALOUT , Output, xtalout , PIO0_9 , state::Unassigned;
    RESETN  , Input , resetn  , PIO0_5 , state::Assigned<PIO0_5>;
    CLKIN   , Input , clkin   , PIO0_1 , state::Unassigned;
    VDDCMP  , Input , vddcmp  , PIO0_6 , state::Unassigned;
    I2C0_SDA, Output, i2c0_sda, PIO0_11, state::Unassigned;
    I2C0_SCL, Output, i2c0_scl, PIO0_10, state::Unassigned;
    ADC_0   , Adc   , adc_0   , PIO0_7 , state::Unassigned;
    ADC_1   , Adc   , adc_1   , PIO0_6 , state::Unassigned;
    ADC_2   , Adc   , adc_2   , PIO0_14, state::Unassigned;
    ADC_3   , Adc   , adc_3   , PIO0_23, state::Unassigned;
    ADC_4   , Adc   , adc_4   , PIO0_22, state::Unassigned;
    ADC_5   , Adc   , adc_5   , PIO0_21, state::Unassigned;
    ADC_6   , Adc   , adc_6   , PIO0_20, state::Unassigned;
    ADC_7   , Adc   , adc_7   , PIO0_19, state::Unassigned;
    ADC_8   , Adc   , adc_8   , PIO0_18, state::Unassigned;
    ADC_9   , Adc   , adc_9   , PIO0_17, state::Unassigned;
    ADC_10  , Adc   , adc_10  , PIO0_13, state::Unassigned;
    ADC_11  , Adc   , adc_11  , PIO0_4 , state::Unassigned;
);


/// Contains types that indicate the state of a movable function
pub mod state {
    use core::marker::PhantomData;


    /// Implemented by types that indicate the state of a movable function
    ///
    /// This trait is implemented by types that indicate the state of a movable
    /// function. It exists only to document which types those are. The user
    /// should not need to implement this trait, nor use it directly.
    pub trait State {
        /// Returns an instance of the state
        ///
        /// This method is intended for internal use. Any changes to this method
        /// won't be considered breaking changes.
        fn new() -> Self;
    }


    /// Indicates that the current state of the movable function is unknown
    ///
    /// This is the case after the HAL is initialized, as we can't know what
    /// happened before that.
    pub struct Unknown;

    impl State for Unknown {
        fn new() -> Self { Unknown }
    }


    /// Indicates that the movable function is unassigned
    pub struct Unassigned;

    impl State for Unassigned {
        fn new() -> Self { Unassigned }
    }


    /// Indicates that the movable function is assigned to a pin
    pub struct Assigned<Pin>(pub(crate) PhantomData<Pin>);

    impl<Pin> State for Assigned<Pin> {
        fn new() -> Self { Assigned(PhantomData) }
    }
}
