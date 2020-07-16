use core::ops::Deref;

use crate::{
    pac, swm,
    syscon::{self, clock_source::PeripheralClockSelector},
};

/// Implemented for all SPI instance
pub trait Instance:
    private::Sealed
    + Deref<Target = pac::spi0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
    + PeripheralClockSelector
{
    /// The movable function that needs to be assigned to this SPI's SCK pin
    type Sck;

    /// The movable function that needs to be assigned to this SPI's MOSI pin
    type Mosi;

    /// The movable function that needs to be assigned to this SPI's MISO pin
    type Miso;
}

/// Implemented for slave select functions of a given SPI instance
pub trait SlaveSelect<I>: private::Sealed {}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $sck:ident,
            $mosi:ident,
            $miso:ident,
            [$($ssel:ident),*];
        )*
    ) => {
        $(
            impl private::Sealed for pac::$instance {}

            impl Instance for pac::$instance {
                type Sck = swm::$sck;
                type Mosi = swm::$mosi;
                type Miso = swm::$miso;
            }

            impl PeripheralClockSelector for pac::$instance {
                const REGISTER_NUM: usize = $clock_num;
            }

            $(
                impl private::Sealed for swm::$ssel {}

                impl SlaveSelect<pac::$instance> for swm::$ssel {}
            )*
        )*
    };
}

instances!(
    SPI0, 9,
        SPI0_SCK, SPI0_MOSI, SPI0_MISO,
        [SPI0_SSEL0, SPI0_SSEL1, SPI0_SSEL2, SPI0_SSEL3];
    SPI1, 10,
        SPI1_SCK, SPI1_MOSI, SPI1_MISO,
        [SPI1_SSEL0, SPI1_SSEL1];
);

mod private {
    pub trait Sealed {}
}
