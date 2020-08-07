use core::ops::Deref;

use crate::{
    dma,
    pac::{self, Interrupt},
    swm,
    syscon::{self, clock_source::PeripheralClockSelector},
};

/// Implemented for all USART instances
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

    /// The movable function that can be assigned to this USART's RTS pin
    type Rts;

    /// The movable function that can be assigned to this USART's CTS pin
    type Cts;

    /// The DMA channel used with this instance for receiving
    type RxChannel: dma::channels::Instance;

    /// The DMA channel used with this instance for transmitting
    type TxChannel: dma::channels::Instance;
}

macro_rules! instances {
    (
        $(
            $instance:ident,
            $clock_num:expr,
            $module:ident,
            $interrupt:ident,
            $rx:ident,
            $tx:ident,
            $rts:ident,
            $cts:ident,
            $rx_channel:ident,
            $tx_channel:ident;
        )*
    ) => {
        $(
            impl private::Sealed for pac::$instance {}

            impl Instance for pac::$instance {
                const INTERRUPT: Interrupt = Interrupt::$interrupt;
                const REGISTERS: *const pac::usart0::RegisterBlock =
                    pac::$instance::ptr();

                type Rx  = swm::$rx;
                type Tx  = swm::$tx;
                type Rts = swm::$rts;
                type Cts = swm::$cts;

                type RxChannel = dma::$rx_channel;
                type TxChannel = dma::$tx_channel;
            }

            impl PeripheralClockSelector for pac::$instance {
                const REGISTER_NUM: usize = $clock_num;
            }
        )*
    };
}

instances!(
    USART0, 0, usart0, USART0,
        U0_RXD, U0_TXD, U0_RTS, U0_CTS,
        Channel0, Channel1;
    USART1, 1, usart1, USART1,
        U1_RXD, U1_TXD, U1_RTS, U1_CTS,
        Channel2, Channel3;
    USART2, 2, usart2, USART2,
        U2_RXD, U2_TXD, U2_RTS, U2_CTS,
        Channel4, Channel5;
);

#[cfg(feature = "845")]
instances!(
    USART3, 3, usart3, PIN_INT6_USART3,
        U3_RXD, U3_TXD, NotAvailable, NotAvailable,
        Channel6, Channel7;
    USART4, 4, usart4, PIN_INT7_USART4,
        U4_RXD, U4_TXD, NotAvailable, NotAvailable,
        Channel8, Channel9;
);

mod private {
    pub trait Sealed {}
}
