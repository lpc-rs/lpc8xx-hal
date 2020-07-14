//! API related to master mode

use core::convert::TryFrom;

use crate::pac::{generic::Variant, i2c0::stat::MSTSTATE_A};

/// The state of an I2C instance set to master mode
#[derive(Debug, Eq, PartialEq)]
pub enum State {
    /// The peripheral is currently idle
    ///
    /// A new transaction can be started.
    Idle,

    /// Data has been received an is available to be read
    ///
    /// A read transaction has previously been initiated, and has been
    /// acknowledged by the slave.
    RxReady,

    /// Data can be transmitted
    ///
    /// A write transaction has previously been initiated, and has been
    /// acknowledged by the slave.
    TxReady,

    /// Slave has sent NACK in response to an address
    NackAddress,

    /// Slave has sent NACK in response to data
    NackData,
}

impl TryFrom<Variant<u8, MSTSTATE_A>> for State {
    /// The value of the MSTSTATE field, if unexpected
    type Error = u8;

    fn try_from(state: Variant<u8, MSTSTATE_A>) -> Result<Self, Self::Error> {
        match state {
            Variant::Val(MSTSTATE_A::IDLE) => Ok(Self::Idle),
            Variant::Val(MSTSTATE_A::RECEIVE_READY) => Ok(Self::RxReady),
            Variant::Val(MSTSTATE_A::TRANSMIT_READY) => Ok(Self::TxReady),
            Variant::Val(MSTSTATE_A::NACK_ADDRESS) => Ok(Self::NackAddress),
            Variant::Val(MSTSTATE_A::NACK_DATA) => Ok(Self::NackData),
            Variant::Res(bits) => Err(bits),
        }
    }
}
