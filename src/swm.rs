//! APIs for the switch matrix (SWM)
//!
//! See user manual, chapter 7.


use core::marker::PhantomData;

use lpc82x;

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
    PinName,
};
use init_state::{
    self,
    InitState,
};
use syscon;

use self::fixed_function::FixedFunction;


/// Interface to the switch matrix (SWM)
///
/// This API expects to be the sole owner of the SWM peripheral. Don't use
/// [`lpc82x::SWM`] directly, unless you know what you're doing.
///
/// [`lpc82x::SWM`]: ../../lpc82x/struct.SWM.html
pub struct SWM<'swm> {
    /// Main SWM API
    pub handle: Handle<'swm, init_state::Unknown>,

    /// Movable functions
    pub movable_functions: MovableFunctions,

    /// Fixed functions
    pub fixed_functions: FixedFunctions,
}

impl<'swm> SWM<'swm> {
    pub(crate) fn new(swm: &'swm lpc82x::SWM) -> Self {
        SWM {
            handle           : Handle::new(swm),
            movable_functions: MovableFunctions::new(),
            fixed_functions  : FixedFunctions::new(),
        }
    }
}


/// Main API of the SWM peripheral
pub struct Handle<'swm, State: InitState = init_state::Initialized> {
    swm   : &'swm lpc82x::SWM,
    _state: State,
}

impl<'swm> Handle<'swm, init_state::Unknown> {
    pub(crate) fn new(swm: &'swm lpc82x::SWM) -> Self {
        Handle {
            swm   : swm,
            _state: init_state::Unknown,
        }
    }

    /// Initialize the switch matrix
    pub fn init(mut self, syscon: &mut syscon::Handle)
        -> Handle<'swm, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.swm);

        Handle {
            swm   : self.swm,
            _state: init_state::Initialized,
        }
    }
}


/// Traits implemented by movable functions
///
/// These traits are implemented for all types that represent movable functions.
/// The user should not need to implement these traits, nor use their methods
/// directly. Changes made to this module will not be considered breaking
/// changes.
pub mod movable_function {
    use gpio::PinName;
    use swm;


    /// Internal trait for unassigned movable functions that can be assigned
    pub trait Assign<P> where P: PinName {
        /// The type that is returned by [`assign`].
        ///
        /// Typically, this will be the same type that implements this trait,
        /// but with a type parameter changed to indicate that the function has
        /// been assigned to a pin.
        ///
        /// [`assign`]: #tymethod.assign
        type Assigned;

        /// Assigns the movable function to a pin
        ///
        /// This method is intended for internal use. Please use
        /// [`Pin::assign_function`] instead.
        ///
        /// [`Pin::assign_function`]: ../gpio/struct.Pin.html#method.assign_function
        fn assign(self, pin: &mut P, swm: &mut swm::Handle) -> Self::Assigned;
    }


    /// Internal trait for assigned movable functions that can be unassigned
    pub trait Unassign<P> where P: PinName {
        /// The type that is returned by [`unassign`].
        ///
        /// Typically, this will be the same type that implements this trait,
        /// but with a type parameter changed to indicate that the function is
        /// no longer assigned to a pin.
        ///
        /// [`unassign`]: #tymethod.unassign
        type Unassigned;

        /// Unassign the movable function
        ///
        /// This method is intended for internal use. Please use
        /// [`Pin::unassign_function`] instead.
        ///
        /// [`Pin::unassign_function`]: ../gpio/struct.Pin.html#method.unassign_function
        fn unassign(self, pin: &mut P, swm: &mut swm::Handle)
            -> Self::Unassigned;
    }


    /// Contains types that indicate the state of a movable function
    pub mod state {
        use core::marker::PhantomData;


        /// Implemented by types that indicate the state of a movable function
        ///
        /// This trait is implemented by types that indicate the state of a
        /// movable function. It exists only to document which types those are.
        /// The user should not need to implement this trait, nor use it
        /// directly.
        pub trait State {}


        /// Indicates that the current state of the movable function is unknown
        ///
        /// This is the case after the HAL is initialized, as we can't know what
        /// happened before that.
        pub struct Unknown;

        impl State for Unknown {}


        /// Indicates that the movable function is unassigned
        pub struct Unassigned;

        impl State for Unassigned {}


        /// Indicates that the movable function is assigned to a pin
        pub struct Assigned<Pin>(PhantomData<Pin>);

        impl<Pin> State for Assigned<Pin> {}
    }
}

macro_rules! movable_functions {
    (
        $(
            $field:ident,
            $type:ident,
            $reg_type:ident,
            $reg_name:ident,
            $reg_field:ident;
        )*
    ) => {
        /// Provides access to all movable functions
        #[allow(missing_docs)]
        pub struct MovableFunctions {
            $(pub $field: $type<movable_function::state::Unknown>,)*
        }

        impl MovableFunctions {
            fn new() -> Self {
                MovableFunctions {
                    $($field: $type(PhantomData),)*
                }
            }
        }


        $(
            /// Represents a movable function
            #[allow(non_camel_case_types)]
            pub struct $type<State>(PhantomData<State>)
                where State: movable_function::state::State;

            impl $type<movable_function::state::Unknown> {
                /// Affirm that the movable function is in its default state
                ///
                /// By calling this method, the user promises that the movable
                /// function is in its default state. This is safe to do, if
                /// nothing has changed that state before the HAL has been
                /// initialized.
                ///
                /// If the movable function's state has been changed by any
                /// other means than the HAL API, then the user must use those
                /// means to return the movable function to its default state,
                /// as specified in the user manual, before calling this method.
                pub unsafe fn affirm_default_state(self)
                    -> $type<movable_function::state::Unassigned>
                {
                    $type(PhantomData)
                }

            }

            impl<P> movable_function::Assign<P>
                for $type<movable_function::state::Unassigned>
                where P: PinName
            {
                type Assigned = $type<movable_function::state::Assigned<P>>;

                fn assign(self,
                    _pin: &mut P,
                    swm : &mut Handle,
                )
                    -> Self::Assigned
                {
                    swm.swm.$reg_name.modify(|_, w|
                        unsafe { w.$reg_field().bits(P::ID) }
                    );
                    $type(PhantomData)
                }
            }

            impl<P> movable_function::Unassign<P>
                for $type<movable_function::state::Assigned<P>>
                where P: PinName
            {
                type Unassigned = $type<movable_function::state::Unassigned>;

                fn unassign(self,
                    _pin: &mut P,
                    swm : &mut Handle,
                )
                    -> Self::Unassigned
                {
                    swm.swm.$reg_name.modify(|_, w|
                        unsafe { w.$reg_field().bits(0xff) }
                    );
                    $type(PhantomData)
                }
            }
        )*
    }
}

movable_functions!(
    u0_txd       , U0_TXD       , PINASSIGN0 , pinassign0 , u0_txd_o;
    u0_rxd       , U0_RXD       , PINASSIGN0 , pinassign0 , u0_rxd_i;
    u0_rts       , U0_RTS       , PINASSIGN0 , pinassign0 , u0_rts_o;
    u0_cts       , U0_CTS       , PINASSIGN0 , pinassign0 , u0_cts_i;
    u0_sclk      , U0_SCLK      , PINASSIGN1 , pinassign1 , u0_sclk_io;
    u1_txd       , U1_TXD       , PINASSIGN1 , pinassign1 , u1_txd_o;
    u1_rxd       , U1_RXD       , PINASSIGN1 , pinassign1 , u1_rxd_i;
    u1_rts       , U1_RTS       , PINASSIGN1 , pinassign1 , u1_rts_o;
    u1_cts       , U1_CTS       , PINASSIGN2 , pinassign2 , u1_cts_i;
    u1_sclk      , U1_SCLK      , PINASSIGN2 , pinassign2 , u1_sclk_io;
    u2_txd       , U2_TXD       , PINASSIGN2 , pinassign2 , u2_txd_o;
    u2_rxd       , U2_RXD       , PINASSIGN2 , pinassign2 , u2_rxd_i;
    u2_rts       , U2_RTS       , PINASSIGN3 , pinassign3 , u2_rts_o;
    u2_cts       , U2_CTS       , PINASSIGN3 , pinassign3 , u2_cts_i;
    u2_sclk      , U2_SCLK      , PINASSIGN3 , pinassign3 , u2_sclk_io;
    spi0_sck     , SPI0_SCK     , PINASSIGN3 , pinassign3 , spi0_sck_io;
    spi0_mosi    , SPI0_MOSI    , PINASSIGN4 , pinassign4 , spi0_mosi_io;
    spi0_miso    , SPI0_MISO    , PINASSIGN4 , pinassign4 , spi0_miso_io;
    spi0_ssel0   , SPI0_SSEL0   , PINASSIGN4 , pinassign4 , spi0_ssel0_io;
    spi0_ssek1   , SPI0_SSEL1   , PINASSIGN4 , pinassign4 , spi0_ssel1_io;
    spi0_ssel2   , SPI0_SSEL2   , PINASSIGN5 , pinassign5 , spi0_ssel2_io;
    spi0_ssel3   , SPI0_SSEL3   , PINASSIGN5 , pinassign5 , spi0_ssel3_io;
    spi1_sck     , SPI1_SCK     , PINASSIGN5 , pinassign5 , spi1_sck_io;
    spi1_mosi    , SPI1_MOSI    , PINASSIGN5 , pinassign5 , spi1_mosi_io;
    spi1_miso    , SPI1_MISO    , PINASSIGN6 , pinassign6 , spi1_miso_io;
    spi1_ssel0   , SPI1_SSEL0   , PINASSIGN6 , pinassign6 , spi1_ssel0_io;
    spi1_ssel1   , SPI1_SSEL1   , PINASSIGN6 , pinassign6 , spi1_ssel1_io;
    sct_pin0     , SCT_PIN0     , PINASSIGN6 , pinassign6 , sct_in0_i;
    sct_pin1     , SCT_PIN1     , PINASSIGN7 , pinassign7 , sct_in1_i;
    sct_pin2     , SCT_PIN2     , PINASSIGN7 , pinassign7 , sct_in2_i;
    sct_pin3     , SCT_PIN3     , PINASSIGN7 , pinassign7 , sct_in3_i;
    sct_out0     , SCT_OUT0     , PINASSIGN7 , pinassign7 , sct_out0_o;
    sct_out1     , SCT_OUT1     , PINASSIGN8 , pinassign8 , sct_out1_o;
    sct_out2     , SCT_OUT2     , PINASSIGN8 , pinassign8 , sct_out2_o;
    sct_out3     , SCT_OUT3     , PINASSIGN8 , pinassign8 , sct_out3_o;
    sct_out4     , SCT_OUT4     , PINASSIGN8 , pinassign8 , sct_out4_o;
    sct_out5     , SCT_OUT5     , PINASSIGN9 , pinassign9 , sct_out5_o;
    i2c1_sda     , I2C1_SDA     , PINASSIGN9 , pinassign9 , i2c1_sda_io;
    i2c1_scl     , I2C1_SCL     , PINASSIGN9 , pinassign9 , i2c1_scl_io;
    i2c2_sda     , I2C2_SDA     , PINASSIGN9 , pinassign9 , i2c2_sda_io;
    i2c2_scl     , I2C2_SCL     , PINASSIGN10, pinassign10, i2c2_scl_io;
    i2c3_sda     , I2C3_SDA     , PINASSIGN10, pinassign10, i2c3_sda_io;
    i2c3_scl     , I2C3_SCL     , PINASSIGN10, pinassign10, i2c3_scl_io;
    adc_pintrig0 , ADC_PINTRIG0 , PINASSIGN10, pinassign10, adc_pintrig0_i;
    acd_pintrig1 , ADC_PINTRIG1 , PINASSIGN11, pinassign11, adc_pintrig1_i;
    acmp_o       , ACMP_O       , PINASSIGN11, pinassign11, acmp_o_o;
    clkout       , CLKOUT       , PINASSIGN11, pinassign11, clkout_o;
    gpio_int_bmat, GPIO_INT_BMAT, PINASSIGN11, pinassign11, gpio_int_bmat_o;
);


/// Traits implemented by fixed functions
///
/// These traits are implemented for all types that represent fixed functions.
/// The user should not need to implement these traits, nor use their methods
/// directly. Changes made to this module will not be considered breaking
/// changes.
pub mod fixed_function {
    use gpio::PinName;
    use swm;

    use self::state::State;


    /// A fixed function
    ///
    /// This trait is implemented for all types that represent fixed functions.
    /// The user should not need to implement this trait, nor use it directly.
    /// Any changes to this trait will not be considered breaking changes.
    pub trait FixedFunction {
        /// The pin that this fixed function can be enabled on
        type Pin: PinName;

        /// The default state of this function
        type DefaultState: State;
    }

    /// Internal trait for disabled fixed functions that can be enabled
    pub trait Enable: FixedFunction {
        /// The type that is returned by [`enable`].
        ///
        /// Typically, this will be the same type that implements this trait,
        /// but with a type parameter changed to indicate that the function has
        /// been enabled.
        ///
        /// [`enable`]: #tymethod.enable
        type Enabled;

        /// Enable the fixed function
        ///
        /// This method is intended for internal use. Please use
        /// [`Pin::enable_function`] instead.
        ///
        /// [`Pin::enable_function`]: ../gpio/struct.Pin.html#method.enable_function
        fn enable(self, pin: &mut Self::Pin, swm: &mut swm::Handle)
            -> Self::Enabled;
    }

    /// Internal trait for enabled fixed functions that can be disabled
    pub trait Disable: FixedFunction {
        /// The type that is returned by [`disable`].
        ///
        /// Typically, this will be the same type that implements this trait,
        /// but with a type parameter changed to indicate that the function has
        /// been enabled.
        ///
        /// [`disable`]: #tymethod.disable
        type Disabled;

        /// Disable the fixed function
        ///
        /// This method is intended for internal use. Please use
        /// [`Pin::disable_function`] instead.
        ///
        /// [`Pin::disable_function`]: ../gpio/struct.Pin.html#method.disable_function
        fn disable(self, pin: &mut Self::Pin, swm: &mut swm::Handle)
            -> Self::Disabled;
    }


    /// Contains types that indicate the state of a fixed function
    pub mod state {
        /// Implemented by types that indicate the state of a fixed function
        ///
        /// This trait is implemented by types that indicate the state of a
        /// fixed function. It exists only to document which types those are.
        /// The user should not need to implement this trait, nor use it
        /// directly.
        pub trait State {}

        /// Indicates that the current state of the fixed function is unknown
        ///
        /// This is the case after the HAL is initialized, as we can't know what
        /// happened before that.
        pub struct Unknown;
        impl State for Unknown {}

        /// Indicates that the fixed function is disabled
        pub struct Disabled;
        impl State for Disabled {}

        /// Indicates that the fixed function is enabled
        pub struct Enabled;
        impl State for Enabled {}
    }
}

macro_rules! fixed_functions {
    ($($type:ident, $field:ident, $pin:ty, $default_state:ident;)*) => {
        /// Provides access to all fixed functions
        #[allow(missing_docs)]
        pub struct FixedFunctions {
            $(pub $field: $type<fixed_function::state::Unknown>,)*
        }

        impl FixedFunctions {
            fn new() -> Self {
                FixedFunctions {
                    $($field: $type(PhantomData),)*
                }
            }
        }


        $(
            /// Represents a fixed function
            #[allow(non_camel_case_types)]
            pub struct $type<State>(PhantomData<State>)
                where State: fixed_function::state::State;

            impl $type<fixed_function::state::Unknown> {
                /// Affirm that the fixed function is in its default state
                ///
                /// By calling this method, the user promises that the fixed
                /// function is in its default state. This is safe to do, if
                /// nothing has changed that state before the HAL has been
                /// initialized.
                ///
                /// If the fixed function's state has been changed by any other
                /// means than the HAL API, then the user must use those means
                /// to return the fixed function to its default state, as
                /// specified in the user manual, before calling this method.
                pub unsafe fn affirm_default_state(self)
                    -> $type<<Self as FixedFunction>::DefaultState>
                {
                    $type(PhantomData)
                }

            }

            impl<State> FixedFunction for $type<State>
                where State: fixed_function::state::State
            {
                type Pin = $pin;

                type DefaultState = fixed_function::state::$default_state;
            }

            impl fixed_function::Enable for
                $type<fixed_function::state::Disabled>
            {
                type Enabled = $type<fixed_function::state::Enabled>;

                fn enable(self, _pin: &mut Self::Pin, swm: &mut Handle)
                    -> Self::Enabled
                {
                    swm.swm.pinenable0.modify(|_, w| w.$field().clear_bit());
                    $type(PhantomData)
                }
            }

            impl fixed_function::Disable for
                $type<fixed_function::state::Enabled>
            {
                type Disabled = $type<fixed_function::state::Disabled>;

                fn disable(self, _pin: &mut Self::Pin, swm: &mut Handle)
                    -> Self::Disabled
                {
                    swm.swm.pinenable0.modify(|_, w| w.$field().set_bit());
                    $type(PhantomData)
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


/// Marker trait to mark all ADC functions
pub trait AdcFunction {}

impl<State> AdcFunction for ADC_0<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_1<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_2<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_3<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_4<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_5<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_6<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_7<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_8<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_9<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_10<State>
    where State: fixed_function::state::State {}
impl<State> AdcFunction for ADC_11<State>
    where State: fixed_function::state::State {}


/// Marker trait for output functions
///
/// This trait marks all functions that include output, which means
/// bidirectional functions are also included.
pub trait OutputFunction {}

// Which movable functions are output functions is documented in the user manual
// in section 7.4.1, table 65.
impl<State> OutputFunction for U0_TXD<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U0_RTS<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U0_SCLK<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U1_TXD<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U1_RTS<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U1_SCLK<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U2_TXD<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U2_RTS<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for U2_SCLK<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_SCK<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_MOSI<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_MISO<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_SSEL0<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_SSEL1<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_SSEL2<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI0_SSEL3<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI1_SCK<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI1_MOSI<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI1_MISO<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI1_SSEL0<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SPI1_SSEL1<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SCT_OUT0<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SCT_OUT1<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SCT_OUT2<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SCT_OUT3<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SCT_OUT4<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for SCT_OUT5<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for I2C1_SDA<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for I2C1_SCL<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for I2C2_SDA<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for I2C2_SCL<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for I2C3_SDA<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for I2C3_SCL<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for ACMP_O<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for CLKOUT<State>
    where State: movable_function::state::State {}
impl<State> OutputFunction for GPIO_INT_BMAT<State>
    where State: movable_function::state::State {}

// See user manual, section 31.4, table 397
impl<State> OutputFunction for SWCLK<State>
    where State: fixed_function::state::State {}
impl<State> OutputFunction for SWDIO<State>
    where State: fixed_function::state::State {}

// See user manual, section 5.4, table 20
impl<State> OutputFunction for XTALOUT<State>
    where State: fixed_function::state::State {}

// See user manual, section 15.4, table 202
impl<State> OutputFunction for I2C0_SDA<State>
    where State: fixed_function::state::State {}
impl<State> OutputFunction for I2C0_SCL<State>
    where State: fixed_function::state::State {}


/// Marker trait for input functions
///
/// This trait marks only functions that are pure input functions, which means
/// bidirectional functions are not included.
pub trait InputFunction {}

// Which movable functions are input functions is documented in the user manual
// in section 7.4.1, table 65.
impl<State> InputFunction for U0_RXD<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for U0_CTS<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for U1_RXD<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for U1_CTS<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for U2_RXD<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for U2_CTS<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for SCT_PIN0<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for SCT_PIN1<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for SCT_PIN2<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for SCT_PIN3<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for ADC_PINTRIG0<State>
    where State: movable_function::state::State {}
impl<State> InputFunction for ADC_PINTRIG1<State>
    where State: movable_function::state::State {}

// See user manual, section 22.4, table 294
impl<State> InputFunction for ACMP_I1<State>
    where State: fixed_function::state::State {}
impl<State> InputFunction for ACMP_I2<State>
    where State: fixed_function::state::State {}
impl<State> InputFunction for ACMP_I3<State>
    where State: fixed_function::state::State {}
impl<State> InputFunction for ACMP_I4<State>
    where State: fixed_function::state::State {}
impl<State> InputFunction for VDDCMP<State>
    where State: fixed_function::state::State {}

// See user manual, section 5.4, table 20
impl<State> InputFunction for XTALIN<State>
    where State: fixed_function::state::State {}
impl<State> InputFunction for RESETN<State>
    where State: fixed_function::state::State {}
impl<State> InputFunction for CLKIN<State>
    where State: fixed_function::state::State {}
