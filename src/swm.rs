//! APIs for the switch matrix (SWM)
//!
//! See user manual, chapter 7.


use core::marker::PhantomData;

use gpio::{
    PIO0_0,
    PIO0_1,
    PIO0_2,
    PIO0_3,
    PIO0_4,
    PIO0_5,
    PIO0_6,
    PIO0_7,
    PIO0_8,
    PIO0_9,
    PIO0_10,
    PIO0_11,
    PIO0_13,
    PIO0_14,
    PIO0_17,
    PIO0_18,
    PIO0_19,
    PIO0_20,
    PIO0_21,
    PIO0_22,
    PIO0_23,
    PinTrait,
};
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

impl<T> Function<T, state::Unknown> {
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
    pub unsafe fn affirm_default_state(self) -> Function<T, state::Unassigned> {
        Function {
            ty    : self.ty,
            _state: state::Unassigned,
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
    pub fn assign<P>(mut self, pin: &mut P, swm: &mut Handle)
        -> Function<T, state::Assigned<P>>
        where
            T: FunctionTrait<P>,
            P: PinTrait,
    {
        self.ty.assign(pin, swm);

        Function {
            ty    : self.ty,
            _state: state::Assigned(PhantomData),
        }
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


/// Implemented by all movable functions
pub trait FunctionTrait<P: PinTrait> {
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


macro_rules! movable_functions {
    (
        $(
            $field:ident,
            $type:ident,
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

            impl_function!($type, $reg_name, $reg_field, PIO0_0 );
            impl_function!($type, $reg_name, $reg_field, PIO0_1 );
            impl_function!($type, $reg_name, $reg_field, PIO0_2 );
            impl_function!($type, $reg_name, $reg_field, PIO0_3 );
            impl_function!($type, $reg_name, $reg_field, PIO0_4 );
            impl_function!($type, $reg_name, $reg_field, PIO0_5 );
            impl_function!($type, $reg_name, $reg_field, PIO0_6 );
            impl_function!($type, $reg_name, $reg_field, PIO0_7 );
            impl_function!($type, $reg_name, $reg_field, PIO0_8 );
            impl_function!($type, $reg_name, $reg_field, PIO0_9 );
            impl_function!($type, $reg_name, $reg_field, PIO0_10);
            impl_function!($type, $reg_name, $reg_field, PIO0_11);
            impl_function!($type, $reg_name, $reg_field, PIO0_12);
            impl_function!($type, $reg_name, $reg_field, PIO0_13);
            impl_function!($type, $reg_name, $reg_field, PIO0_14);
            impl_function!($type, $reg_name, $reg_field, PIO0_15);
            impl_function!($type, $reg_name, $reg_field, PIO0_16);
            impl_function!($type, $reg_name, $reg_field, PIO0_17);
            impl_function!($type, $reg_name, $reg_field, PIO0_18);
            impl_function!($type, $reg_name, $reg_field, PIO0_19);
            impl_function!($type, $reg_name, $reg_field, PIO0_20);
            impl_function!($type, $reg_name, $reg_field, PIO0_21);
            impl_function!($type, $reg_name, $reg_field, PIO0_22);
            impl_function!($type, $reg_name, $reg_field, PIO0_23);
            impl_function!($type, $reg_name, $reg_field, PIO0_24);
            impl_function!($type, $reg_name, $reg_field, PIO0_25);
            impl_function!($type, $reg_name, $reg_field, PIO0_26);
            impl_function!($type, $reg_name, $reg_field, PIO0_27);
            impl_function!($type, $reg_name, $reg_field, PIO0_28);
        )*
    }
}

macro_rules! impl_function {
    ($type:ident, $reg_name:ident, $reg_field:ident, $pin:ident) => {
        impl FunctionTrait<::gpio::$pin> for $type {
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
    u0_txd       , U0_TXD       , pinassign0 , u0_txd_o;
    u0_rxd       , U0_RXD       , pinassign0 , u0_rxd_i;
    u0_rts       , U0_RTS       , pinassign0 , u0_rts_o;
    u0_cts       , U0_CTS       , pinassign0 , u0_cts_i;
    u0_sclk      , U0_SCLK      , pinassign1 , u0_sclk_io;
    u1_txd       , U1_TXD       , pinassign1 , u1_txd_o;
    u1_rxd       , U1_RXD       , pinassign1 , u1_rxd_i;
    u1_rts       , U1_RTS       , pinassign1 , u1_rts_o;
    u1_cts       , U1_CTS       , pinassign2 , u1_cts_i;
    u1_sclk      , U1_SCLK      , pinassign2 , u1_sclk_io;
    u2_txd       , U2_TXD       , pinassign2 , u2_txd_o;
    u2_rxd       , U2_RXD       , pinassign2 , u2_rxd_i;
    u2_rts       , U2_RTS       , pinassign3 , u2_rts_o;
    u2_cts       , U2_CTS       , pinassign3 , u2_cts_i;
    u2_sclk      , U2_SCLK      , pinassign3 , u2_sclk_io;
    spi0_sck     , SPI0_SCK     , pinassign3 , spi0_sck_io;
    spi0_mosi    , SPI0_MOSI    , pinassign4 , spi0_mosi_io;
    spi0_miso    , SPI0_MISO    , pinassign4 , spi0_miso_io;
    spi0_ssel0   , SPI0_SSEL0   , pinassign4 , spi0_ssel0_io;
    spi0_ssel1   , SPI0_SSEL1   , pinassign4 , spi0_ssel1_io;
    spi0_ssel2   , SPI0_SSEL2   , pinassign5 , spi0_ssel2_io;
    spi0_ssel3   , SPI0_SSEL3   , pinassign5 , spi0_ssel3_io;
    spi1_sck     , SPI1_SCK     , pinassign5 , spi1_sck_io;
    spi1_mosi    , SPI1_MOSI    , pinassign5 , spi1_mosi_io;
    spi1_miso    , SPI1_MISO    , pinassign6 , spi1_miso_io;
    spi1_ssel0   , SPI1_SSEL0   , pinassign6 , spi1_ssel0_io;
    spi1_ssel1   , SPI1_SSEL1   , pinassign6 , spi1_ssel1_io;
    sct_pin0     , SCT_PIN0     , pinassign6 , sct_in0_i;
    sct_pin1     , SCT_PIN1     , pinassign7 , sct_in1_i;
    sct_pin2     , SCT_PIN2     , pinassign7 , sct_in2_i;
    sct_pin3     , SCT_PIN3     , pinassign7 , sct_in3_i;
    sct_out0     , SCT_OUT0     , pinassign7 , sct_out0_o;
    sct_out1     , SCT_OUT1     , pinassign8 , sct_out1_o;
    sct_out2     , SCT_OUT2     , pinassign8 , sct_out2_o;
    sct_out3     , SCT_OUT3     , pinassign8 , sct_out3_o;
    sct_out4     , SCT_OUT4     , pinassign8 , sct_out4_o;
    sct_out5     , SCT_OUT5     , pinassign9 , sct_out5_o;
    i2c1_sda     , I2C1_SDA     , pinassign9 , i2c1_sda_io;
    i2c1_scl     , I2C1_SCL     , pinassign9 , i2c1_scl_io;
    i2c2_sda     , I2C2_SDA     , pinassign9 , i2c2_sda_io;
    i2c2_scl     , I2C2_SCL     , pinassign10, i2c2_scl_io;
    i2c3_sda     , I2C3_SDA     , pinassign10, i2c3_sda_io;
    i2c3_scl     , I2C3_SCL     , pinassign10, i2c3_scl_io;
    adc_pintrig0 , ADC_PINTRIG0 , pinassign10, adc_pintrig0_i;
    acd_pintrig1 , ADC_PINTRIG1 , pinassign11, adc_pintrig1_i;
    acmp_o       , ACMP_O       , pinassign11, acmp_o_o;
    clkout       , CLKOUT       , pinassign11, clkout_o;
    gpio_int_bmat, GPIO_INT_BMAT, pinassign11, gpio_int_bmat_o;
);


/// A fixed function that can be enabled on a specific pin
pub struct FixedFunction<T, State> {
    ty    : T,
    _state: State,
}

impl<T> FixedFunction<T, init_state::Unknown> {
    /// Affirm that the fixed function is in its default state
    ///
    /// By calling this method, the user promises that the fixed function is in
    /// its default state. This is safe to do, if nothing has changed that state
    /// before the HAL has been initialized.
    ///
    /// If the fixed function's state has been changed by any other means than
    /// the HAL API, then the user must use those means to return the fixed
    /// function to its default state, as specified in the user manual, before
    /// calling this method.
    pub unsafe fn affirm_default_state(self)
        -> FixedFunction<T, T::DefaultState>
        where T: FixedFunctionTrait
    {
        FixedFunction {
            ty    : self.ty,
            _state: InitState::new(),
        }
    }
}

impl<T> FixedFunction<T, init_state::Disabled> {
    /// Enable the fixed function
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::enable_input_function`] and [`Pin::enable_output_function`]
    /// instead.
    ///
    /// [`Pin::enable_input_function`]: ../gpio/struct.Pin.html#method.enable_input_function
    /// [`Pin::enable_output_function`]: ../gpio/struct.Pin.html#method.enable_output_function
    pub fn enable(mut self, pin: &mut T::Pin, swm: &mut Handle)
        -> FixedFunction<T, init_state::Enabled>
        where T: FixedFunctionTrait
    {
        self.ty.enable(pin, swm);

        FixedFunction {
            ty    : self.ty,
            _state: init_state::Enabled,
        }
    }
}

impl<T> FixedFunction<T, init_state::Enabled> {
    /// Disable the fixed function
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::disable_input_function`] and [`Pin::disable_output_function`]
    /// instead.
    ///
    /// [`Pin::disable_input_function`]: ../gpio/struct.Pin.html#method.disable_input_function
    /// [`Pin::disable_output_function`]: ../gpio/struct.Pin.html#method.disable_output_function
    pub fn disable(mut self, pin: &mut T::Pin, swm: &mut Handle)
        -> FixedFunction<T, init_state::Disabled>
        where T: FixedFunctionTrait
    {
        self.ty.disable(pin, swm);

        FixedFunction {
            ty    : self.ty,
            _state: init_state::Disabled,
        }
    }
}


/// A fixed function
///
/// This trait is implemented for all types that represent fixed functions.
/// The user should not need to implement this trait, nor use it directly.
/// Any changes to this trait will not be considered breaking changes.
pub trait FixedFunctionTrait {
    /// The pin that this fixed function can be enabled on
    type Pin: PinTrait;

    /// The default state of this function
    type DefaultState: InitState;


    /// Enable the fixed function
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::enable_input_function`] and [`Pin::enable_output_function`]
    /// instead.
    ///
    /// [`Pin::enable_input_function`]: ../gpio/struct.Pin.html#method.enable_input_function
    /// [`Pin::enable_output_function`]: ../gpio/struct.Pin.html#method.enable_output_function
    fn enable(&mut self, pin: &mut Self::Pin, swm: &mut Handle);

    /// Disable the fixed function
    ///
    /// This method is intended for internal use only. Please use
    /// [`Pin::disable_input_function`] and [`Pin::disable_output_function`]
    /// instead.
    ///
    /// [`Pin::disable_input_function`]: ../gpio/struct.Pin.html#method.disable_input_function
    /// [`Pin::disable_output_function`]: ../gpio/struct.Pin.html#method.disable_output_function
    fn disable(&mut self, pin: &mut Self::Pin, swm: &mut Handle);
}


macro_rules! fixed_functions {
    ($($type:ident, $field:ident, $pin:ident, $default_state:ident;)*) => {
        /// Provides access to all fixed functions
        ///
        /// This struct is part of [`SWM`].
        ///
        /// [`SWM`]: struct.SWM.html
        #[allow(missing_docs)]
        pub struct FixedFunctions {
            $(pub $field: FixedFunction<$type, init_state::Unknown>,)*
        }

        impl FixedFunctions {
            fn new() -> Self {
                FixedFunctions {
                    $($field: FixedFunction {
                        ty    : $type(()),
                        _state: init_state::Unknown,
                    },)*
                }
            }
        }


        $(
            /// Represents a fixed function
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl FunctionTrait<::gpio::$pin> for $type {
                fn assign(&mut self, _: &mut ::gpio::$pin, swm : &mut Handle) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().clear_bit());
                }

                fn unassign(&mut self, _: &mut ::gpio::$pin, swm : &mut Handle)
                {
                    swm.swm.pinenable0.modify(|_, w| w.$field().set_bit());
                }
            }

            impl FixedFunctionTrait for $type {
                type Pin = $pin;

                type DefaultState = init_state::$default_state;


                fn enable(&mut self, _pin: &mut Self::Pin, swm: &mut Handle) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().clear_bit());
                }

                fn disable(&mut self, _pin: &mut Self::Pin, swm: &mut Handle) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().set_bit());
                }
            }
        )*
    }
}

fixed_functions!(
    ACMP_I1 , acmp_i1 , PIO0_0 , Disabled;
    ACMP_I2 , acmp_i2 , PIO0_1 , Disabled;
    ACMP_I3 , acmp_i3 , PIO0_14, Disabled;
    ACMP_I4 , acmp_i4 , PIO0_23, Disabled;
    SWCLK   , swclk   , PIO0_3 , Enabled;
    SWDIO   , swdio   , PIO0_2 , Enabled;
    XTALIN  , xtalin  , PIO0_8 , Disabled;
    XTALOUT , xtalout , PIO0_9 , Disabled;
    RESETN  , resetn  , PIO0_5 , Enabled;
    CLKIN   , clkin   , PIO0_1 , Disabled;
    VDDCMP  , vddcmp  , PIO0_6 , Disabled;
    I2C0_SDA, i2c0_sda, PIO0_11, Disabled;
    I2C0_SCL, i2c0_scl, PIO0_10, Disabled;
    ADC_0   , adc_0   , PIO0_7 , Disabled;
    ADC_1   , adc_1   , PIO0_6 , Disabled;
    ADC_2   , adc_2   , PIO0_14, Disabled;
    ADC_3   , adc_3   , PIO0_23, Disabled;
    ADC_4   , adc_4   , PIO0_22, Disabled;
    ADC_5   , adc_5   , PIO0_21, Disabled;
    ADC_6   , adc_6   , PIO0_20, Disabled;
    ADC_7   , adc_7   , PIO0_19, Disabled;
    ADC_8   , adc_8   , PIO0_18, Disabled;
    ADC_9   , adc_9   , PIO0_17, Disabled;
    ADC_10  , adc_10  , PIO0_13, Disabled;
    ADC_11  , adc_11  , PIO0_4 , Disabled;
);


/// Marker trait for fixed functions representing ADC channels
///
/// This is an internal trait. Any changes made to it won't be considered
/// breaking changes.
pub trait AdcChannel {}

impl AdcChannel for ADC_0 {}
impl AdcChannel for ADC_1 {}
impl AdcChannel for ADC_2 {}
impl AdcChannel for ADC_3 {}
impl AdcChannel for ADC_4 {}
impl AdcChannel for ADC_5 {}
impl AdcChannel for ADC_6 {}
impl AdcChannel for ADC_7 {}
impl AdcChannel for ADC_8 {}
impl AdcChannel for ADC_9 {}
impl AdcChannel for ADC_10 {}
impl AdcChannel for ADC_11 {}


/// Marker trait for output functions
///
/// This trait marks all functions that include output, which means
/// bidirectional functions are also included.
///
/// This is an internal trait. Any changes made to it won't be considered
/// breaking changes.
pub trait OutputFunction {}

// Which movable functions are output functions is documented in the user manual
// in section 7.4.1, table 65.
impl OutputFunction for U0_TXD {}
impl OutputFunction for U0_RTS {}
impl OutputFunction for U0_SCLK {}
impl OutputFunction for U1_TXD {}
impl OutputFunction for U1_RTS {}
impl OutputFunction for U1_SCLK {}
impl OutputFunction for U2_TXD {}
impl OutputFunction for U2_RTS {}
impl OutputFunction for U2_SCLK {}
impl OutputFunction for SPI0_SCK {}
impl OutputFunction for SPI0_MOSI {}
impl OutputFunction for SPI0_MISO {}
impl OutputFunction for SPI0_SSEL0 {}
impl OutputFunction for SPI0_SSEL1 {}
impl OutputFunction for SPI0_SSEL2 {}
impl OutputFunction for SPI0_SSEL3 {}
impl OutputFunction for SPI1_SCK {}
impl OutputFunction for SPI1_MOSI {}
impl OutputFunction for SPI1_MISO {}
impl OutputFunction for SPI1_SSEL0 {}
impl OutputFunction for SPI1_SSEL1 {}
impl OutputFunction for SCT_OUT0 {}
impl OutputFunction for SCT_OUT1 {}
impl OutputFunction for SCT_OUT2 {}
impl OutputFunction for SCT_OUT3 {}
impl OutputFunction for SCT_OUT4 {}
impl OutputFunction for SCT_OUT5 {}
impl OutputFunction for I2C1_SDA {}
impl OutputFunction for I2C1_SCL {}
impl OutputFunction for I2C2_SDA {}
impl OutputFunction for I2C2_SCL {}
impl OutputFunction for I2C3_SDA {}
impl OutputFunction for I2C3_SCL {}
impl OutputFunction for ACMP_O {}
impl OutputFunction for CLKOUT {}
impl OutputFunction for GPIO_INT_BMAT {}

// See user manual, section 31.4, table 397
impl OutputFunction for SWCLK {}
impl OutputFunction for SWDIO {}

// See user manual, section 5.4, table 20
impl OutputFunction for XTALOUT {}

// See user manual, section 15.4, table 202
impl OutputFunction for I2C0_SDA {}
impl OutputFunction for I2C0_SCL {}


/// Marker trait for input functions
///
/// This trait marks only functions that are pure input functions, which means
/// bidirectional functions are not included.
///
/// This is an internal trait. Any changes made to it won't be considered
/// breaking changes.
pub trait InputFunction {}

// Which movable functions are input functions is documented in the user manual
// in section 7.4.1, table 65.
impl InputFunction for U0_RXD {}
impl InputFunction for U0_CTS {}
impl InputFunction for U1_RXD {}
impl InputFunction for U1_CTS {}
impl InputFunction for U2_RXD {}
impl InputFunction for U2_CTS {}
impl InputFunction for SCT_PIN0 {}
impl InputFunction for SCT_PIN1 {}
impl InputFunction for SCT_PIN2 {}
impl InputFunction for SCT_PIN3 {}
impl InputFunction for ADC_PINTRIG0 {}
impl InputFunction for ADC_PINTRIG1 {}

// See user manual, section 22.4, table 294
impl InputFunction for ACMP_I1 {}
impl InputFunction for ACMP_I2 {}
impl InputFunction for ACMP_I3 {}
impl InputFunction for ACMP_I4 {}
impl InputFunction for VDDCMP {}

// See user manual, section 5.4, table 20
impl InputFunction for XTALIN {}
impl InputFunction for RESETN {}
impl InputFunction for CLKIN {}


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
