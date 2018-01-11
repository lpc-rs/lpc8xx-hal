//! APIs for the switch matrix (SWM)
//!
//! See user manual, chapter 7.


use lpc82x;
use lpc82x::swm::{
    PINASSIGN0,
    PINASSIGN1,
    PINASSIGN2,
    PINASSIGN3,
    PINASSIGN4,
    PINASSIGN5,
    PINASSIGN6,
    PINASSIGN7,
    PINASSIGN8,
    PINASSIGN9,
    PINASSIGN10,
    PINASSIGN11,
    PINENABLE0,
};

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


/// Interface to the switch matrix (SWM)
///
/// This API expects to be the sole owner of the SWM peripheral. Don't use
/// [`lpc82x::SWM`] directly, unless you know what you're doing.
///
/// [`lpc82x::SWM`]: ../../lpc82x/struct.SWM.html
pub struct SWM<'swm> {
    /// Main SWM API
    pub api: Api<'swm, init_state::Unknown>,

    /// Movable functions
    pub movable_functions: MovableFunctions<'swm>,

    /// Fixed functions
    pub fixed_functions: FixedFunctions<'swm>,
}

impl<'swm> SWM<'swm> {
    pub(crate) fn new(swm: &'swm lpc82x::SWM) -> Self {
        SWM {
            api              : Api::new(swm),
            movable_functions: MovableFunctions::new(swm),
            fixed_functions  : FixedFunctions::new(swm),
        }
    }
}


/// Main API of the SWM peripheral
pub struct Api<'swm, State: InitState = init_state::Initialized> {
    swm   : &'swm lpc82x::SWM,
    _state: State,
}

impl<'swm> Api<'swm, init_state::Unknown> {
    pub(crate) fn new(swm: &'swm lpc82x::SWM) -> Self {
        Api {
            swm   : swm,
            _state: init_state::Unknown,
        }
    }

    /// Initialize the switch matrix
    pub fn init(mut self, syscon: &mut syscon::Api)
        -> Api<'swm, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.swm);

        Api {
            swm   : self.swm,
            _state: init_state::Initialized,
        }
    }
}


/// A movable function
///
/// This trait is implemented for all types that represent movable functions.
/// The user should not need to implement this trait, nor use its methods
/// directly. Any changes to this trait will not be considered breaking changes.
pub trait MovableFunction {
    /// Assigns the movable function to a pin
    ///
    /// This method is intended for internal use. Please use
    /// [`Pin::assign_function`] instead.
    ///
    /// [`Pin::assign_function`]: ../gpio/struct.Pin.html#method.assign_function
    fn assign<P: PinName>(&mut self, pin: &mut P, swm: &mut Api);

    /// Unassign the movable function
    ///
    /// This method is intended for internal use. Please use
    /// [`Pin::unassign_function`] instead.
    ///
    /// [`Pin::unassign_function`]: ../gpio/struct.Pin.html#method.unassign_function
    fn unassign<P: PinName>(&mut self, pin: &mut P, swm: &mut Api);
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
        pub struct MovableFunctions<'swm> {
            $(pub $field: $type<'swm>,)*
        }

        impl<'swm> MovableFunctions<'swm> {
            fn new(swm: &'swm lpc82x::SWM) -> Self {
                MovableFunctions {
                    $($field: $type(&swm.$reg_name),)*
                }
            }
        }


        $(
            /// Represents a movable function
            #[allow(non_camel_case_types)]
            pub struct $type<'swm>(&'swm $reg_type);

            impl<'swm> MovableFunction for $type<'swm> {
                fn assign<P: PinName>(&mut self,
                    _pin: &mut P,
                    _swm: &mut Api,
                ) {
                    // We're not using the `_swm` argument, but we require it,
                    // because the SWM needs to be clocked for this to work.

                    self.0.modify(|_, w|
                        unsafe { w.$reg_field().bits(P::ID) }
                    )
                }

                fn unassign<P: PinName>(&mut self,
                    _pin: &mut P,
                    _swm: &mut Api,
                ) {
                    // We're not using the `_swm` argument, but we require it,
                    // because the SWM needs to be clocked for this to work.

                    self.0.modify(|_, w|
                        unsafe { w.$reg_field().bits(0xff) }
                    )
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


/// A fixed function
///
/// This trait is implemented for all types that represent fixed functions. The
/// user should not need to implement this trait, nor use its methods directly.
/// Any changes to this trait will not be considered breaking changes.
pub trait FixedFunction {
    /// The pin that this fixed function can be enabled on
    type Pin: PinName;

    /// Enable the fixed function
    ///
    /// This method is intended for internal use. Please use
    /// [`Pin::enable_function`] instead.
    ///
    /// [`Pin::enable_function`]: ../gpio/struct.Pin.html#method.enable_function
    fn enable(&mut self, pin: &mut Self::Pin, swm: &mut Api);

    /// Disable the fixed function
    ///
    /// This method is intended for internal use. Please use
    /// [`Pin::disable_function`] instead.
    ///
    /// [`Pin::disable_function`]: ../gpio/struct.Pin.html#method.disable_function
    fn disable(&mut self, pin: &mut Self::Pin, swm: &mut Api);
}

macro_rules! fixed_functions {
    ($($type:ident, $field:ident, $pin:ty;)*) => {
        // Provides access to all fixed functions
        #[allow(missing_docs)]
        pub struct FixedFunctions<'swm> {
            $(pub $field: $type<'swm>,)*
        }

        impl<'swm> FixedFunctions<'swm> {
            fn new(swm: &'swm lpc82x::SWM) -> Self {
                FixedFunctions {
                    $($field: $type(&swm.pinenable0),)*
                }
            }
        }


        $(
            /// Represents a fixed function
            #[allow(non_camel_case_types)]
            pub struct $type<'swm>(&'swm PINENABLE0);

            impl<'swm> FixedFunction for $type<'swm> {
                type Pin = $pin;

                fn enable(&mut self, _pin: &mut Self::Pin, swm: &mut Api) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().clear_bit());
                }

                fn disable(&mut self, _pin: &mut Self::Pin, swm: &mut Api) {
                    swm.swm.pinenable0.modify(|_, w| w.$field().set_bit());
                }
            }
        )*
    }
}

fixed_functions!(
    ACMP_I1 , acmp_i1 , PIO0_0;
    ACMP_I2 , acmp_i2 , PIO0_1;
    ACMP_I3 , acmp_i3 , PIO0_14;
    ACMP_I4 , acmp_i4 , PIO0_23;
    SWCLK   , swclk   , PIO0_3;
    SWDIO   , swdio   , PIO0_2;
    XTALIN  , xtalin  , PIO0_8;
    XTALOUT , xtalout , PIO0_9;
    RESETN  , resetn  , PIO0_5;
    CLKIN   , clkin   , PIO0_1;
    VDDCMP  , vddcmp  , PIO0_6;
    I2C0_SDA, i2c0_sda, PIO0_11;
    I2C0_SCL, i2c0_scl, PIO0_10;
    ADC_0   , adc_0   , PIO0_7;
    ADC_1   , adc_1   , PIO0_6;
    ADC_2   , adc_2   , PIO0_14;
    ADC_3   , adc_3   , PIO0_23;
    ADC_4   , adc_4   , PIO0_22;
    ADC_5   , adc_5   , PIO0_21;
    ADC_6   , adc_6   , PIO0_20;
    ADC_7   , adc_7   , PIO0_19;
    ADC_8   , adc_8   , PIO0_18;
    ADC_9   , adc_9   , PIO0_17;
    ADC_10  , adc_10  , PIO0_13;
    ADC_11  , adc_11  , PIO0_4;
);
