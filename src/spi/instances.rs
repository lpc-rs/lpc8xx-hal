use core::ops::Deref;

use crate::{
    pac::{self, spi0::TXCTL},
    swm,
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
pub trait SlaveSelect<I> {
    /// Select slave
    ///
    /// Intended for internal use only.
    fn select(_: &TXCTL);

    /// Deselect slave
    ///
    /// Intended for internal use only.
    fn deselect(_: &TXCTL);
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $sck:ident,
            $mosi:ident,
            $miso:ident,
            [$($ssel:ident, $ssel_reg:ident;)*];
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
                impl SlaveSelect<pac::$instance> for swm::$ssel {
                    fn select(txctl: &TXCTL) {
                        txctl.modify(|_, w| w.$ssel_reg().clear_bit());
                    }

                    fn deselect(txctl: &TXCTL) {
                        txctl.modify(|_, w| w.$ssel_reg().set_bit());
                    }
                }
            )*
        )*
    };
}

instances!(
    SPI0, 9,
        SPI0_SCK, SPI0_MOSI, SPI0_MISO,
        [
            SPI0_SSEL0, txssel0_n;
            SPI0_SSEL1, txssel1_n;
            SPI0_SSEL2, txssel2_n;
            SPI0_SSEL3, txssel3_n;
        ];
    SPI1, 10,
        SPI1_SCK, SPI1_MOSI, SPI1_MISO,
        [
            SPI1_SSEL0, txssel0_n;
            SPI1_SSEL1, txssel1_n;
        ];
);

mod private {
    pub trait Sealed {}
}
