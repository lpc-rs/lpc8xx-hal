//! API for delays with the systick timer
//!
//! Please be aware of potential overflows when using `delay_us`.
//! E.g. at 30MHz the maximum delay is 146 seconds.
//!
//! # Example
//!
//! ``` no_run
//! use lpc8xx_hal::{
//!     prelude::*,
//!     delay::Delay,
//!     pac::CorePeripherals,
//! };
//!
//! let mut cp = CorePeripherals::take().unwrap();
//!
//! let mut delay = Delay::new(cp.SYST);
//! loop {
//!     delay.delay_ms(1_000_u16);
//! }
//! ```

use cortex_m::peripheral::syst::SystClkSource;

use crate::pac::SYST;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal_alpha::blocking::delay::{
    DelayMs as DelayMsAlpha, DelayUs as DelayUsAlpha,
};
use void::Void;

const SYSTICK_RANGE: u32 = 0x0100_0000;
const SYSTEM_CLOCK: u32 = 12_000_000;

/// System timer (SysTick) as a delay provider
///
/// # `embedded-hal` traits
/// - [`embedded_hal::blocking::delay::DelayUs`]
/// - [`embedded_hal::blocking::delay::DelayMs`]
///
/// [`embedded_hal::blocking::delay::DelayUs`]: #impl-DelayUs%3Cu32%3E
/// [`embedded_hal::blocking::delay::DelayMs`]: #impl-DelayMs%3Cu32%3E
#[derive(Clone)]
pub struct Delay {
    scale: u32,
}

impl Delay {
    /// Configures the system timer (SysTick) as a delay provider
    pub fn new(mut syst: SYST) -> Self {
        assert!(SYSTEM_CLOCK >= 1_000_000);
        let scale = SYSTEM_CLOCK / 1_000_000;
        syst.set_clock_source(SystClkSource::Core);

        syst.set_reload(SYSTICK_RANGE - 1);
        syst.clear_current();
        syst.enable_counter();

        Delay { scale }
        // As access to the count register is possible without a reference to the systick, we can
        // safely clone the enabled instance.
    }
}

impl DelayMs<u32> for Delay {
    /// Pauses execution for `ms` milliseconds
    // At 30 MHz (the maximum frequency), calling delay_us with ms * 1_000 directly overflows at 0x418937 (over the max u16 value)
    // So we implement a separate, higher level, delay loop
    fn delay_ms(&mut self, mut ms: u32) {
        const MAX_MS: u32 = 0x0000_FFFF;
        while ms != 0 {
            let current_ms = if ms <= MAX_MS { ms } else { MAX_MS };
            self.delay_us(current_ms * 1_000);
            ms -= current_ms;
        }
    }
}

impl DelayMsAlpha<u32> for Delay {
    type Error = Void;

    /// Pauses execution for `ms` milliseconds
    // At 30 MHz (the maximum frequency), calling delay_us with ms * 1_000 directly overflows at 0x418937 (over the max u16 value)
    // So we implement a separate, higher level, delay loop
    fn try_delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        Ok(self.delay_ms(ms))
    }
}

impl DelayMs<u16> for Delay {
    /// Pauses execution for `ms` milliseconds
    fn delay_ms(&mut self, ms: u16) {
        // Call delay_us directly, since we don't have to use the additional
        // delay loop the u32 variant uses
        self.delay_us(ms as u32 * 1_000);
    }
}

impl DelayMsAlpha<u16> for Delay {
    type Error = Void;

    /// Pauses execution for `ms` milliseconds
    // At 30 MHz (the maximum frequency), calling delay_us with ms * 1_000 directly overflows at 0x418937 (over the max u16 value)
    // So we implement a separate, higher level, delay loop
    fn try_delay_ms(&mut self, ms: u16) -> Result<(), Self::Error> {
        Ok(self.delay_ms(ms))
    }
}

impl DelayMs<u8> for Delay {
    /// Pauses execution for `ms` milliseconds
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u16);
    }
}

impl DelayMsAlpha<u8> for Delay {
    type Error = Void;

    /// Pauses execution for `ms` milliseconds
    // At 30 MHz (the maximum frequency), calling delay_us with ms * 1_000 directly overflows at 0x418937 (over the max u16 value)
    // So we implement a separate, higher level, delay loop
    fn try_delay_ms(&mut self, ms: u8) -> Result<(), Self::Error> {
        Ok(self.delay_ms(ms))
    }
}

// At 30MHz (the maximum frequency), this overflows at approx. 2^32 / 30 = 146 seconds
impl DelayUs<u32> for Delay {
    /// Pauses execution for `us` microseconds
    fn delay_us(&mut self, us: u32) {
        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        // Here half the maximum is used so we have some play if there's a long running interrupt.
        const MAX_TICKS: u32 = 0x007F_FFFF;

        let mut total_ticks = us * self.scale;

        while total_ticks != 0 {
            let current_ticks = if total_ticks <= MAX_TICKS {
                total_ticks
            } else {
                MAX_TICKS
            };

            let start_count = SYST::get_current();
            total_ticks -= current_ticks;

            // Use the wrapping subtraction and the modulo to deal with the systick wrapping around
            // from 0 to 0xFFFF
            while (start_count.wrapping_sub(SYST::get_current())
                % SYSTICK_RANGE)
                < current_ticks
            {}
        }
    }
}

impl DelayUsAlpha<u32> for Delay {
    type Error = Void;

    /// Pauses execution for `us` microseconds
    fn try_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        Ok(self.delay_us(us))
    }
}

impl DelayUs<u16> for Delay {
    /// Pauses execution for `us` microseconds
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUsAlpha<u16> for Delay {
    type Error = Void;

    /// Pauses execution for `us` microseconds
    fn try_delay_us(&mut self, us: u16) -> Result<(), Self::Error> {
        Ok(self.delay_us(us))
    }
}

impl DelayUs<u8> for Delay {
    /// Pauses execution for `us` microseconds
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}

impl DelayUsAlpha<u8> for Delay {
    type Error = Void;

    /// Pauses execution for `us` microseconds
    fn try_delay_us(&mut self, us: u8) -> Result<(), Self::Error> {
        Ok(self.delay_us(us))
    }
}
