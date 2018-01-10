//! APIs for the switch matrix (SWM)
//!
//! See user manual, chapter 7.


use lpc82x;
use lpc82x::swm::pinenable0;

use gpio::PinName;
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
}

impl<'swm> SWM<'swm> {
    pub(crate) fn new(swm: &'swm lpc82x::SWM) -> Self {
        SWM {
            api: Api::new(swm),
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

impl<'swm> Api<'swm> {
    /// Assigns a movable function to a pin
    ///
    /// # Limitations
    ///
    /// This method can be used to assign movable functions to pins that are
    /// currently used for something else. The HAL user needs to make sure that
    /// this assignment doesn't conflict with any other uses of the pin.
    pub fn assign_pin<F: MovableFunction, P: PinName>(&mut self) {
        F::assign_pin::<P>(&self.swm);
    }

    /// Enables a fixed function
    ///
    /// # Limitations
    ///
    /// The fixed function can be enabled on a pin that is currently used for
    /// something else. The HAL user needs to make sure that this assignment
    /// doesn't conflict with any other uses of the pin.
    pub fn enable_fixed_function<F: FixedFunction>(&mut self) {
        self.swm.pinenable0.modify(|_, w| F::enable(w));
    }

    /// Disables a fixed function
    pub fn disable_fixed_function<F: FixedFunction>(&mut self) {
        self.swm.pinenable0.modify(|_, w| F::disable(w));
    }
}


/// Implemented for types that represent movable functions
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
pub trait MovableFunction {
    /// Internal method to assign a pin to a movable function.
    fn assign_pin<P: PinName>(swm: &lpc82x::SWM);
}

macro_rules! movable_functions {
    ($($type:ident, $register:ident, $reg_field:ident;)*) => {
        $(
            /// Represents a movable function
            ///
            /// Can be used with [`SWM::assign_pin`] to assign this movable
            /// function to a pin.
            ///
            /// [`SWM::assign_pin`]: struct.SWM.html#method.assign_pin
            #[allow(non_camel_case_types)]
            pub struct $type;

            impl MovableFunction for $type {
                fn assign_pin<P: PinName>(swm: &lpc82x::SWM) {
                    swm.$register.modify(|_, w|
                        unsafe { w.$reg_field().bits(P::ID)
                    })
                }
            }
        )*
    }
}

movable_functions!(
    U0_TXD       , pinassign0 , u0_txd_o;
    U0_RXD       , pinassign0 , u0_rxd_i;
    U0_RTS       , pinassign0 , u0_rts_o;
    U0_CTS       , pinassign0 , u0_cts_i;
    U0_SCLK      , pinassign1 , u0_sclk_io;
    U1_TXD       , pinassign1 , u1_txd_o;
    U1_RXD       , pinassign1 , u1_rxd_i;
    U1_RTS       , pinassign1 , u1_rts_o;
    U1_CTS       , pinassign2 , u1_cts_i;
    U1_SCLK      , pinassign2 , u1_sclk_io;
    U2_TXD       , pinassign2 , u2_txd_o;
    U2_RXD       , pinassign2 , u2_rxd_i;
    U2_RTS       , pinassign3 , u2_rts_o;
    U2_CTS       , pinassign3 , u2_cts_i;
    U2_SCLK      , pinassign3 , u2_sclk_io;
    SPI0_SCK     , pinassign3 , spi0_sck_io;
    SPI0_MOSI    , pinassign4 , spi0_mosi_io;
    SPI0_MISO    , pinassign4 , spi0_miso_io;
    SPI0_SSEL0   , pinassign4 , spi0_ssel0_io;
    SPI0_SSEL1   , pinassign4 , spi0_ssel1_io;
    SPI0_SSEL2   , pinassign5 , spi0_ssel2_io;
    SPI0_SSEL3   , pinassign5 , spi0_ssel3_io;
    SPI1_SCK     , pinassign5 , spi1_sck_io;
    SPI1_MOSI    , pinassign5 , spi1_mosi_io;
    SPI1_MISO    , pinassign6 , spi1_miso_io;
    SPI1_SSEL0   , pinassign6 , spi1_ssel0_io;
    SPI1_SSEL1   , pinassign6 , spi1_ssel1_io;
    SCT_PIN0     , pinassign6 , sct_in0_i;
    SCT_PIN1     , pinassign7 , sct_in1_i;
    SCT_PIN2     , pinassign7 , sct_in2_i;
    SCT_PIN3     , pinassign7 , sct_in3_i;
    SCT_OUT0     , pinassign7 , sct_out0_o;
    SCT_OUT1     , pinassign8 , sct_out1_o;
    SCT_OUT2     , pinassign8 , sct_out2_o;
    SCT_OUT3     , pinassign8 , sct_out3_o;
    SCT_OUT4     , pinassign8 , sct_out4_o;
    SCT_OUT5     , pinassign9 , sct_out5_o;
    I2C1_SDA     , pinassign9 , i2c1_sda_io;
    I2C1_SCL     , pinassign9 , i2c1_scl_io;
    I2C2_SDA     , pinassign9 , i2c2_sda_io;
    I2C2_SCL     , pinassign10, i2c2_scl_io;
    I2C3_SDA     , pinassign10, i2c3_sda_io;
    I2C3_SCL     , pinassign10, i2c3_scl_io;
    ADC_PINTRIG0 , pinassign10, adc_pintrig0_i;
    ADC_PINTRIG1 , pinassign11, adc_pintrig1_i;
    ACMP_O       , pinassign11, acmp_o_o;
    CLKOUT       , pinassign11, clkout_o;
    GPIO_INT_BMAT, pinassign11, gpio_int_bmat_o;
);


/// Implemented for types that represent movable functions
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
pub trait FixedFunction {
    /// Internal method to enable a fixed function
    fn enable(w: &mut pinenable0::W) -> &mut pinenable0::W;

    /// Internal method to disable a fixed function
    fn disable(w: &mut pinenable0::W) -> &mut pinenable0::W;
}

macro_rules! impl_fixed_function {
    ($fixed_function:ident, $field:ident) => {
        /// Represents a fixed function
        ///
        /// Can be used with [`SWM::enable_fixed_function`] and
        /// [`SWM::disable_fixed_function`].
        ///
        /// [`SWM::enable_fixed_function`]: struct.SWM.html#method.enable_fixed_function
        /// [`SWM::disable_fixed_function`]: struct.SWM.html#method.disable_fixed_function
        #[allow(non_camel_case_types)]
        pub struct $fixed_function;

        impl FixedFunction for $fixed_function {
            fn enable(w: &mut pinenable0::W) -> &mut pinenable0::W {
                w.$field().clear_bit()
            }

            fn disable(w: &mut pinenable0::W) -> &mut pinenable0::W {
                w.$field().set_bit()
            }
        }
    }
}

impl_fixed_function!(ACMP_I1 , acmp_i1 );
impl_fixed_function!(ACMP_I2 , acmp_i2 );
impl_fixed_function!(ACMP_I3 , acmp_i3 );
impl_fixed_function!(ACMP_I4 , acmp_i4 );
impl_fixed_function!(SWCLK   , swclk   );
impl_fixed_function!(SWDIO   , swdio   );
impl_fixed_function!(XTALIN  , xtalin  );
impl_fixed_function!(XTALOUT , xtalout );
impl_fixed_function!(RESETN  , resetn  );
impl_fixed_function!(CLKIN   , clkin   );
impl_fixed_function!(VDDCMP  , vddcmp  );
impl_fixed_function!(I2C0_SDA, i2c0_sda);
impl_fixed_function!(I2C0_SCL, i2c0_scl);
impl_fixed_function!(ADC_0   , adc_0   );
impl_fixed_function!(ADC_1   , adc_1   );
impl_fixed_function!(ADC_2   , adc_2   );
impl_fixed_function!(ADC_3   , adc_3   );
impl_fixed_function!(ADC_4   , adc_4   );
impl_fixed_function!(ADC_5   , adc_5   );
impl_fixed_function!(ADC_6   , adc_6   );
impl_fixed_function!(ADC_7   , adc_7   );
impl_fixed_function!(ADC_8   , adc_8   );
impl_fixed_function!(ADC_9   , adc_9   );
impl_fixed_function!(ADC_10  , adc_10  );
impl_fixed_function!(ADC_11  , adc_11  );
