use super::{master, Instance};

/// I2C error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Event Timeout
    ///
    /// Corresponds to the EVENTTIMEOUT flag in the STAT register.
    EventTimeout,

    /// Master Arbitration Loss
    ///
    /// Corresponds to the MSTARBLOSS flag in the STAT register.
    MasterArbitrationLoss,

    /// Master Start/Stop Error
    ///
    /// Corresponds to the MSTSTSTPERR flag in the STAT register.
    MasterStartStopError,

    /// Monitor Overflow
    ///
    /// Corresponds to the MONOV flag in the STAT register.
    MonitorOverflow,

    /// SCL Timeout
    ///
    /// Corresponds to the SCLTIMEOUT flag in the STAT register.
    SclTimeout,

    /// The I2C code encountered an unexpected hardware state
    UnexpectedState {
        /// The state that was expected
        expected: master::State,

        /// The state that was actually set
        ///
        /// The `Ok` variant represents a valid state. The `Err` variant
        /// represents an invalid bit pattern in the MSTSTATE field.
        actual: Result<master::State, u8>,
    },
}

impl Error {
    pub(super) fn read<I: Instance>(i2c: &I) -> Option<Self> {
        let stat = i2c.stat.read();

        // Check for error flags. If one is set, clear it and return the error.
        if stat.mstarbloss().bit_is_set() {
            i2c.stat.write(|w| w.mstarbloss().set_bit());
            return Some(Self::MasterArbitrationLoss);
        }
        if stat.mstststperr().bit_is_set() {
            i2c.stat.write(|w| w.mstststperr().set_bit());
            return Some(Self::MasterStartStopError);
        }
        if stat.monov().bit_is_set() {
            i2c.stat.write(|w| w.monov().set_bit());
            return Some(Self::MonitorOverflow);
        }
        if stat.eventtimeout().bit_is_set() {
            i2c.stat.write(|w| w.eventtimeout().set_bit());
            return Some(Self::EventTimeout);
        }
        if stat.scltimeout().bit_is_set() {
            i2c.stat.write(|w| w.scltimeout().set_bit());
            return Some(Self::SclTimeout);
        }

        None
    }
}
