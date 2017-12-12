//! APIs for the switch matrix (SWM)
//!
//! See user manual, chapter 7.


use lpc82x;
use lpc82x::swm::pinenable0;

use ::{
    syscon,
    Pin,
};
use init_state::{
    self,
    InitState,
};


/// Interface to the switch matrix (SWM)
///
/// This API expects to be the sole owner of the SWM peripheral. Don't use
/// [`lpc82x::SWM`] directly, unless you know what you're doing.
///
/// [`lpc82x::SWM`]: ../../lpc82x/struct.SWM.html
pub struct SWM<'swm, State: InitState = init_state::Initialized> {
    swm   : &'swm lpc82x::SWM,
    _state: State,
}

impl<'swm> SWM<'swm, init_state::Unknown> {
    pub(crate) fn new(swm: &'swm lpc82x::SWM) -> Self {
        SWM {
            swm   : swm,
            _state: init_state::Unknown,
        }
    }

    /// Initialize the switch matrix
    pub fn init(mut self, syscon: &mut syscon::Api)
        -> SWM<'swm, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.swm);

        SWM {
            swm   : self.swm,
            _state: init_state::Initialized,
        }
    }
}

impl<'swm> SWM<'swm> {
    /// Assigns a movable function to a pin
    ///
    /// # Limitations
    ///
    /// This method can be used to assign movable functions to pins that are
    /// currently used for something else. The HAL user needs to make sure that
    /// this assignment doesn't conflict with any other uses of the pin.
    pub fn assign_pin<F: MovableFunction, P: Pin>(&mut self) {
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


/// Extends [`Pin`] with SWM-specific functionality
///
/// HAL users should not need to implement this trait.
pub trait PinExt {
    /// Disables the fixed function on the pin
    fn disable_fixed_functions(swm: &mut SWM);
}

macro_rules! impl_pin_ext {
    ($pin:ty $(, $fixed_function:ty)*) => {
        impl PinExt for $pin {
            fn disable_fixed_functions(_swm: &mut SWM) {
                $(
                    _swm.disable_fixed_function::<$fixed_function>();
                )*
            }
        }
    }
}

impl_pin_ext!(::PIO0_0 , ACMP_I1);
impl_pin_ext!(::PIO0_1 , ACMP_I2, CLKIN);
impl_pin_ext!(::PIO0_2 , SWDIO);
impl_pin_ext!(::PIO0_3 , SWCLK);
impl_pin_ext!(::PIO0_4 , ADC_11);
impl_pin_ext!(::PIO0_5 , RESETN);
impl_pin_ext!(::PIO0_6 , VDDCMP, ADC_1);
impl_pin_ext!(::PIO0_7 , ADC_0);
impl_pin_ext!(::PIO0_8 , XTALIN);
impl_pin_ext!(::PIO0_9 , XTALOUT);
impl_pin_ext!(::PIO0_10, I2C0_SCL);
impl_pin_ext!(::PIO0_11, I2C0_SDA);
impl_pin_ext!(::PIO0_12);
impl_pin_ext!(::PIO0_13, ADC_10);
impl_pin_ext!(::PIO0_14, ACMP_I3, ADC_2);
impl_pin_ext!(::PIO0_15);
impl_pin_ext!(::PIO0_16);
impl_pin_ext!(::PIO0_17, ADC_9);
impl_pin_ext!(::PIO0_18, ADC_8);
impl_pin_ext!(::PIO0_19, ADC_7);
impl_pin_ext!(::PIO0_20, ADC_6);
impl_pin_ext!(::PIO0_21, ADC_5);
impl_pin_ext!(::PIO0_22, ADC_4);
impl_pin_ext!(::PIO0_23, ACMP_I4, ADC_3);
impl_pin_ext!(::PIO0_24);
impl_pin_ext!(::PIO0_25);
impl_pin_ext!(::PIO0_26);
impl_pin_ext!(::PIO0_27);
impl_pin_ext!(::PIO0_28);


/// Implemented for types that represent movable functions
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
pub trait MovableFunction {
    /// Internal method to assign a pin to a movable function.
    fn assign_pin<P: Pin>(swm: &lpc82x::SWM);
}

macro_rules! impl_movable_function {
    ($movable_function:ident, $register:ident, $field:ident) => {
        /// Represents a movable function
        ///
        /// Can be used with [`SWM::assign_pin`] to assign this movable function
        /// to a pin.
        ///
        /// [`SWM::assign_pin`]: struct.SWM.html#method.assign_pin
        #[allow(non_camel_case_types)]
        pub struct $movable_function;

        impl MovableFunction for $movable_function {
            fn assign_pin<P: Pin>(swm: &lpc82x::SWM) {
                swm.$register.modify(|_, w|
                    unsafe { w.$field().bits(P::id())
                })
            }
        }
    }
}

impl_movable_function!(U0_TXD       , pinassign0 , u0_txd_o       );
impl_movable_function!(U0_RXD       , pinassign0 , u0_rxd_i       );
impl_movable_function!(U0_RTS       , pinassign0 , u0_rts_o       );
impl_movable_function!(U0_CTS       , pinassign0 , u0_cts_i       );
impl_movable_function!(U0_SCLK      , pinassign1 , u0_sclk_io     );
impl_movable_function!(U1_TXD       , pinassign1 , u1_txd_o       );
impl_movable_function!(U1_RXD       , pinassign1 , u1_rxd_i       );
impl_movable_function!(U1_RTS       , pinassign1 , u1_rts_o       );
impl_movable_function!(U1_CTS       , pinassign2 , u1_cts_i       );
impl_movable_function!(U1_SCLK      , pinassign2 , u1_sclk_io     );
impl_movable_function!(U2_TXD       , pinassign2 , u2_txd_o       );
impl_movable_function!(U2_RXD       , pinassign2 , u2_rxd_i       );
impl_movable_function!(U2_RTS       , pinassign3 , u2_rts_o       );
impl_movable_function!(U2_CTS       , pinassign3 , u2_cts_i       );
impl_movable_function!(U2_SCLK      , pinassign3 , u2_sclk_io     );
impl_movable_function!(SPI0_SCK     , pinassign3 , spi0_sck_io    );
impl_movable_function!(SPI0_MOSI    , pinassign4 , spi0_mosi_io   );
impl_movable_function!(SPI0_MISO    , pinassign4 , spi0_miso_io   );
impl_movable_function!(SPI0_SSEL0   , pinassign4 , spi0_ssel0_io  );
impl_movable_function!(SPI0_SSEL1   , pinassign4 , spi0_ssel1_io  );
impl_movable_function!(SPI0_SSEL2   , pinassign5 , spi0_ssel2_io  );
impl_movable_function!(SPI0_SSEL3   , pinassign5 , spi0_ssel3_io  );
impl_movable_function!(SPI1_SCK     , pinassign5 , spi1_sck_io    );
impl_movable_function!(SPI1_MOSI    , pinassign5 , spi1_mosi_io   );
impl_movable_function!(SPI1_MISO    , pinassign6 , spi1_miso_io   );
impl_movable_function!(SPI1_SSEL0   , pinassign6 , spi1_ssel0_io  );
impl_movable_function!(SPI1_SSEL1   , pinassign6 , spi1_ssel1_io  );
impl_movable_function!(SCT_PIN0     , pinassign6 , sct_in0_i      );
impl_movable_function!(SCT_PIN1     , pinassign7 , sct_in1_i      );
impl_movable_function!(SCT_PIN2     , pinassign7 , sct_in2_i      );
impl_movable_function!(SCT_PIN3     , pinassign7 , sct_in3_i      );
impl_movable_function!(SCT_OUT0     , pinassign7 , sct_out0_o     );
impl_movable_function!(SCT_OUT1     , pinassign8 , sct_out1_o     );
impl_movable_function!(SCT_OUT2     , pinassign8 , sct_out2_o     );
impl_movable_function!(SCT_OUT3     , pinassign8 , sct_out3_o     );
impl_movable_function!(SCT_OUT4     , pinassign8 , sct_out4_o     );
impl_movable_function!(SCT_OUT5     , pinassign9 , sct_out5_o     );
impl_movable_function!(I2C1_SDA     , pinassign9 , i2c1_sda_io    );
impl_movable_function!(I2C1_SCL     , pinassign9 , i2c1_scl_io    );
impl_movable_function!(I2C2_SDA     , pinassign9 , i2c2_sda_io    );
impl_movable_function!(I2C2_SCL     , pinassign10, i2c2_scl_io    );
impl_movable_function!(I2C3_SDA     , pinassign10, i2c3_sda_io    );
impl_movable_function!(I2C3_SCL     , pinassign10, i2c3_scl_io    );
impl_movable_function!(ADC_PINTRIG0 , pinassign10, adc_pintrig0_i );
impl_movable_function!(ADC_PINTRIG1 , pinassign11, adc_pintrig1_i );
impl_movable_function!(ACMP_O       , pinassign11, acmp_o_o       );
impl_movable_function!(CLKOUT       , pinassign11, clkout_o       );
impl_movable_function!(GPIO_INT_BMAT, pinassign11, gpio_int_bmat_o);


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
