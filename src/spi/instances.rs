use core::ops::Deref;

use crate::{
    pac, swm,
    syscon::{self, clock_source::PeripheralClockSelector},
};

/// Internal trait for SPI peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Instance:
    Deref<Target = pac::spi0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
{
    /// The movable function that needs to be assigned to this SPI's SCK pin
    type Sck;

    /// The movable function that needs to be assigned to this SPI's MOSI pin
    type Mosi;

    /// The movable function that needs to be assigned to this SPI's MISO pin
    type Miso;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $sck:ident,
            $mosi:ident,
            $miso:ident;
        )*
    ) => {
        $(
            impl Instance for pac::$instance {
                type Sck = swm::$sck;
                type Mosi = swm::$mosi;
                type Miso = swm::$miso;
            }

            impl PeripheralClockSelector for pac::$instance {
                const REGISTER_NUM: usize = $clock_num;
            }
        )*
    };
}

instances!(
    SPI0,  9, SPI0_SCK, SPI0_MOSI, SPI0_MISO;
    SPI1, 10, SPI1_SCK, SPI1_MOSI, SPI1_MISO;
);
