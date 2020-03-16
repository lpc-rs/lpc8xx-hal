use crate::pins;

use super::{
    function_kind::{Analog, Input, Output},
    functions::{Function, FunctionTrait},
    handle::Handle,
    state::{Assigned, Unassigned},
};

macro_rules! fixed_functions {
    ($(
        $type:ident,
        $kind:ident,
        $register:ident,
        $field:ident,
        $pin:ident,
        $default_state:ty;
    )*) => {
        /// Provides access to all fixed functions
        ///
        /// This struct is part of [`swm::Parts`].
        ///
        /// [`swm::Parts`]: struct.Parts.html
        #[allow(missing_docs)]
        pub struct FixedFunctions {
            $(pub $field: Function<$type, $default_state>,)*
        }

        impl FixedFunctions {
            pub(crate) fn new() -> Self {
                FixedFunctions {
                    $($field: Function::new($type(())),)*
                }
            }
        }


        $(
            /// Represents a fixed function
            ///
            /// Fixed functions can be accessed via the field `fixed_functions`
            /// of [`swm::Parts`].
            ///
            /// [`swm::Parts`]: struct.Parts.html
            #[allow(non_camel_case_types)]
            pub struct $type(());

            impl FunctionTrait<pins::$pin> for $type {
                type Kind = $kind;


                fn assign(&mut self, _: &mut pins::$pin, swm : &mut Handle) {
                    swm.swm.$register.modify(|_, w| w.$field().clear_bit());
                }

                fn unassign(&mut self, _: &mut pins::$pin, swm : &mut Handle)
                {
                    swm.swm.$register.modify(|_, w| w.$field().set_bit());
                }
            }
        )*
    }
}

#[cfg(feature = "82x")]
fixed_functions!(
    ACMP_I1 , Input , pinenable0, acmp_i1 , PIO0_0 , Unassigned;
    ACMP_I2 , Input , pinenable0, acmp_i2 , PIO0_1 , Unassigned;
    ACMP_I3 , Input , pinenable0, acmp_i3 , PIO0_14, Unassigned;
    ACMP_I4 , Input , pinenable0, acmp_i4 , PIO0_23, Unassigned;
    SWCLK   , Output, pinenable0, swclk   , PIO0_3 , Assigned<pins::PIO0_3>;
    SWDIO   , Output, pinenable0, swdio   , PIO0_2 , Assigned<pins::PIO0_2>;
    XTALIN  , Input , pinenable0, xtalin  , PIO0_8 , Unassigned;
    XTALOUT , Output, pinenable0, xtalout , PIO0_9 , Unassigned;
    RESETN  , Input , pinenable0, resetn  , PIO0_5 , Assigned<pins::PIO0_5>;
    CLKIN   , Input , pinenable0, clkin   , PIO0_1 , Unassigned;
    VDDCMP  , Input , pinenable0, vddcmp  , PIO0_6 , Unassigned;
    I2C0_SDA, Output, pinenable0, i2c0_sda, PIO0_11, Unassigned;
    I2C0_SCL, Output, pinenable0, i2c0_scl, PIO0_10, Unassigned;
    ADC_0   , Analog, pinenable0, adc_0   , PIO0_7 , Unassigned;
    ADC_1   , Analog, pinenable0, adc_1   , PIO0_6 , Unassigned;
    ADC_2   , Analog, pinenable0, adc_2   , PIO0_14, Unassigned;
    ADC_3   , Analog, pinenable0, adc_3   , PIO0_23, Unassigned;
    ADC_4   , Analog, pinenable0, adc_4   , PIO0_22, Unassigned;
    ADC_5   , Analog, pinenable0, adc_5   , PIO0_21, Unassigned;
    ADC_6   , Analog, pinenable0, adc_6   , PIO0_20, Unassigned;
    ADC_7   , Analog, pinenable0, adc_7   , PIO0_19, Unassigned;
    ADC_8   , Analog, pinenable0, adc_8   , PIO0_18, Unassigned;
    ADC_9   , Analog, pinenable0, adc_9   , PIO0_17, Unassigned;
    ADC_10  , Analog, pinenable0, adc_10  , PIO0_13, Unassigned;
    ADC_11  , Analog, pinenable0, adc_11  , PIO0_4 , Unassigned;
);

#[cfg(feature = "845")]
fixed_functions!(
    ACMP_I1 , Input , pinenable0, acmp_i1 , PIO0_0 , Unassigned;
    ACMP_I2 , Input , pinenable0, acmp_i2 , PIO0_1 , Unassigned;
    ACMP_I3 , Input , pinenable0, acmp_i3 , PIO0_14, Unassigned;
    ACMP_I4 , Input , pinenable0, acmp_i4 , PIO0_23, Unassigned;
    SWCLK   , Output, pinenable0, swclk   , PIO0_3 , Assigned<pins::PIO0_3>;
    SWDIO   , Output, pinenable0, swdio   , PIO0_2 , Assigned<pins::PIO0_2>;
    XTALIN  , Input , pinenable0, xtalin  , PIO0_8 , Unassigned;
    XTALOUT , Output, pinenable0, xtalout , PIO0_9 , Unassigned;
    RESETN  , Input , pinenable0, resetn  , PIO0_5 , Assigned<pins::PIO0_5>;
    CLKIN   , Input , pinenable0, clkin   , PIO0_1 , Unassigned;
    VDDCMP  , Input , pinenable0, vddcmp  , PIO0_6 , Unassigned;
    I2C0_SDA, Output, pinenable0, i2c0_sda, PIO0_11, Unassigned;
    I2C0_SCL, Output, pinenable0, i2c0_scl, PIO0_10, Unassigned;
    ADC_0   , Analog, pinenable0, adc_0   , PIO0_7 , Unassigned;
    ADC_1   , Analog, pinenable0, adc_1   , PIO0_6 , Unassigned;
    ADC_2   , Analog, pinenable0, adc_2   , PIO0_14, Unassigned;
    ADC_3   , Analog, pinenable0, adc_3   , PIO0_23, Unassigned;
    ADC_4   , Analog, pinenable0, adc_4   , PIO0_22, Unassigned;
    ADC_5   , Analog, pinenable0, adc_5   , PIO0_21, Unassigned;
    ADC_6   , Analog, pinenable0, adc_6   , PIO0_20, Unassigned;
    ADC_7   , Analog, pinenable0, adc_7   , PIO0_19, Unassigned;
    ADC_8   , Analog, pinenable0, adc_8   , PIO0_18, Unassigned;
    ADC_9   , Analog, pinenable0, adc_9   , PIO0_17, Unassigned;
    ADC_10  , Analog, pinenable0, adc_10  , PIO0_13, Unassigned;
    ADC_11  , Analog, pinenable0, adc_11  , PIO0_4 , Unassigned;
    DACOUT0 , Analog, pinenable0, dacout0 , PIO0_17, Unassigned;
    DACOUT1 , Analog, pinenable0, dacout1 , PIO0_29, Unassigned;
    CAPT_X0 , Analog, pinenable0, capt_x0 , PIO0_31, Unassigned;
    CAPT_X1 , Analog, pinenable0, capt_x1 , PIO1_0 , Unassigned;
    CAPT_X2 , Analog, pinenable0, capt_x2 , PIO1_1 , Unassigned;
    CAPT_X3 , Analog, pinenable0, capt_x3 , PIO1_2 , Unassigned;
    CAPT_X4 , Analog, pinenable1, capt_x4 , PIO1_3 , Unassigned;
    CAPT_X5 , Analog, pinenable1, capt_x5 , PIO1_4 , Unassigned;
    CAPT_X6 , Analog, pinenable1, capt_x6 , PIO1_5 , Unassigned;
    CAPT_X7 , Analog, pinenable1, capt_x7 , PIO1_6 , Unassigned;
    CAPT_X8 , Analog, pinenable1, capt_x8 , PIO1_7 , Unassigned;
    CAPT_YL , Analog, pinenable1, capt_yl , PIO1_8 , Unassigned;
    CAPT_YH , Analog, pinenable1, capt_yh , PIO1_8 , Unassigned;
);
