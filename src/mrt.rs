//! API for the MRT (Multi-Rate Timer) peripheral
//!
//! Please be aware that this doesn't try to abstract everything, it only
//! implements the embedded-hal `Timer` functionality.
//!
//! The MRT consists of 4 channels, which are mostly separate and can each act
//! as a run-of-the-mill timer.

use crate::{
    pac::{self, mrt0::CHANNEL},
    reg_proxy::{Reg, RegProxy},
    syscon,
};

use embedded_hal::timer::{CountDown, Periodic};
use embedded_hal_alpha::timer::{
    CountDown as CountDownAlpha, Periodic as PeriodicAlpha,
};
use embedded_time::{clock, fraction::Fraction, Instant};
use void::Void;

/// Represents the MRT instance
pub struct MRT {
    mrt: pac::MRT0,
}

impl MRT {
    /// Assumes peripheral is in reset state
    ///
    /// This means:
    /// - Each channel is in repeat mode
    /// - All channel interrupts are disabled
    pub(crate) fn new(mrt: pac::MRT0) -> Self {
        Self { mrt }
    }

    /// Enables the MRT and splits it into it's four channels
    pub fn split(self, syscon: &mut syscon::Handle) -> Channels {
        syscon.enable_clock(&self.mrt);

        Channels::new()
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
    pub fn free(self) -> pac::MRT0 {
        self.mrt
    }
}

/// The maximum timer value
pub const MAX_VALUE: u32 = 0x7fff_ffff - 1;

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
    fn new() -> Self {
        Self(RegProxy::new())
    }

    /// Start the timer
    ///
    /// The `reload` argument must be smaller than or equal to [`MAX_VALUE`].
    ///
    /// [`MAX_VALUE`]: constant.MAX_VALUE.html
    pub fn start(&mut self, reload: impl Into<u32>) {
        let reload = reload.into();
        debug_assert!(reload <= MAX_VALUE);

        // This stops the timer, to prevent race conditions when resetting the
        // interrupt bit
        self.0.intval.write(|w| {
            w.load().set_bit();
            unsafe { w.ivalue().bits(0) }
        });
        self.0.stat.write(|w| w.intflag().set_bit());
        self.0
            .intval
            .write(|w| unsafe { w.ivalue().bits(reload + 1) });
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
    type Time = u32;

    fn start<Time>(&mut self, count: Time)
    where
        Time: Into<Self::Time>,
    {
        self.start(count);
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
    type Time = u32;

    fn try_start<Time>(&mut self, count: Time) -> Result<(), Self::Error>
    where
        Time: Into<Self::Time>,
    {
        Ok(self.start(count))
    }

    fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
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

/// Implemented for types that identify MRT channels
pub trait Trait: Reg<Target = CHANNEL> + sealed::Sealed {}

macro_rules! channels {
    ($($channel:ident, $field:ident, $index:expr;)*) => {
        /// Provides access to the MRT channels
        pub struct Channels {
            $(
                #[allow(missing_docs)]
                pub $field: Channel<$channel>,
            )*
        }

        impl Channels {
            fn new() -> Self {
                Self {
                    $($field: Channel::new(),)*
                }
            }
        }

        $(
            /// Represents one of the MRT channels
            ///
            /// Used as a type parameter for [`Channel`].
            pub struct $channel;

            reg_cluster_array!($channel, CHANNEL, pac::MRT0, channel, $index);

            impl sealed::Sealed for $channel {}
            impl Trait for $channel {}
        )*
    }
}

channels!(
    MRT0, mrt0, 0;
    MRT1, mrt1, 1;
    MRT2, mrt2, 2;
    MRT3, mrt3, 3;
);

mod sealed {
    pub trait Sealed {}
}
