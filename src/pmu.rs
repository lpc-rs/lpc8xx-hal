//! API for the Power Management Unit (PMU)
//!
//! The entry point to this API is [`PMU`]. Please refer to [`PMU`]'s
//! documentation for additional information.
//!
//! The PMU is described in the user manual, chapter 6.
//!
//! # Examples
//!
//! Use the PMU to enter sleep mode:
//!
//! ``` no_run
//! use lpc82x_hal::{
//!     raw,
//!     Peripherals,
//! };
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut pmu = p.PMU.split();
//!
//! // Enters sleep mode. Unless we set up some interrupts, we won't wake up
//! // from this again.
//! pmu.handle.enter_sleep_mode(&mut p.SCB);
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/lpc82x-hal/examples

use cortex_m::{asm, interrupt};

use crate::{clock, init_state, pac};

/// Entry point to the PMU API
///
/// The PMU API is split into multiple parts, which are all available through
/// [`pmu::Parts`]. You can use [`PMU::split`] to gain access to [`pmu::Parts`].
///
/// You can also use this struct to gain access to the raw peripheral using
/// [`PMU::free`]. This is the main reason this struct exists, as it's no longer
/// possible to do this after the API has been split.
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`pmu::Parts`]: struct.Parts.html
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct PMU {
    pmu: pac::PMU,
}

impl PMU {
    pub(crate) fn new(pmu: pac::PMU) -> Self {
        PMU { pmu }
    }

    /// Splits the PMU API into its component parts
    ///
    /// This is the regular way to access the PMU API. It exists as an explicit
    /// step, as it's no longer possible to gain access to the raw peripheral
    /// using [`PMU::free`] after you've called this method.
    pub fn split(self) -> Parts {
        Parts {
            handle: Handle { pmu: self.pmu },
            low_power_clock: LowPowerClock::new(),
        }
    }

    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> pac::PMU {
        self.pmu
    }
}

/// The main API for the PMU peripheral
///
/// Provides access to all types that make up the PMU API. Please refer to the
/// [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts {
    /// The handle to the PMU peripheral
    pub handle: Handle,

    /// The 10 kHz low-power clock
    pub low_power_clock: LowPowerClock<init_state::Disabled>,
}

/// Handle to the PMU peripheral
///
/// This handle to the PMU peripheral provides access to the main part of the
/// PMU API. It is also required by other parts of the API to synchronize access
/// the the underlying registers, wherever this is required.
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [module documentation]: index.html
pub struct Handle {
    pmu: pac::PMU,
}

impl Handle {
    /// Enter sleep mode
    ///
    /// The microcontroller will wake up from sleep mode, if an NVIC-enabled
    /// interrupt occurs. See user manual, section 6.7.4.3.
    pub fn enter_sleep_mode(&mut self, scb: &mut pac::SCB) {
        interrupt::free(|_| {
            // Default power mode indicates active or sleep mode.
            self.pmu.pcon.modify(|_, w| w.pm().default());

            // The SLEEPDEEP bit must be cleared when entering regular sleep
            // mode. See user manual, section 6.7.4.2.
            scb.clear_sleepdeep();

            asm::dsb();
            asm::wfi();
        })
    }

    /// Enter deep-sleep mode
    ///
    /// The microcontroller will wake up from deep-sleep mode, if an
    /// NVIC-enabled interrupt occurs. See user manual, section 6.7.5.3.
    ///
    /// # Limitations
    ///
    /// According to the user manual, section 6.7.5.2, the IRC must be selected
    /// as the main clock before entering deep-sleep mode.
    ///
    /// If you intend to wake up from this mode again, you need to configure the
    /// STARTERP0 and STARTERP1 registers of the SYSCON appropriately. See user
    /// manual, section 6.5.1.
    ///
    /// # Safety
    ///
    /// The configuration of various peripherals after wake-up is controlled by
    /// the PDAWAKECFG register. If the configuration in that register doesn't
    /// match the peripheral states in the HAL API, you can confuse the API into
    /// believing that peripherals have a different state than they actually
    /// have which can lead to all kinds of adverse consequences.
    ///
    /// Please make sure that the peripheral states configured in PDAWAKECFG
    /// match the peripheral states as tracked by the API before calling this
    /// method.
    pub unsafe fn enter_deep_sleep_mode(&mut self, scb: &mut pac::SCB) {
        interrupt::free(|_| {
            self.pmu.pcon.modify(|_, w| w.pm().deep_sleep_mode());

            // The SLEEPDEEP bit must be set for entering regular sleep mode.
            // See user manual, section 6.7.5.2.
            scb.set_sleepdeep();

            asm::dsb();
            asm::wfi();
        })
    }

    /// Enter power-down mode
    ///
    /// The microcontroller will wake up from power-down mode, if an
    /// NVIC-enabled interrupt occurs. See user manual, section 6.7.6.3.
    ///
    /// # Limitations
    ///
    /// According to the user manual, section 6.7.6.2, the IRC must be selected
    /// as the main clock before entering deep-sleep mode.
    ///
    /// If you intend to wake up from this mode again, you need to configure the
    /// STARTERP0 and STARTERP1 registers of the SYSCON appropriately. See user
    /// manual, section 6.5.1.
    ///
    /// # Safety
    ///
    /// The configuration of various peripherals after wake-up is controlled by
    /// the PDAWAKECFG register. If the configuration in that register doesn't
    /// match the peripheral states in this API, you can confuse the API into
    /// believing that peripherals have a different state than they actually
    /// have which can lead to all kinds of adverse consequences.
    ///
    /// Please make sure that the peripheral states configured in PDAWAKECFG
    /// match the peripheral states as tracked by the API before calling this
    /// method.
    pub unsafe fn enter_power_down_mode(&mut self, scb: &mut pac::SCB) {
        interrupt::free(|_| {
            self.pmu.pcon.modify(|_, w| w.pm().power_down_mode());

            // The SLEEPDEEP bit must be set for entering regular sleep mode.
            // See user manual, section 6.7.5.2.
            scb.set_sleepdeep();

            asm::dsb();
            asm::wfi();
        })
    }
}

/// The 10 kHz low-power clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
pub struct LowPowerClock<State = init_state::Enabled> {
    _state: State,
}

impl LowPowerClock<init_state::Disabled> {
    pub(crate) fn new() -> Self {
        LowPowerClock {
            _state: init_state::Disabled,
        }
    }
}

impl LowPowerClock<init_state::Disabled> {
    /// Enable the low-power clock
    ///
    /// This method is only available, if `LowPowerClock` is in the [`Disabled`]
    /// state. Code that attempts to call this method when the clock is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`]. That new
    /// instance implements [`clock::Enabled`], which might be required by APIs
    /// that need an enabled clock.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, pmu: &mut Handle) -> LowPowerClock<init_state::Enabled> {
        pmu.pmu.dpdctrl.modify(|_, w| w.lposcen().enabled());

        LowPowerClock {
            _state: init_state::Enabled(()),
        }
    }
}

impl LowPowerClock<init_state::Enabled> {
    /// Disable the low-power clock
    ///
    /// This method is only available, if `LowPowerClock` is in the [`Enabled`]
    /// state. Code that attempts to call this method when the clock is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns another instance
    /// that has its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(self, pmu: &mut Handle) -> LowPowerClock<init_state::Disabled> {
        pmu.pmu.dpdctrl.modify(|_, w| w.lposcen().disabled());

        LowPowerClock {
            _state: init_state::Disabled,
        }
    }
}

impl<State> clock::Frequency for LowPowerClock<State> {
    fn hz(&self) -> u32 {
        10_000
    }
}

impl clock::Enabled for LowPowerClock<init_state::Enabled> {}
