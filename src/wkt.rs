//! APIs for the self-wake-up timer (WKT)
//!
//! See user manual, chapter 18.


use core::cell::RefCell;

use cortex_m::interrupt::{
    self,
    Mutex,
};
use embedded_hal;
use lpc82x::{
    self,
    Interrupt,
};
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


/// Indicates whether the timer has woken up
///
/// Used for communication between the interrupt handler and the main program.
static HAS_WOKEN: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));


/// Interface to the self-wake-up timer (WKT)
///
/// This API expects to be the sole owner of the WKT peripheral. Don't use
/// [`lpc82x::WKT`] directly, unless you know what you're doing.
///
/// # Limitations
///
/// This struct implements [`Timer`], but only [`Timer::set_timeout`] and
/// [`Timer::wait`] are implemented. All other [`Timer`] methods will panic.
///
/// [`lpc82x::WKT`]: ../../lpc82x/struct.WKT.html
/// [`Timer`]: ../../embedded_hal/trait.Timer.html
/// [`Timer::set_timeout`]: ../../embedded_hal/trait.Timer.html#tymethod.set_timeout
/// [`Timer::wait`]: ../../embedded_hal/trait.Timer.html#tymethod.wait
pub struct WKT<'wkt, State: InitState = init_state::Initialized> {
    wkt   : &'wkt lpc82x::WKT,
    _state: State,
}

impl<'wkt> WKT<'wkt, init_state::Unknown> {
    pub(crate) fn new(wkt: &'wkt lpc82x::WKT) -> Self {
        WKT {
            wkt   : wkt,
            _state: init_state::Unknown,
        }
    }

    /// Initialize the self-wake-up timer
    pub fn init(mut self, syscon: &mut syscon::Api)
        -> WKT<'wkt, init_state::Initialized>
    {
        syscon.enable_clock(&mut self.wkt);
        syscon.clear_reset(&mut self.wkt);

        WKT {
            wkt   : self.wkt,
            _state: init_state::Initialized,
        }
    }
}

impl<'wkt> WKT<'wkt> {
    /// Select the clock that runs the self-wake-up timer
    ///
    /// Clocks that can run the self-wake-up timer implement [`wkt::Clock`].
    ///
    /// # Limitations
    ///
    /// Selecting an external clock via the WKTCLKIN pin is currently not
    /// supported.
    ///
    /// [`wkt::Clock`]: trait.Clock.html
    pub fn select_clock<C>(&mut self) where C: Clock {
        self.wkt.ctrl.modify(|r, w|
            C::select(r, w)
        );
    }

    /// Enable the WKT interrupt
    ///
    /// The user is responsible for handling that interrupt. If the interrupt is
    /// not handled, the timer won't work correctly.
    ///
    /// See [`handle_interrupt`] for details.
    ///
    /// [`handle_interrupt`]: #method.handle_interrupt
    pub fn enable_interrupt(&mut self, nvic: &lpc82x::NVIC) {
        nvic.enable(Interrupt::WKT);
    }

    /// Handles the WKT interrupt
    ///
    /// If the WKT interrupt has been enabled, this function must be called from
    /// the interrupt handler.
    ///
    /// If this function is called from any other context than the interrupt
    /// handler, the WKT API will believe that an interrupt has occured, which
    /// may confuse the API and lead to unwanted behavior.
    ///
    /// # Example
    ///
    ///
    /// ``` rust
    /// # // We need to fake the `interrupt!` macro here, as this crate doesn't
    /// # // depend on the runtime feature of lpc82x, meaning the macro won't be
    /// # // available.
    /// # macro_rules! interrupt(($arg1:tt, $arg2:tt) => ());
    /// #
    /// #[macro_use]
    /// extern crate lpc82x;
    /// extern crate lpc82x_hal;
    ///
    /// use lpc82x_hal::WKT;
    ///
    /// interrupt!(WKT, handle_wkt);
    ///
    /// fn handle_wkt() {
    ///     WKT::handle_interrupt();
    /// }
    /// #
    /// # fn main() {}
    /// ```
    ///
    /// [`enable_interrupt`]: #method.enable_interrupt
    pub fn handle_interrupt() {
        interrupt::free(|cs| {
            let mut has_woken = HAS_WOKEN
                .borrow(cs)
                .borrow_mut();
            *has_woken = true;

            // Reset the alarm flag. Otherwise the interrupt will be triggered
            // over and over.
            lpc82x::WKT.borrow(cs).ctrl.modify(|_, w| w.alarmflag().set_bit());
        });
    }
}

impl<'wkt> embedded_hal::Timer for WKT<'wkt> {
    type Time = u32;

    fn get_timeout(&self) -> Self::Time {
        unimplemented!();
    }

    fn pause(&mut self) {
        unimplemented!();
    }

    fn restart(&mut self) {
        unimplemented!();
    }

    fn resume(&mut self) {
        unimplemented!();
    }

    fn set_timeout<T>(&mut self, timeout: T) where T: Into<Self::Time> {
        interrupt::free(|cs| {
            let mut has_woken = HAS_WOKEN
                .borrow(cs)
                .borrow_mut();
            *has_woken = false;
        });

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
            // That the alarm flag is set here, means the interrupt must be
            // disabled. If it weren't, it would be running right now.
            return Ok(());
        }

        // That we reached this point means the interrupt might be enabled. We
        // need to check its status using the static variable.
        interrupt::free(|cs| {
            let has_woken = HAS_WOKEN
                .borrow(cs)
                .borrow();

            if *has_woken {
                return Ok(());
            }

            // The flag is not set and the interrupt has not triggered. That
            // means the wait is not over.
            Err(nb::Error::WouldBlock)
        })
    }
}


/// A clock that is usable by the self-wake-up timer (WKT)
///
/// This trait should be implemented by all clocks that are supported by the
/// WKT. The user shouldn't need to implement this trait themselves, except to
/// compensate for missing pieces of HAL functionality.
pub trait Clock {
    /// Select the clock as the clock source for the WKT
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
