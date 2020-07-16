use core::ops::Deref;

use crate::{
    pac::{self, Interrupt},
    swm,
    syscon::{self, clock_source::PeripheralClockSelector},
};

/// Internal trait for USART peripherals
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Instance:
    private::Sealed
    + Deref<Target = pac::usart0::RegisterBlock>
    + syscon::ClockControl
    + syscon::ResetControl
    + PeripheralClockSelector
{
    /// The interrupt that is triggered for this USART peripheral
    const INTERRUPT: Interrupt;

    /// A pointer to this instance's register block
    const REGISTERS: *const pac::usart0::RegisterBlock;

    /// The movable function that needs to be assigned to this USART's RX pin
    type Rx;

    /// The movable function that needs to be assigned to this USART's TX pin
    type Tx;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $module:ident,
            $interrupt:ident,
            $rx:ident,
            $tx:ident;
        )*
    ) => {
        $(
            impl private::Sealed for pac::$instance {}

            impl Instance for pac::$instance {
                const INTERRUPT: Interrupt = Interrupt::$interrupt;
                const REGISTERS: *const pac::usart0::RegisterBlock =
                    pac::$instance::ptr();

                type Rx = swm::$rx;
                type Tx = swm::$tx;
            }

            impl PeripheralClockSelector for pac::$instance {
                const REGISTER_NUM: usize = $clock_num;
            }
        )*
    };
}

instances!(
    USART0, 0, usart0, USART0, U0_RXD, U0_TXD;
    USART1, 1, usart1, USART1, U1_RXD, U1_TXD;
    USART2, 2, usart2, USART2, U2_RXD, U2_TXD;
);

#[cfg(feature = "845")]
instances!(
    USART3, 3, usart3, PIN_INT6_USART3, U3_RXD, U3_TXD;
    USART4, 4, usart4, PIN_INT7_USART4, U4_RXD, U4_TXD;
);

mod private {
    pub trait Sealed {}
}
