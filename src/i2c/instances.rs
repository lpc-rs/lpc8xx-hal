use core::ops::Deref;

use crate::{
    pac::{self, Interrupt},
    swm,
    syscon::{self, clock_source::PeripheralClockSelector},
};

/// Internal trait for I2C peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Instance:
    Deref<Target = pac::i2c0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
{
    /// The interrupt that is triggered for this I2C peripheral
    const INTERRUPT: Interrupt;

    /// The movable function that needs to be assigned to this I2C's SDA pin
    type Sda;

    /// The movable function that needs to be assigned to this I2C's SCL pin
    type Scl;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $interrupt:ident,
            $rx:ident,
            $tx:ident;
        )*
    ) => {
        $(
            impl Instance for pac::$instance {
                const INTERRUPT: Interrupt = Interrupt::$interrupt;

                type Sda = swm::$rx;
                type Scl = swm::$tx;
            }

            impl PeripheralClockSelector for pac::$instance {
                const REGISTER_NUM: usize = $clock_num;
            }
        )*
    };
}

instances!(
    I2C0, 5, I2C0, I2C0_SDA, I2C0_SCL;
    I2C1, 6, I2C1, I2C1_SDA, I2C1_SCL;
    I2C2, 7, I2C2, I2C2_SDA, I2C2_SCL;
    I2C3, 8, I2C3, I2C3_SDA, I2C3_SCL;
);
