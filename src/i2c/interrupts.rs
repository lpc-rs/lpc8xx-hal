use super::Instance;

macro_rules! interrupts {
    (
        $(
            $doc:expr,
            $field:ident,
            $enable:ident,
            $disable:ident;
        )*
    ) => {
        /// Used to enable or disable I2C interrupts
        ///
        /// See [`I2C::enable_interrupts`] or [`I2C::disable_interrupts`].
        ///
        /// [`I2C::enable_interrupts`]: struct.I2C.html#method.enable_interrupts
        /// [`I2C::disable_interrupts`]: struct.I2C.html#method.disable_interrupts
        pub struct Interrupts {
            $(
                #[doc = $doc]
                pub $field: bool,
            )*
        }

        impl Interrupts {
            pub(super) fn enable<I: Instance>(&self, i2c: &I) {
                i2c.intenset.modify(|_, w| {
                    $(
                        if self.$field {
                            w.$enable().enabled();
                        }
                    )*

                    w
                })
            }

            pub(super) fn disable<I: Instance>(&self, i2c: &I) {
                i2c.intenclr.write(|w| {
                    $(
                        if self.$field {
                            w.$disable().set_bit();
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
    "Master Pending", master_pending,
        mstpendingen, mstpendingclr;
    "Master Arbitration Loss", master_arbitration_loss,
        mstarblossen, mstarblossclr;
    "Master Start/Stop Error", master_start_stop_error,
        mstststperren, mstststperrclr;
    "Slave Pending", slave_pending,
        slvpendingen, slvpendingclr;
    "Slave Not Stretching", slave_not_stretching,
        slvnotstren, slvnotstrclr;
    "Slave Deselect", slave_deselect,
        slvdeselen, slvdeselclr;
    "Monitor Ready", monitor_ready,
        monrdyen, monrdyclr;
    "Monitor Overrun", monitor_overrun,
        monoven, monovclr;
    "Monitor Idle", monitor_idle,
        monidleen, monidleclr;
    "Event Timeout", event_timeout,
        eventtimeouten, eventtimeoutclr;
    "SCL Timeout", scl_timeout,
        scltimeouten, scltimeoutclr;
);
