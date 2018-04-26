//! API for the Power Management Unit (PMU)
//!
//! To use this API, you need to gain access to the [`PMU`] instance via
//! [`Peripherals`]. From [`PMU`], you can get the [`pmu::Handle`] and other
//! parts of the PMU API.
//!
//! This API expects to be the sole owner of the PMU. Don't use [`lpc82x::PMU`]
//! directly, unless you know what you're doing.
//!
//! The PMU is described in the user manual, chapter 6.
//!
//! # Examples
//!
//! Use the PMU to enter sleep mode:
//!
//! ``` no_run
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//!
//! use lpc82x_hal::PMU;
//!
//! let     core_peripherals = lpc82x::CorePeripherals::take().unwrap();
//! let mut peripherals      = lpc82x::Peripherals::take().unwrap();
//!
//! let mut pmu = PMU::new(&mut peripherals.PMU);
//!
//! // Enters sleep mode. Unless we set up some interrupts, we won't wake up
//! // from this again.
//! pmu.handle.enter_sleep_mode(&core_peripherals.SCB);
//! ```
//!
//! [`PMU`]: struct.PMU.html
//! [`Peripherals`]: ../struct.Peripherals.html
//! [`pmu::Handle`]: struct.Handle.html
//! [`lpc82x::PMU`]: https://docs.rs/lpc82x/0.2.*/lpc82x/struct.PMU.html


use cortex_m::{
    asm,
    interrupt,
};
use lpc82x;
use lpc82x::pmu::{
    DPDCTRL,
    PCON,
};

use clock;
use clock::state::ClockState;


/// Entry point to the PMU API
///
/// Provides access to all types that make up the PMU API. Please refer to the
/// [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct PMU<'pmu> {
    /// The handle to the PMU peripheral
    pub handle: Handle<'pmu>,

    /// The 10 kHz low-power clock
    ///
    /// # Limitations
    ///
    /// This field currently assumes that the low-power clock always starts out
    /// being disabled, but since we don't know what happened before the HAL API
    /// was initialized, this might not be the case. Please make sure you
    /// haven't enabled the low-power clock, or called any code that might have,
    /// before using this field.
    pub low_power_clock: LowPowerClock<clock::state::Disabled>,
}

impl<'pmu> PMU<'pmu> {
    /// Create an instance of `PMU`
    pub fn new(pmu: &'pmu mut lpc82x::PMU) -> Self {
        PMU {
            handle: Handle {
                dpdctrl: &pmu.dpdctrl,
                pcon   : &pmu.pcon,
            },
            low_power_clock: LowPowerClock::new(),
        }
    }
}


/// The handle to the PMU peripheral
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [module documentation]: index.html
pub struct Handle<'pmu> {
    dpdctrl: &'pmu DPDCTRL,
    pcon   : &'pmu PCON,
}

impl<'pmu> Handle<'pmu> {
    /// Enter sleep mode
    ///
    /// The microcontroller will wake up from sleep mode, if an NVIC-enabled
    /// interrupt occurs. See user manual, section 6.7.4.3.
    pub fn enter_sleep_mode(&mut self, scb: &lpc82x::SCB) {
        interrupt::free(|_| {
            // Default power mode indicates active or sleep mode.
            self.pcon.modify(|_, w|
                w.pm().default()
            );
            // The SLEEPDEEP bit must not be set for entering regular sleep
            // mode.
            unsafe {
                scb.scr.modify(|scr|
                    scr & !SLEEPDEEP
                );
            }

            asm::dsb();
            asm::wfi();
        })
    }
}


/// The 10 kHz low-power clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
pub struct LowPowerClock<State: ClockState = clock::state::Enabled> {
    _state: State,
}

impl LowPowerClock<clock::state::Disabled> {
    pub(crate) fn new() -> Self {
        LowPowerClock {
            _state: clock::state::Disabled,
        }
    }

    /// Enable the low-power clock
    ///
    /// This method is only available if the low-power clock is disabled. Code
    /// attempting to call this method when this is not the case will not
    /// compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns a new instance
    /// whose state indicates that the clock is enabled. That new instance
    /// implements [`clock::Enabled`], which might be required by APIs that need
    /// an enabled clock.
    ///
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, pmu: &mut Handle)
        -> LowPowerClock<clock::state::Enabled>
    {
        pmu.dpdctrl.modify(|_, w|
            w.lposcen().enabled()
        );

        LowPowerClock {
            _state: clock::state::Enabled,
        }
    }
}

impl LowPowerClock<clock::state::Enabled> {
    /// Disable the low-power clock
    ///
    /// This method is only available if the low-power clock is enabled. Code
    /// attempting to call this method when this is not the case will not
    /// compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns a new instance
    /// whose state indicates that the clock is disabled.
    pub fn disable(self, pmu: &mut Handle)
        -> LowPowerClock<clock::state::Disabled>
    {
        pmu.dpdctrl.modify(|_, w|
            w.lposcen().disabled()
        );

        LowPowerClock {
            _state: clock::state::Disabled,
        }
    }
}

impl<State> clock::Frequency for LowPowerClock<State> where State: ClockState {
    fn hz(&self) -> u32 { 10_000 }
}

impl clock::Enabled for LowPowerClock<clock::state::Enabled> {}


/// The SLEEPDEEP bit from the SCB's SCR register
///
/// This is a crutch, currently used due to lack of higher-level APIs in
/// cortex-m.
const SLEEPDEEP: u32 = 0x1 << 2;
