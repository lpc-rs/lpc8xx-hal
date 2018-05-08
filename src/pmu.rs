//! API for the Power Management Unit (PMU)
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
//! let mut core_peripherals = lpc82x::CorePeripherals::take().unwrap();
//! let mut peripherals      = lpc82x::Peripherals::take().unwrap();
//!
//! let mut pmu = PMU::new(&mut peripherals.PMU);
//!
//! // Enters sleep mode. Unless we set up some interrupts, we won't wake up
//! // from this again.
//! pmu.handle.enter_sleep_mode(&mut core_peripherals.SCB);
//! ```
//!
//! [`PMU`]: struct.PMU.html
//! [`Peripherals`]: ../struct.Peripherals.html
//! [`pmu::Handle`]: struct.Handle.html
//! [`lpc82x::PMU`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.PMU.html


use cortex_m::{
    asm,
    interrupt,
};

use clock;
use init_state::{
    self,
    InitState,
};
use raw;


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
    pub low_power_clock: LowPowerClock<init_state::Unknown>,
}

impl<'pmu> PMU<'pmu> {
    /// Create an instance of `PMU`
    pub fn new(pmu: &'pmu mut raw::PMU) -> Self {
        PMU {
            handle: Handle {
                pmu: pmu,
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
    pmu: &'pmu raw::PMU,
}

impl<'pmu> Handle<'pmu> {
    /// Enter sleep mode
    ///
    /// The microcontroller will wake up from sleep mode, if an NVIC-enabled
    /// interrupt occurs. See user manual, section 6.7.4.3.
    pub fn enter_sleep_mode(&mut self, scb: &mut raw::SCB) {
        interrupt::free(|_| {
            // Default power mode indicates active or sleep mode.
            self.pmu.pcon.modify(|_, w|
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
pub struct LowPowerClock<State: InitState = init_state::Enabled> {
    _state: State,
}

impl LowPowerClock<init_state::Unknown> {
    pub(crate) fn new() -> Self {
        LowPowerClock {
            _state: init_state::Unknown,
        }
    }
}

impl<State> LowPowerClock<State> where State: init_state::NotEnabled {
    /// Enable the low-power clock
    ///
    /// This method is only available if the low-power clock is not already
    /// enabled. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns a new instance
    /// whose state indicates that the clock is enabled. That new instance
    /// implements [`clock::Enabled`], which might be required by APIs that need
    /// an enabled clock.
    ///
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, pmu: &mut Handle)
        -> LowPowerClock<init_state::Enabled>
    {
        pmu.pmu.dpdctrl.modify(|_, w|
            w.lposcen().enabled()
        );

        LowPowerClock {
            _state: init_state::Enabled,
        }
    }
}

impl<State> LowPowerClock<State> where State: init_state::NotDisabled {
    /// Disable the low-power clock
    ///
    /// This method is only available if the low-power clock is not already
    /// disabled. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns a new instance
    /// whose state indicates that the clock is disabled.
    pub fn disable(self, pmu: &mut Handle)
        -> LowPowerClock<init_state::Disabled>
    {
        pmu.pmu.dpdctrl.modify(|_, w|
            w.lposcen().disabled()
        );

        LowPowerClock {
            _state: init_state::Disabled,
        }
    }
}

impl<State> clock::Frequency for LowPowerClock<State> where State: InitState {
    fn hz(&self) -> u32 { 10_000 }
}

impl clock::Enabled for LowPowerClock<init_state::Enabled> {}


/// The SLEEPDEEP bit from the SCB's SCR register
///
/// This is a crutch, currently used due to lack of higher-level APIs in
/// cortex-m.
const SLEEPDEEP: u32 = 0x1 << 2;
