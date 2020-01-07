//! API for the self-wake-up timer (WKT)
//!
//! The entry point to this API is [`WKT`].
//!
//! The WKT peripheral is described in the user manual, chapter 9.
//!
//! # Examples
//!
//! ``` no_run
//! extern crate lpc82x_hal;
//! extern crate nb;
//!
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::Peripherals;
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut syscon = p.SYSCON.split();
//! let mut timer  = p.WKT.enable(&mut syscon.handle);
//!
//! // Start the timer at 750000. Sine the IRC/FRO-derived clock runs at 750 kHz,
//! // this translates to a one second wait.
//! timer.start(750_000u32);
//!
//! while let Err(nb::Error::WouldBlock) = timer.wait() {
//!     // do stuff
//! }
//! ```
//!
//! Please refer to the [examples in the repository] for more example code.
//!
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/lpc82x-hal/examples

use embedded_hal::timer;
use nb;
use void::Void;

use crate::{
    init_state,
    pac::{self, wkt::ctrl},
    pmu::LowPowerClock,
    syscon::{self, IoscDerivedClock},
};

/// Interface to the self-wake-up timer (WKT)
///
/// Controls the WKT. Use [`Peripherals`] to gain access to an instance of this
/// struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct WKT<State = init_state::Enabled> {
    wkt: pac::WKT,
    _state: State,
}

impl WKT<init_state::Disabled> {
    pub(crate) fn new(wkt: pac::WKT) -> Self {
        WKT {
            wkt: wkt,
            _state: init_state::Disabled,
        }
    }

    /// Enable the WKT
    ///
    /// This method is only available, if `WKT` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `WKT` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(mut self, syscon: &mut syscon::Handle) -> WKT<init_state::Enabled> {
        syscon.enable_clock(&mut self.wkt);

        WKT {
            wkt: self.wkt,
            _state: init_state::Enabled(()),
        }
    }
}

impl WKT<init_state::Enabled> {
    /// Disable the WKT
    ///
    /// This method is only available, if `WKT` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `WKT` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(mut self, syscon: &mut syscon::Handle) -> WKT<init_state::Disabled> {
        syscon.disable_clock(&mut self.wkt);

        WKT {
            wkt: self.wkt,
            _state: init_state::Disabled,
        }
    }

    /// Select the clock that runs the self-wake-up timer
    ///
    /// This method is only available if the WKT is enabled. Code attempting to
    /// call this method when this is not the case will not compile.
    ///
    /// All clocks that can run the WKT implement a common trait. Please refer
    /// to [`wkt::Clock`] for a list of clocks that can be passed to this
    /// method. Selecting an external clock via the WKTCLKIN pin is currently
    /// not supported.
    ///
    /// # Limitations
    ///
    /// Currently, nothing prevents the user from selecting a clock that is
    /// disabled, attempting to start the timer while the clock is disabled, or
    /// disabling the clock while the timer is running.
    ///
    /// [`wkt::Clock`]: trait.Clock.html
    pub fn select_clock<C>(&mut self)
    where
        C: Clock,
    {
        self.wkt.ctrl.modify(|_, w| {
            C::select(w);
            w
        });
    }
}

impl timer::CountDown for WKT<init_state::Enabled> {
    type Time = u32;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        // Either clearing the counter or writing a value to it resets the alarm
        // flag, so no reason to worry about that here.

        // It's not allowed to write to the counter without clearing it first.
        self.wkt.ctrl.modify(|_, w| w.clearctr().clear_bit());

        // The counter has been cleared, which halts counting. Writing a new
        // count is perfectly safe.
        self.wkt
            .count
            .write(|w| unsafe { w.value().bits(timeout.into()) });
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.wkt.ctrl.read().alarmflag().bit_is_set() {
            return Ok(());
        }

        Err(nb::Error::WouldBlock)
    }
}

impl<State> WKT<State> {
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
    pub fn free(self) -> pac::WKT {
        self.wkt
    }
}

/// A clock that is usable by the self-wake-up timer (WKT)
///
/// This trait is implemented for all clocks that are supported by the WKT. The
/// user shouldn't need to implement this trait themselves.
pub trait Clock {
    /// Internal method to select the clock as the clock source for the WKT
    ///
    /// This is an internal method, to be called by the WKT API. Users generally
    /// shouldn't need to call this. This method is exempt from any guarantees
    /// of API stability.
    fn select(w: &mut ctrl::W);
}

impl<State> Clock for IoscDerivedClock<State> {
    fn select(w: &mut ctrl::W) {
        w.sel_extclk().internal();
        target::select_internal_oscillator(w);
    }
}

impl<State> Clock for LowPowerClock<State> {
    fn select(w: &mut ctrl::W) {
        w.sel_extclk().internal().clksel().low_power_clock();
    }
}

#[cfg(feature = "82x")]
mod target {
    pub fn select_internal_oscillator(w: &mut crate::pac::wkt::ctrl::W) {
        w.clksel().divided_irc_clock();
    }
}

#[cfg(feature = "845")]
mod target {
    pub fn select_internal_oscillator(w: &mut crate::pac::wkt::ctrl::W) {
        w.clksel().divided_fro_clock();
    }
}
