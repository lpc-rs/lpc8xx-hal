use super::Instance;

macro_rules! interrupts {
    (
        $(
            $doc:expr,
            $field:ident,
            $reg_field:ident;
        )*
    ) => {
        /// Used to enable or disable SPI interrupts
        ///
        /// See [`SPI::enable_interrupts`] or [`SPI::disable_interrupts`].
        ///
        /// [`SPI::enable_interrupts`]: struct.SPI.html#method.enable_interrupts
        /// [`SPI::disable_interrupts`]: struct.SPI.html#method.disable_interrupts
        pub struct Interrupts {
            $(
                #[doc = $doc]
                pub $field: bool,
            )*
        }

        impl Interrupts {
            pub(super) fn enable<I: Instance>(&self, spi: &I) {
                spi.intenset.write(|w| {
                    $(
                        if self.$field {
                            w.$reg_field().set_bit();
                        }
                    )*

                    w
                })
            }

            pub(super) fn disable<I: Instance>(&self, spi: &I) {
                spi.intenclr.write(|w| {
                    $(
                        if self.$field {
                            w.$reg_field().set_bit();
                        }
                    )*

                    w
                })
            }
        }

        impl Default for Interrupts {
            fn default() -> Self {
                Self {
                    $(
                        $field: false,
                    )*
                }
            }
        }
    };
}

interrupts!(
    "RX Ready", rx_ready, rxrdyen;
    "TX Ready", tx_ready, txrdyen;
    "Receiver Overrun", rx_overrun, rxoven;
    "Transmitter Underrun", tx_underrun, txuren;
    "Slave Select Asserted", slave_select_asserted, ssaen;
    "Slave Select Deasserted", slave_select_deasserted, ssden;
);
