use crate::pins::{self, Trait as _};

use super::{
    function_kind::{Input, Output},
    functions::{Function, FunctionTrait},
    handle::Handle,
    state::Unassigned,
};

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
        /// This struct is part of [`swm::Parts`].
        ///
        /// [`swm::Parts`]: struct.Parts.html
        #[allow(missing_docs)]
        pub struct MovableFunctions {
            $(pub $field: Function<$type, Unassigned>,)*
        }

        impl MovableFunctions {
            pub(crate) fn new() -> Self {
                MovableFunctions {
                    $($field: Function::new($type(())),)*
                }
            }
        }


        $(
            /// Represents a movable function
            ///
            /// Movable functions can be accessed through [`MovableFunctions`].
            ///
            /// [`MovableFunctions`]: struct.MovableFunctions.html
            #[allow(non_camel_case_types)]
            pub struct $type(());

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
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO0_29);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO0_30);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO0_31);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_0 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_1 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_2 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_3 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_4 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_5 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_6 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_7 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_8 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_9 );
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_10);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_11);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_12);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_13);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_14);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_15);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_16);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_17);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_18);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_19);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_20);
            #[cfg(feature = "845")] impl_function!($type, $kind, $reg_name, $reg_field, PIO1_21);
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
        impl FunctionTrait<pins::$pin> for $type {
            type Kind = $kind;

            fn assign(&mut self, _pin: &mut pins::$pin, swm: &mut Handle) {
                swm.swm.$reg_name.modify(|_, w| unsafe {
                    w.$reg_field()
                        .bits(pins::$pin::ID | (pins::$pin::PORT as u8) << 5)
                });
            }

            fn unassign(&mut self, _pin: &mut pins::$pin, swm: &mut Handle) {
                swm.swm
                    .$reg_name
                    .modify(|_, w| unsafe { w.$reg_field().bits(0xff) });
            }
        }
    };
}

#[cfg(feature = "82x")]
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
    sct_pin0     , SCT_PIN0     , Input , pinassign6 , sct_pin0_i;
    sct_pin1     , SCT_PIN1     , Input , pinassign7 , sct_pin1_i;
    sct_pin2     , SCT_PIN2     , Input , pinassign7 , sct_pin2_i;
    sct_pin3     , SCT_PIN3     , Input , pinassign7 , sct_pin3_i;
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

#[cfg(feature = "845")]
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
    u3_txd       , U3_TXD       , Output, pinassign11, uart3_txd;
    u3_rxd       , U3_RXD       , Input , pinassign12, uart3_rxd;
    u3_sclk      , U3_SCLK      , Output, pinassign12, uart3_sclk;
    u4_txd       , U4_TXD       , Output, pinassign12, uart4_txd;
    u4_rxd       , U4_RXD       , Input , pinassign12, uart4_rxd;
    u4_sclk      , U4_SCLK      , Output, pinassign13, uart4_sclk;
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
    sct_pin0     , SCT_PIN0     , Input , pinassign6 , sct0_gpio_in_a_i;
    sct_pin1     , SCT_PIN1     , Input , pinassign7 , sct0_gpio_in_b_i;
    sct_pin2     , SCT_PIN2     , Input , pinassign7 , sct0_gpio_in_c_i;
    sct_pin3     , SCT_PIN3     , Input , pinassign7 , sct0_gpio_in_d_i;
    sct_out0     , SCT_OUT0     , Output, pinassign7 , sct_out0_o;
    sct_out1     , SCT_OUT1     , Output, pinassign8 , sct_out1_o;
    sct_out2     , SCT_OUT2     , Output, pinassign8 , sct_out2_o;
    sct_out3     , SCT_OUT3     , Output, pinassign8 , sct_out3_o;
    sct_out4     , SCT_OUT4     , Output, pinassign8 , sct_out4_o;
    sct_out5     , SCT_OUT5     , Output, pinassign9 , sct_out5_o;
    sct_out6     , SCT_OUT6     , Output, pinassign9 , sct_out6_o;
    i2c1_sda     , I2C1_SDA     , Output, pinassign9 , i2c1_sda_io;
    i2c1_scl     , I2C1_SCL     , Output, pinassign9 , i2c1_scl_io;
    i2c2_sda     , I2C2_SDA     , Output, pinassign10, i2c2_sda_io;
    i2c2_scl     , I2C2_SCL     , Output, pinassign10, i2c2_scl_io;
    i2c3_sda     , I2C3_SDA     , Output, pinassign10, i2c3_sda_io;
    i2c3_scl     , I2C3_SCL     , Output, pinassign10, i2c3_scl_io;
    acmp_o       , ACMP_O       , Output, pinassign11, comp0_out_o;
    clkout       , CLKOUT       , Output, pinassign11, clkout_o;
    gpio_int_bmat, GPIO_INT_BMAT, Output, pinassign11, gpio_int_bmat_o;
    t0_mat0      , T0_MAT0      , Output, pinassign13, t0_mat0;
    t0_mat1      , T0_MAT1      , Output, pinassign13, t0_mat1;
    t0_mat2      , T0_MAT2      , Output, pinassign13, t0_mat2;
    t0_mat3      , T0_MAT3      , Output, pinassign14, t0_mat3;
    t0_cap0      , T0_CAP0      , Output, pinassign14, t0_cap0;
    t0_cap1      , T0_CAP1      , Output, pinassign14, t0_cap1;
    t0_cap2      , T0_CAP2      , Output, pinassign14, t0_cap2;
);
