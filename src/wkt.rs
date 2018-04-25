//! API for the self-wake-up timer (WKT)
//!
//! This API expects to be the sole owner of the WKT peripheral. Don't use
//! [`lpc82x::WKT`] directly, unless you know what you're doing.
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
//! let peripherals = unsafe { lpc82x::Peripherals::all() };
//!
//! let mut syscon = unsafe { SYSCON::new(peripherals.SYSCON) };
//! let     timer  = unsafe { WKT::new(peripherals.WKT)       };
//!
//! let mut timer = timer.init(&mut syscon.handle);
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
//! [`lpc82x::WKT`]: https://docs.rs/lpc82x/0.2.*/lpc82x/struct.WKT.html


use embedded_hal::timer;
use lpc82x;
use lpc82x::wkt::ctrl;
use nb;

use syscon::{
    self,
    IrcDerivedClock,
};
use clock::state::ClockState;
use init_state::{
    self,
    InitState,
};
use pmu::LowPowerClock;


/// The API for the self-wake-up timer (WKT)
///
/// This is the main API for the WKT. All aspects of the WKT can be controlled
/// via this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct WKT<'wkt, State: InitState = init_state::Enabled> {
    wkt   : &'wkt lpc82x::WKT,
    _state: State,
}

impl<'wkt> WKT<'wkt, init_state::Unknown> {
    /// Create an instance of `WKT`
    ///
    /// # Safety
    ///
    /// Only a single instance of `WKT` is allowed to exist at any given time.
    /// If you use this method to create multiple instances of `WKT`, the
    /// guarantees this API makes cannot be upheld.
    pub unsafe fn new(wkt: &'wkt lpc82x::WKT) -> Self {
        WKT {
            wkt   : wkt,
            _state: init_state::Unknown,
        }
    }

    /// Initialize the self-wake-up timer
    ///
    /// This method is only available, if `WKT` is in the [`Unknown`] state.
    /// This is the initial state after initializing the HAL API. Code that
    /// attempts to call this method after the WKT has been initialized will not
    /// compile.
    ///
    /// Consumes this instance of `WKT` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`]. This makes available
    /// those methods that can only work if the WKT is enabled.
    ///
    /// [`Unknown`]: ../init_state/struct.Unknown.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn init(mut self, syscon: &mut syscon::Handle)
        -> WKT<'wkt, init_state::Enabled>
    {
        syscon.enable_clock(&mut self.wkt);
        syscon.clear_reset(&mut self.wkt);

        WKT {
            wkt   : self.wkt,
            _state: init_state::Enabled,
        }
    }
}

impl<'wkt> WKT<'wkt> {
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
        self.wkt.ctrl.modify(|r, w|
            C::select(r, w)
        );
    }
}

impl<'wkt> timer::CountDown for WKT<'wkt> {
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
    fn select<'w>(r: &ctrl::R, w: &'w mut ctrl::W) -> &'w mut ctrl::W;
}

impl<State> Clock for IrcDerivedClock<State> where State: ClockState {
    fn select<'w>(r: &ctrl::R, w: &'w mut ctrl::W) -> &'w mut ctrl::W {
        unsafe {
            w
                .bits(r.bits() & !SEL_EXTCLK)
                .clksel().divided_irc_clock_t()
        }
    }
}

impl<State> Clock for LowPowerClock<State> where State: ClockState {
    fn select<'w>(r: &ctrl::R, w: &'w mut ctrl::W) -> &'w mut ctrl::W {
        unsafe {
            w
                .bits(r.bits() & !SEL_EXTCLK)
                .clksel().low_power_clock_thi()
        }
    }
}


/// The SEL_EXTCLK bit in WKT's CTRL register
///
/// This belongs in the lpc82x crate, but it's currently missing, due to a bug
/// in the SVD file.
const SEL_EXTCLK: u32 = 0x1 << 3;
