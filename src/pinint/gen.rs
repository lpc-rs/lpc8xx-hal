use super::{interrupt::Interrupt, traits::Trait};

macro_rules! interrupts {
    ($($struct:ident, $field:ident, $index:expr;)*) => {
        /// Provides access to all pin interrupts
        #[allow(missing_docs)]
        pub struct Interrupts<State> {
            $(pub $field: Interrupt<$struct, (), State>,)*
        }

        impl<State> Interrupts<State> {
            pub(crate) fn new() -> Self {
                Self {
                    $(
                        $field: Interrupt::new(),
                    )*
                }
            }
        }


        $(
            /// Represents a pin interrupt
            pub struct $struct;

            impl Trait for $struct {
                const INDEX: usize = $index;
                const MASK: u8 = 0x1 << $index;
            }
        )*
    };
}

interrupts!(
    PININT0, pinint0, 0;
    PININT1, pinint1, 1;
    PININT2, pinint2, 2;
    PININT3, pinint3, 3;
    PININT4, pinint4, 4;
    PININT5, pinint5, 5;
    PININT6, pinint6, 6;
    PININT7, pinint7, 7;
);
