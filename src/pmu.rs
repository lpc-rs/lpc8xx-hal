//! APIs for the power management unit (PMU)
//!
//! See user manual, chapter 6.


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


/// Interface to the power management unit (PMU)
///
/// This API expects to be the sole owner of the PMU peripheral. Don't use
/// [`lpc82x::PMU`] directly, unless you know what you're doing.
///
/// [`lpc82x::PMU`]: ../../lpc82x/struct.PMU.html
pub struct PMU<'pmu> {
    /// Main PMU API
    pub api: Api<'pmu>,

    /// The 10 kHz low-power clock
    ///
    /// Can be used to run the self-wake-up timer (WKT).
    pub low_power_clock: LowPowerClock<clock::state::Disabled>,
}

impl<'pmu> PMU<'pmu> {
    pub(crate) fn new(pmu: &'pmu lpc82x::PMU) -> Self {
        PMU {
            api: Api {
                dpdctrl: &pmu.dpdctrl,
                pcon   : &pmu.pcon,
            },
            low_power_clock: LowPowerClock::new(),
        }
    }
}


/// Main API of the PMU peripheral
pub struct Api<'pmu> {
    dpdctrl: &'pmu DPDCTRL,
    pcon   : &'pmu PCON,
}

impl<'pmu> Api<'pmu> {
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
/// See user manual, section 18.5.1.
pub struct LowPowerClock<State: ClockState = clock::state::Enabled> {
    _state: State,
}

impl LowPowerClock<clock::state::Disabled> {
    /// Create a new instance of the low-power clock
    ///
    /// This method is only intended for use within [`System`].
    ///
    /// [`System`]: ../struct.System.html
    pub(crate) fn new() -> Self {
        LowPowerClock {
            _state: clock::state::Disabled,
        }
    }

    /// Enables the clock
    ///
    /// This method consumes this instance of `LowPowerClock` and returns an
    /// instance that implements [`clock::Enabled`].
    ///
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, pmu: &mut Api) -> LowPowerClock<clock::state::Enabled> {
        pmu.dpdctrl.modify(|_, w|
            w.lposcen().enabled()
        );

        LowPowerClock {
            _state: clock::state::Enabled,
        }
    }
}

impl LowPowerClock<clock::state::Enabled> {
    /// Disables the clock
    ///
    /// This method consumes an enabled instance of `LowPowerClock` and returns
    /// an instance that is disabled.
    pub fn disable(self, pmu: &mut Api)
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
