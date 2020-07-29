use core::ops::Deref;

use crate::{
    dma, pac, swm,
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

    /// The DMA channel used with this instance for receiving
    type RxChannel: dma::channels::Instance;

    /// The DMA channel used with this instance for transmitting
    type TxChannel: dma::channels::Instance;
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
            [$($ssel:ident),*],
            $rx_channel:ident,
            $tx_channel:ident;
        )*
    ) => {
        $(
            impl private::Sealed for pac::$instance {}

            impl Instance for pac::$instance {
                type Sck = swm::$sck;
                type Mosi = swm::$mosi;
                type Miso = swm::$miso;

                type RxChannel = dma::$rx_channel;
                type TxChannel = dma::$tx_channel;
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

#[cfg(feature = "82x")]
instances!(
    SPI0, 9,
        SPI0_SCK, SPI0_MOSI, SPI0_MISO,
        [SPI0_SSEL0, SPI0_SSEL1, SPI0_SSEL2, SPI0_SSEL3],
        Channel6, Channel7;
    SPI1, 10,
        SPI1_SCK, SPI1_MOSI, SPI1_MISO,
        [SPI1_SSEL0, SPI1_SSEL1],
        Channel8, Channel9;
);

#[cfg(feature = "845")]
instances!(
    SPI0, 9,
        SPI0_SCK, SPI0_MOSI, SPI0_MISO,
        [SPI0_SSEL0, SPI0_SSEL1, SPI0_SSEL2, SPI0_SSEL3],
        Channel10, Channel11;
    SPI1, 10,
        SPI1_SCK, SPI1_MOSI, SPI1_MISO,
        [SPI1_SSEL0, SPI1_SSEL1],
        Channel12, Channel13;
);

mod private {
    pub trait Sealed {}
}
