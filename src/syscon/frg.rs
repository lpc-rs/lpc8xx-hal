//! The fractional generator (FRG), available on LPC845

use crate::{
    pac::{
        self,
        syscon::frg::{frgclksel::SEL_A, FRGCLKSEL, FRGDIV, FRGMULT},
    },
    reg_proxy::{Reg, RegProxy},
};

/// Fractional generator
///
/// Can be used as a clock source for serial peripherals.
pub struct FRG<I: Instance> {
    div: RegProxy<I::Div>,
    mult: RegProxy<I::Mult>,
    clksel: RegProxy<I::Clksel>,
}

impl<I> FRG<I>
where
    I: Instance,
{
    pub(crate) fn new() -> Self {
        Self {
            div: RegProxy::new(),
            mult: RegProxy::new(),
            clksel: RegProxy::new(),
        }
    }

    /// Select clock source for FRG
    pub fn select_clock(&mut self, clock: SEL_A) {
        self.clksel.write(|w| w.sel().variant(clock));
    }

    /// Set the fractional generator divider value
    pub fn set_div(&mut self, div: u8) {
        // Safe, as all `u8` values are valid.
        self.div.write(|w| unsafe { w.bits(div.into()) });
    }

    /// Set the fractional generator multiplier value
    pub fn set_mult(&mut self, mult: u8) {
        // Safe, as all `u8` values are valid.
        self.mult.write(|w| unsafe { w.bits(mult.into()) });
    }
}

/// Internal implementation detail
///
/// This trait should neither be used nor implemented by the user.
pub trait Instance {
    /// FRG0DIV or FRG1DIV
    type Div: Reg<Target = FRGDIV>;

    /// FRG0MULT or FRG1MULT
    type Mult: Reg<Target = FRGMULT>;

    /// FRG0CLKSEL or FRG1CLKSEL
    type Clksel: Reg<Target = FRGCLKSEL>;
}

macro_rules! instances {
    ($($name:ident, $field:ident, $div:ident, $mult:ident, $clksel:ident;)*) => {
        $(
            /// Represents an instance of the fractional generator
            ///
            /// See [`FRG`] for details.
            pub struct $name;


            /// Represents and instance of FRGDIV
            ///
            /// This is an internal implementation detail.
            pub struct $div;

            /// Represents and instance of FRGMULT
            ///
            /// This is an internal implementation detail.
            pub struct $mult;

            /// Represents and instance of FRGCLKSEL
            ///
            /// This is an internal implementation detail.
            pub struct $clksel;

            reg_cluster!($div,    FRGDIV,    pac::SYSCON, $field, frgdiv);
            reg_cluster!($mult,   FRGMULT,   pac::SYSCON, $field, frgmult);
            reg_cluster!($clksel, FRGCLKSEL, pac::SYSCON, $field, frgclksel);

            impl Instance for $name {
                type Div    = $div;
                type Mult   = $mult;
                type Clksel = $clksel;
            }
        )*
    }
}

instances!(
    FRG0, frg0, FRG0DIV, FRG0MULT, FRG0CLKSEL;
    FRG1, frg1, FRG1DIV, FRG1MULT, FRG1CLKSEL;
);
