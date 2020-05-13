use core::ops::Deref;

use crate::{pac, swm, syscon};

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

impl Instance for pac::SPI0 {
    type Sck = swm::SPI0_SCK;
    type Mosi = swm::SPI0_MOSI;
    type Miso = swm::SPI0_MISO;
}

impl Instance for pac::SPI1 {
    type Sck = swm::SPI1_SCK;
    type Mosi = swm::SPI1_MOSI;
    type Miso = swm::SPI1_MISO;
}
