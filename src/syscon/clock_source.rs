//! Clock configuration for the peripherals

use crate::syscon;

/// Internal trait used configure peripheral clock sources
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClock {
    /// The variant of FCLKSEL.SEL that selects this clock source
    ///
    /// This is not available (or required) on LPC82x.
    #[cfg(feature = "845")]
    const CLOCK: crate::pac::syscon::fclksel::SEL_A;

    /// Select the clock
    ///
    /// The `selector` argument should not be required to implement this trait,
    /// but it makes sure that the caller has access to the peripheral they are
    /// selecting the clock for.
    fn select<S>(selector: &S, handle: &mut syscon::Handle)
    where
        S: PeripheralClockSelector;
}

/// Internal trait used for defining the fclksel index for a peripheral
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait PeripheralClockSelector {
    /// The index of the FCLKSEL register
    ///
    /// This is not relevant on LPC82x.
    const REGISTER_NUM: usize;
}

/// A struct containing the clock configuration for the ADC peripheral
pub struct AdcClock {
    pub(crate) caldiv: u8,
    pub(crate) div: u8,
}

impl AdcClock {
    /// Create the clock config for the ADC peripheral
    ///
    /// The system clock is divided by `caldiv` during calibration or `div`
    /// during normal operation.
    /// During calibration the frequency of the ADC peripheral has to be 500 kHz
    /// and during normal operation it can't be higher than 30 MHz.
    pub unsafe fn new(caldiv: u8, div: u8) -> Self {
        Self { caldiv, div }
    }
    /// Create a new ADC clock config with the maximum sample rate
    ///
    /// Assumes the internal oscillator runs at 12 MHz
    pub fn new_default() -> Self {
        Self { caldiv: 24, div: 0 }
    }
}

#[cfg(feature = "845")]
mod target {
    use crate::{
        pac::syscon::fclksel::SEL_A,
        syscon::{
            self,
            frg::{FRG, FRG0, FRG1},
            IOSC,
        },
    };

    use super::{PeripheralClock, PeripheralClockSelector};

    macro_rules! peripheral_clocks {
        (
            $(
                $clock:ty,
                $sel:ident;
            )*
         ) => {
            $(
                impl PeripheralClock for $clock {
                    const CLOCK: SEL_A = SEL_A::$sel;

                    fn select<S>(_: &S, syscon: &mut syscon::Handle)
                    where
                        S: PeripheralClockSelector,
                    {
                        syscon.fclksel[S::REGISTER_NUM]
                            .write(|w| w.sel().variant(Self::CLOCK));
                    }
                }
            )*
        };
    }

    peripheral_clocks!(
        FRG<FRG0>, FRG0CLK;
        FRG<FRG1>, FRG1CLK;
        IOSC, FRO;
    );
}
