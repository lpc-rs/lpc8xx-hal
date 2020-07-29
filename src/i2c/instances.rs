use core::ops::Deref;

use crate::{
    dma,
    pac::{self, Interrupt},
    swm,
    syscon::{self, clock_source::PeripheralClockSelector},
};

/// Implemented for all I2C instances
pub trait Instance:
    private::Sealed
    + Deref<Target = pac::i2c0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
    + PeripheralClockSelector
{
    /// The interrupt that is triggered for this I2C peripheral
    const INTERRUPT: Interrupt;

    /// A pointer to this instance's register block
    const REGISTERS: *const pac::i2c0::RegisterBlock;

    /// The movable function that needs to be assigned to this I2C's SDA pin
    type Sda;

    /// The movable function that needs to be assigned to this I2C's SCL pin
    type Scl;

    /// The DMA channel used with this instance for slave mode
    type SlvChannel: dma::channels::Instance;

    /// The DMA channel used with this instance for master mode
    type MstChannel: dma::channels::Instance;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $interrupt:ident,
            $rx:ident,
            $tx:ident,
            $slv_channel:ident,
            $mst_channel:ident;
        )*
    ) => {
        $(
            impl private::Sealed for pac::$instance {}

            impl Instance for pac::$instance {
                const INTERRUPT: Interrupt = Interrupt::$interrupt;
                const REGISTERS: *const pac::i2c0::RegisterBlock =
                    pac::$instance::ptr();

                type Sda = swm::$rx;
                type Scl = swm::$tx;

                type SlvChannel = dma::$slv_channel;
                type MstChannel = dma::$mst_channel;
            }

            impl PeripheralClockSelector for pac::$instance {
                const REGISTER_NUM: usize = $clock_num;
            }
        )*
    };
}

#[cfg(feature = "82x")]
instances!(
    I2C0, 5, I2C0, I2C0_SDA, I2C0_SCL, Channel10, Channel11;
    I2C1, 6, I2C1, I2C1_SDA, I2C1_SCL, Channel12, Channel13;
    I2C2, 7, I2C2, I2C2_SDA, I2C2_SCL, Channel14, Channel15;
    I2C3, 8, I2C3, I2C3_SDA, I2C3_SCL, Channel16, Channel17;
);

#[cfg(feature = "845")]
instances!(
    I2C0, 5, I2C0, I2C0_SDA, I2C0_SCL, Channel14, Channel15;
    I2C1, 6, I2C1, I2C1_SDA, I2C1_SCL, Channel16, Channel17;
    I2C2, 7, I2C2, I2C2_SDA, I2C2_SCL, Channel18, Channel19;
    I2C3, 8, I2C3, I2C3_SDA, I2C3_SCL, Channel20, Channel21;
);

mod private {
    pub trait Sealed {}
}
