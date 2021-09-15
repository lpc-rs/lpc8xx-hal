use crate::reg_proxy::{Reg, RegProxy};

use embedded_hal::timer::{CountDown, Periodic};
use embedded_hal_alpha::timer::{
    nb::CountDown as CountDownAlpha, Periodic as PeriodicAlpha,
};
use embedded_time::{clock, fraction::Fraction, Instant};
use void::Void;

use super::{Ticks, Trait};

/// Represents a MRT0 channel
///
/// # `embedded-hal` traits
/// - [`embedded_hal::timer::CountDown`]
///
/// [`embedded_hal::timer::CountDown`]: #impl-CountDown
pub struct Channel<T: Reg>(RegProxy<T>);

impl<T> Channel<T>
where
    T: Trait,
{
    pub(super) fn new() -> Self {
        Self(RegProxy::new())
    }

    /// Start the timer
    ///
    /// The `reload` argument must be smaller than or equal to [`MAX_VALUE`].
    ///
    /// [`MAX_VALUE`]: constant.MAX_VALUE.html
    pub fn start(&mut self, reload: Ticks) {
        // This stops the timer, to prevent race conditions when resetting the
        // interrupt bit
        self.0.intval.write(|w| {
            w.load().set_bit();
            unsafe { w.ivalue().bits(0) }
        });
        self.0.stat.write(|w| w.intflag().set_bit());
        self.0
            .intval
            .write(|w| unsafe { w.ivalue().bits(reload.0 + 1) });
    }

    /// Indicates whether the timer is running
    pub fn is_running(&self) -> bool {
        self.0.stat.read().run().is_running()
    }

    /// Returns the current timer value
    pub fn value(&self) -> u32 {
        self.0.timer.read().value().bits()
    }

    /// Returns the reload value of the timer
    pub fn reload_value(&self) -> u32 {
        self.0.intval.read().ivalue().bits()
    }

    /// Non-blockingly "waits" until the count down finishes
    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.0.stat.read().intflag().is_pending_interrupt() {
            // Reset the interrupt flag
            self.0.stat.write(|w| w.intflag().set_bit());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T> CountDown for Channel<T>
where
    T: Trait,
{
    /// The timer operates in clock ticks from the system clock, that means it
    /// runs at 12_000_000 ticks per second if you haven't changed it.
    ///
    /// It can also only use values smaller than 0x7FFFFFFF.
    type Time = Ticks;

    fn start<Time>(&mut self, count: Time)
    where
        Time: Into<Self::Time>,
    {
        self.start(count.into());
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        self.wait()
    }
}

impl<T> CountDownAlpha for Channel<T>
where
    T: Trait,
{
    type Error = Void;

    /// The timer operates in clock ticks from the system clock, that means it
    /// runs at 12_000_000 ticks per second if you haven't changed it.
    ///
    /// It can also only use values smaller than 0x7FFFFFFF.
    type Time = Ticks;

    fn start<Time>(&mut self, count: Time) -> Result<(), Self::Error>
    where
        Time: Into<Self::Time>,
    {
        Ok(self.start(count.into()))
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.wait()
    }
}

impl<T> Periodic for Channel<T> where T: Trait {}

impl<T> PeriodicAlpha for Channel<T> where T: Trait {}

impl<T> embedded_time::Clock for Channel<T>
where
    T: Trait,
{
    type T = u32;

    const SCALING_FACTOR: Fraction = Fraction::new(1, 12_000_000);

    fn try_now(&self) -> Result<Instant<Self>, clock::Error> {
        if self.is_running() {
            // embedded-time assumes that clocks are counting up, but we are
            // counting down here. Thus, the need for some translation.
            Ok(Instant::new(self.reload_value() - self.value()))
        } else {
            Err(clock::Error::NotRunning)
        }
    }
}
