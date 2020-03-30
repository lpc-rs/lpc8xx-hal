//! API for the MRT peripheral
//!
//! Please be aware that this doesn't try to abstract everything, it only
//! implements the embedded-hal `Timer` functionality.
//!
//! The MRT consists of 4 channels, which are mostly separate and can each act
//! as a run-of-the-mill timer.
//!
//! # Example
//!
//! ``` no_run
//! use lpc8xx_hal::prelude::*;
//! use lpc8xx_hal::Peripherals;
//!
//! use nb::block;
//!
//! let mut p = Peripherals::take().unwrap();
//!
//! let mut syscon = p.SYSCON.split();
//! let [mut timer, _, _, _] = p.SYST.enable_delay();
//! timer.start(12_000_000u32);
//! loop {
//!     block!(timer.wait()).unwrap();
//! }
//! ```

use crate::{
    pac::{self, mrt0::CHANNEL},
    reg_proxy::RegProxy,
    syscon,
};

use embedded_hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use void::Void;

/// Represent a MRT0 instance
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

/// Represent a MRT0 channel
pub struct Channel {
    channel: u8,
    channels: RegProxy<CHANNEL>,
}

impl Channel {
    fn new(channel: u8) -> Self {
        Self {
            channel,
            channels: RegProxy::new(),
        }
    }
}

impl CountDown for Channel {
    /// The timer operates in clock ticks from the system clock, that means it
    /// runs at 12_000_000 ticks per second if you haven't changed it.
    ///
    /// It can also only use values smaller than 0x7FFFFFFF.
    type Time = u32;

    fn start<Time>(&mut self, count: Time)
    where
        Time: Into<Self::Time>,
    {
        let reload: Self::Time = count.into();
        debug_assert!(reload < (1 << 31) - 1);
        // This stops the timer, to prevent race conditions when resetting the
        // interrupt bit
        self.channels[self.channel as usize].intval.write(|w| {
            w.load().set_bit();
            unsafe { w.ivalue().bits(0) }
        });
        self.channels[self.channel as usize]
            .stat
            .write(|w| w.intflag().set_bit());
        self.channels[self.channel as usize]
            .intval
            .write(|w| unsafe { w.ivalue().bits(reload + 1) });
    }

    fn wait(&mut self) -> Result<(), Void> {
        if self.channels[self.channel as usize]
            .stat
            .read()
            .intflag()
            .is_pending_interrupt()
        {
            // Reset the interrupt flag
            self.channels[self.channel as usize]
                .stat
                .write(|w| w.intflag().set_bit());
            Ok(())
        } else {
            Err(Error::WouldBlock)
        }
    }
}

impl Periodic for Channel {}

macro_rules! channels {
    ($($field:ident, $index:expr;)*) => {
        /// Provides access to the MRT channels
        pub struct Channels {
            $(
                #[allow(missing_docs)]
                pub $field: Channel,
            )*
        }

        impl Channels {
            fn new() -> Self {
                Self {
                    $($field: Channel::new($index),)*
                }
            }
        }
    }
}

channels!(
    mrt0, 0;
    mrt1, 1;
    mrt2, 2;
    mrt3, 3;
);

reg!(CHANNEL, [CHANNEL; 4], pac::MRT0, channel);
