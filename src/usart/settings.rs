use crate::pac::usart0::cfg::{
    self, CLKPOL_A, PARITYSEL_A, RXPOL_A, STOPLEN_A, TXPOL_A,
};

/// USART settings
pub struct Settings {
    pub(super) parity: PARITYSEL_A,
    pub(super) stop_len: STOPLEN_A,
    pub(super) clock_pol: CLKPOL_A,
    pub(super) rx_pol: RXPOL_A,
    pub(super) tx_pol: TXPOL_A,
}

impl Settings {
    /// Add no parity bit
    ///
    /// Overwrites the previous parity setting. This is the default.
    pub fn parity_none(mut self) -> Self {
        self.parity = PARITYSEL_A::NO_PARITY;
        self
    }

    /// Add even parity bit
    ///
    /// Overwrites the previous parity setting.
    pub fn parity_even(mut self) -> Self {
        self.parity = PARITYSEL_A::EVEN_PARITY;
        self
    }

    /// Add odd parity bit
    ///
    /// Overwrites the previous parity setting.
    pub fn parity_odd(mut self) -> Self {
        self.parity = PARITYSEL_A::ODD_PARITY;
        self
    }

    /// Add one stop bit
    ///
    /// Overwrites the previous stop length setting. This is the default.
    pub fn stop_len_1(mut self) -> Self {
        self.stop_len = STOPLEN_A::BIT_1;
        self
    }

    /// Add two stop bits
    ///
    /// Overwrites the previous stop length setting.
    pub fn stop_len_2(mut self) -> Self {
        self.stop_len = STOPLEN_A::BITS_2;
        self
    }

    /// Sample on falling clock edge
    ///
    /// This is only relevant when receiving data in synchronous mode.
    ///
    /// Overwrites the previous clock polarity setting. This is the default.
    pub fn clock_pol_falling(mut self) -> Self {
        self.clock_pol = CLKPOL_A::FALLING_EDGE;
        self
    }

    /// Sample on rising clock edge
    ///
    /// This is only relevant when receiving data in synchronous mode.
    ///
    /// Overwrites the previous clock polarity setting.
    pub fn clock_pol_rising(mut self) -> Self {
        self.clock_pol = CLKPOL_A::RISING_EDGE;
        self
    }

    /// Don't invert RX signal
    ///
    /// Overwrites the previous RX polarity setting. This is the default.
    pub fn rx_pol_standard(mut self) -> Self {
        self.rx_pol = RXPOL_A::STANDARD;
        self
    }

    /// Invert RX signal
    ///
    /// Overwrites the previous RX polarity setting.
    pub fn rx_pol_inverted(mut self) -> Self {
        self.rx_pol = RXPOL_A::INVERTED;
        self
    }

    /// Don't invert TX signal
    ///
    /// Overwrites the previous TX polarity setting. This is the default.
    pub fn tx_pol_standard(mut self) -> Self {
        self.tx_pol = TXPOL_A::STANDARD;
        self
    }

    /// Invert TX signal
    ///
    /// Overwrites the previous TX polarity setting.
    pub fn tx_pol_inverted(mut self) -> Self {
        self.tx_pol = TXPOL_A::INVERTED;
        self
    }

    pub(super) fn apply(&self, w: &mut cfg::W) {
        w.paritysel().variant(self.parity);
        w.stoplen().variant(self.stop_len);
        w.clkpol().variant(self.clock_pol);
        w.rxpol().variant(self.rx_pol);
        w.txpol().variant(self.tx_pol);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            parity: PARITYSEL_A::NO_PARITY,
            stop_len: STOPLEN_A::BIT_1,
            clock_pol: CLKPOL_A::FALLING_EDGE,
            rx_pol: RXPOL_A::STANDARD,
            tx_pol: TXPOL_A::STANDARD,
        }
    }
}
