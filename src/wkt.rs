//! API for the self-wake-up timer (WKT)
//!
//! The WKT peripheral is described in the user manual, chapter 9.
//!
//! # Examples
//!
//! ``` no_run
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//! extern crate nb;
//!
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::{
//!     SYSCON,
//!     WKT,
//! };
//!
//! let mut peripherals = lpc82x::Peripherals::take().unwrap();
//!
//! let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
//! let     timer  = WKT::new(peripherals.WKT);
//!
//! let mut timer = timer.enable(&mut syscon.handle);
//!
//! // Start the timer at 750000. Sine the IRC-derived clock runs at 750 kHz,
//! // this translates to a one second wait.
//! timer.start(750_000u32);
//!
//! while let Err(nb::Error::WouldBlock) = timer.wait() {
//!     // do stuff
//! }
//! ```
//!
//! [`lpc82x::WKT`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.WKT.html


use embedded_hal::timer;
use nb;

use syscon::{
    self,
    IrcDerivedClock,
};
use init_state::{
    self,
    InitState,
};
use pmu::LowPowerClock;
use raw;
use raw::wkt::ctrl;


/// The API for the self-wake-up timer (WKT)
///
/// This is the main API for the WKT. All aspects of the WKT can be controlled
/// via this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct WKT<State: InitState = init_state::Enabled> {
    wkt   : raw::WKT,
    _state: State,
}

impl WKT<init_state::Unknown> {
    /// Create an instance of `WKT`
    pub fn new(wkt: raw::WKT) -> Self {
        WKT {
            wkt   : wkt,
            _state: init_state::Unknown,
        }
    }
}

impl<State> WKT<State> where State: init_state::NotEnabled {
    /// Enable the self-wake-up timer
    ///
    /// This method is only available, if `WKT` is not already in the
    /// [`Enabled`] state. Code that attempts to call this method when the WKT
    /// is already enabled will not compile.
    ///
    /// Consumes this instance of `WKT` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(mut self, syscon: &mut syscon::Handle)
        -> WKT<init_state::Enabled>
    {
        syscon.enable_clock(&mut self.wkt);
        syscon.clear_reset(&mut self.wkt);

        WKT {
            wkt   : self.wkt,
            _state: init_state::Enabled,
        }
    }
}

impl<State> WKT<State> where State: init_state::NotDisabled {
    /// Disable the self-wake-up timer
    ///
    /// This method is only available, if `WKT` is not already in the
    /// [`Disabled`] state. Code that attempts to call this method when the WKT
    /// is already disabled will not compile.
    ///
    /// Consumes this instance of `WKT` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(mut self, syscon: &mut syscon::Handle)
        -> WKT<init_state::Disabled>
    {
        syscon.disable_clock(&mut self.wkt);

        WKT {
            wkt   : self.wkt,
            _state: init_state::Disabled,
        }
    }
}

impl WKT<init_state::Enabled> {
    /// Select the clock to run the self-wake-up timer
    ///
    /// This method is only available if the WKT has been initialized. Code
    /// attempting to call this method when this is not the case, will not
    /// compile. Call [`init`] to initialize the WKT.
    ///
    /// All clocks that can run the WKT implement a common trait. Please refer
    /// to [`wkt::Clock`] for a list of clocks that can be passed to this
    /// method. Selecting an external clock via the WKTCLKIN pin is currently
    /// not supported.
    ///
    /// # Limitations
    ///
    /// Currently nothing prevents the user from selecting a clock that is
    /// disabled, attempting to start the timer while the clock is disabled, or
    /// disabling the clock while the timer is running.
    ///
    /// [`init`]: #method.init
    /// [`wkt::Clock`]: trait.Clock.html
    pub fn select_clock<C>(&mut self) where C: Clock {
        self.wkt.ctrl.modify(|_, w|
            C::select(w)
        );
    }
}

impl timer::CountDown for WKT<init_state::Enabled> {
    type Time = u32;

    fn start<T>(&mut self, timeout: T) where T: Into<Self::Time> {
        // Either clearing the counter or writing a value to it resets the alarm
        // flag, so no reason to worry about that here.

        // It's not allowed to write to the counter without clearing it first.
        self.wkt.ctrl.modify(|_, w| w.clearctr().clear_bit());

        // The counter has been cleared, which halts counting. Writing a new
        // count is perfectly safe.
        self.wkt.count.write(|w| unsafe { w.value().bits(timeout.into()) });
    }

    fn wait(&mut self) -> nb::Result<(), !> {
        if self.wkt.ctrl.read().alarmflag().bit_is_set() {
            return Ok(());
        }

        Err(nb::Error::WouldBlock)
    }
}


/// A clock that is usable by the self-wake-up timer (WKT)
///
/// This trait should be implemented by all clocks that are supported by the
/// WKT. The user shouldn't need to implement this trait themselves, except to
/// compensate for missing pieces of HAL functionality.
pub trait Clock {
    /// Internal method to select the clock as the clock source for the WKT
    ///
    /// This is an internal method, to be called by the WKT API. Users generally
    /// shouldn't need to call this. This method is exempt from any guarantees
    /// of API stability.
    fn select<'w>(w: &'w mut ctrl::W) -> &'w mut ctrl::W;
}

impl<State> Clock for IrcDerivedClock<State> where State: InitState {
    fn select<'w>(w: &'w mut ctrl::W) -> &'w mut ctrl::W {
        w
            .sel_extclk().internal()
            .clksel().divided_irc_clock_t()
    }
}

impl<State> Clock for LowPowerClock<State> where State: InitState {
    fn select<'w>(w: &'w mut ctrl::W) -> &'w mut ctrl::W {
        w
            .sel_extclk().internal()
            .clksel().low_power_clock_thi()
    }
}
