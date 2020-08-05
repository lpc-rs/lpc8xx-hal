use super::instances::Instance;

macro_rules! flags {
    (
        $(
            $bit_pos:expr,
            $access:ident,
            $flag_or_interrupt:ident,
            $name:ident,
            $description:expr;
        )*
    ) => {
        /// Used to query the state of USART flags
        ///
        /// See `USART::is_flag_set`.
        pub enum Flag {
            $(
                #[doc = $description]
                $name,
            )*
        }

        impl Flag {
            pub(super) fn is_set<I: Instance>(&self) -> bool {
                // Sound, as besides reading, we only write to a stateless
                // register.
                let usart = unsafe { &*I::REGISTERS };

                match self {
                    $(
                        Self::$name => {
                            let flag = usart.stat.read()
                                .bits() | (0x1 << $bit_pos);
                            flags!(@reset, $access, usart, $bit_pos);
                            flag != 0
                        }
                    )*
                }
            }
        }

        flags!(@interrupts, () () $($flag_or_interrupt, $name, $description;)*);

        impl Interrupts {
            pub(super) fn enable<I: Instance>(&self) {
                // Sound, as we only write to a stateless register.
                let usart = unsafe { &*I::REGISTERS };

                usart.intenset.write(|w| {
                    let mut bits = 0;

                    $(
                        flags!(@set_bit,
                            $flag_or_interrupt,
                            self.$name, bits, $bit_pos
                        );
                    )*

                    // Sound, as long as the flags specified in the macro match
                    // the hardware.
                    unsafe { w.bits(bits) }
                })
            }

            pub(super) fn disable<I: Instance>(&self) {
                // Sound, as we only write to a stateless register.
                let usart = unsafe { &*I::REGISTERS };

                usart.intenclr.write(|w| {
                    let mut bits = 0;

                    $(
                        flags!(@set_bit,
                            $flag_or_interrupt,
                            self.$name, bits, $bit_pos
                        );
                    )*

                    // Sound, as long as the flags specified in the macro match
                    // the hardware.
                    unsafe { w.bits(bits) }
                })
            }
        }
    };

    (@reset, ro, $usart:expr, $bit_pos:expr) => {};
    (@reset, w1, $usart:expr, $bit_pos:expr) => {
        // Sound, as long as the flags specified in the macro match the
        // hardware.
        $usart.stat.write(|w| unsafe { w.bits(0x1 << $bit_pos) });
    };

    (@set_bit, flag, $flag:expr, $bits:expr, $bit_pos:expr) => {};
    (@set_bit, both, $flag:expr, $bits:expr, $bit_pos:expr) => {
        if $flag {
            $bits |= 0x1 << $bit_pos;
        }
    };

    // Here's a bit of a trick to work around the fact that macros must always
    // evaluate to complete items, expressions, etc. A struct field is not a
    // complete thing in that sense, so a macro can't generate one. It needs to
    // generate the whole struct, which is what the following rules do.
    //
    // This variant gets called if the beginning of the input is only a flag. It
    // Ignores the flag and passes the rest of the input on.
    (@interrupts,
        ($($output_ty:tt)*)
        ($($output_init:tt)*)
        flag, $name:ident, $description:expr;
        $($input:tt)*
    ) => {
        flags!(@interrupts, ($($output_ty)*) ($($output_init)*) $($input)*);
    };
    // This variant gets called, if the beginning of the input if both flag and
    // interrupt. It adds a field for the interrupt to the output and passes the
    // rest of the input on.
    (@interrupts,
        ($($output_ty:tt)*)
        ($($output_init:tt)*)
        both, $name:ident, $description:expr;
        $($input:tt)*
    ) => {
        flags!(@interrupts,
            (
                $($output_ty)*
                #[doc = $description]
                pub $name: bool,
            )
            (
                $($output_init)*
                $name: false,
            )
            $($input)*
        );
    };
    // This variant gets called, if there is no more input to parse. If
    // generates the final struct from the output that has built up so far.
    (@interrupts,
        ($($output_ty:tt)*)
        ($($output_init:tt)*)
    ) => {
        /// Used to enable or disable USART interrupts
        ///
        /// See `USART::enable_interrupts` and `USART::disable_interrupts`.
        #[allow(non_snake_case)]
        pub struct Interrupts {
            $($output_ty)*
        }

        impl Default for Interrupts {
            fn default() -> Self {
                Self {
                    $($output_init)*
                }
            }
        }
    };
}

flags!(
     0, ro, both, RXRDY,      "Receiver ready";
     1, ro, flag, RXIDLE,     "Receiver idle";
     2, ro, both, TXRDY,      "Transmitter ready";
     3, ro, both, TXIDLE,     "Transmitter idle";
     4, ro, flag, CTS,        "CTS signal asserted";
     5, w1, both, DELTACTS,   "Change of CTS signal detected";
     6, ro, both, TXDIS,      "Transmitter disabled";
     8, w1, both, OVERRUN,    "Overrun error";
    10, ro, flag, RXBRK,      "Received break";
    11, w1, both, DELTARXBRK, "RXBRK signal has changed state";
    12, w1, both, START,      "Start detected";
    13, w1, both, FRAMERR,    "Framing error";
    14, w1, both, PARITYERR,  "Parity error";
    15, w1, both, RXNOISE,    "Received noise";
    16, w1, both, ABERR,      "Autobaud error";
);
