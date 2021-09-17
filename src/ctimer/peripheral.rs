use core::convert::Infallible;

use embedded_hal::{Pwm, PwmPin};
use embedded_hal_alpha::pwm::blocking::{
    Pwm as PwmAlpha, PwmPin as PwmPinAlpha,
};

use crate::{
    init_state::{Disabled, Enabled},
    pac::CTIMER0,
    swm, syscon,
};

use super::{
    channel::{
        self,
        state::{Attached, Detached},
    },
    gen::{Channel1, Channel2, Channel3, Channels},
};

/// Interface to a CTimer peripheral
///
/// Controls the CTimer.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct CTIMER<State, Channel1State, Channel2State, Channel3State> {
    /// The PWM channels of this CTIMER
    pub channels: Channels<State, Channel1State, Channel2State, Channel3State>,

    inner: CTIMER0,
    state: State,
}

impl CTIMER<Disabled, Detached, Detached, Detached> {
    pub(crate) fn new(ct: CTIMER0) -> Self {
        Self {
            channels: Channels::new(),
            inner: ct,
            state: Disabled,
        }
    }
}

impl<Channel1State, Channel2State, Channel3State>
    CTIMER<Disabled, Channel1State, Channel2State, Channel3State>
{
    /// Start the PWM timer, with a predefined period and prescaler
    ///
    /// The `period` sets resolution of the pwm and is returned with
    /// `get_max_duty`.
    pub fn enable(
        self,
        period: u32,
        prescaler: u32,
        syscon: &mut syscon::Handle,
    ) -> CTIMER<Enabled, Channel1State, Channel2State, Channel3State> {
        syscon.enable_clock(&self.inner);

        let mut self_ = CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            state: Enabled(()),
        };

        unsafe { self_.inner.pr.write(|w| w.prval().bits(prescaler)) };
        self_.set_period(period);
        self_.inner.mcr.write(|w| {
            w.mr3r().set_bit();
            // Use shadow registers for the pwm output matches
            w.mr0rl().set_bit();
            w.mr1rl().set_bit();
            w.mr2rl().set_bit()
        });

        self_.inner.pwmc.write(|w| {
            w.pwmen0().set_bit();
            w.pwmen1().set_bit();
            w.pwmen2().set_bit()
        });

        // Start the timer
        self_.inner.tcr.write(|w| w.cen().set_bit());

        self_
    }
}

impl CTIMER<Enabled, Detached, Detached, Detached> {
    /// Attach an output function to channel 1
    ///
    /// This function is only available if no output functions has been attached
    /// to channel 1.
    pub fn attach<Pin>(
        self,
        _: swm::Function<
            <Channel1 as channel::Trait>::Output,
            swm::state::Assigned<Pin>,
        >,
    ) -> CTIMER<Enabled, Attached, Detached, Detached> {
        CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            state: self.state,
        }
    }
}

impl CTIMER<Enabled, Attached, Detached, Detached> {
    /// Attach an output function to channel 2
    ///
    /// This function is only available if an output function has been attached
    /// to channel 1, but no output functions has been attached to channel 2.
    pub fn attach<Pin>(
        self,
        _: swm::Function<
            <Channel2 as channel::Trait>::Output,
            swm::state::Assigned<Pin>,
        >,
    ) -> CTIMER<Enabled, Attached, Attached, Detached> {
        CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            state: self.state,
        }
    }
}

impl CTIMER<Enabled, Attached, Attached, Detached> {
    /// Attach an output function to channel 3
    ///
    /// This function is only available if output functions have been attached
    /// to channels 1 and 2, but no output functions has been attached to
    /// channel 3.
    pub fn attach<Pin>(
        self,
        _: swm::Function<
            <Channel3 as channel::Trait>::Output,
            swm::state::Assigned<Pin>,
        >,
    ) -> CTIMER<Enabled, Attached, Attached, Attached> {
        CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            state: self.state,
        }
    }
}

impl<Channel1State, Channel2State, Channel3State>
    CTIMER<Enabled, Channel1State, Channel2State, Channel3State>
{
    /// Disable the CTIMER
    ///
    /// This method is only available, if `CTIMER` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `CTIMER` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> CTIMER<Disabled, Channel1State, Channel2State, Channel3State> {
        syscon.disable_clock(&self.inner);

        CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            state: Disabled,
        }
    }

    // Private methods

    fn get_period(&self) -> u32 {
        self.inner.mr[3].read().match_().bits()
    }

    fn get_max_duty(&self) -> u32 {
        self.get_period()
    }

    fn set_period(&mut self, period: u32) {
        // Use MAT3 to reset the counter
        unsafe { self.inner.mr[3].write(|w| w.match_().bits(period)) };

        // Reset counter. Otherwise we can run into the case where the counter
        // is already larger than period, and won't be reset until it wrapped.
        self.inner.tcr.modify(|_, w| w.crst().enabled());
        self.inner.tcr.modify(|_, w| w.crst().disabled());
    }
}

impl<State, Channel1State, Channel2State, Channel3State>
    CTIMER<State, Channel1State, Channel2State, Channel3State>
{
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
    pub fn free(self) -> CTIMER0 {
        self.inner
    }
}

impl Pwm for CTIMER<Enabled, Attached, Detached, Detached> {
    type Channel = Channels1;
    type Time = u32;
    type Duty = u32;

    fn disable(&mut self, channel: Self::Channel) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::disable(&mut self.channels.channel1)
            }
        }
    }

    fn enable(&mut self, channel: Self::Channel) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::enable(&mut self.channels.channel1)
            }
        }
    }

    fn get_period(&self) -> Self::Time {
        self.get_period()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::get_duty(&self.channels.channel1)
            }
        }
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.get_max_duty()
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::set_duty(&mut self.channels.channel1, duty)
            }
        }
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.set_period(period.into())
    }
}

impl Pwm for CTIMER<Enabled, Attached, Attached, Detached> {
    type Channel = Channels12;
    type Time = u32;
    type Duty = u32;

    fn disable(&mut self, channel: Self::Channel) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::disable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPin::disable(&mut self.channels.channel2)
            }
        }
    }

    fn enable(&mut self, channel: Self::Channel) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::enable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPin::enable(&mut self.channels.channel2)
            }
        }
    }

    fn get_period(&self) -> Self::Time {
        self.get_period()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::get_duty(&self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPin::get_duty(&self.channels.channel2)
            }
        }
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.get_max_duty()
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::set_duty(&mut self.channels.channel1, duty)
            }
            Self::Channel::Channel2 => {
                PwmPin::set_duty(&mut self.channels.channel2, duty)
            }
        }
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.set_period(period.into())
    }
}

impl Pwm for CTIMER<Enabled, Attached, Attached, Attached> {
    type Channel = Channels123;
    type Time = u32;
    type Duty = u32;

    fn disable(&mut self, channel: Self::Channel) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::disable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPin::disable(&mut self.channels.channel2)
            }
            Self::Channel::Channel3 => {
                PwmPin::disable(&mut self.channels.channel3)
            }
        }
    }

    fn enable(&mut self, channel: Self::Channel) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::enable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPin::enable(&mut self.channels.channel2)
            }
            Self::Channel::Channel3 => {
                PwmPin::enable(&mut self.channels.channel3)
            }
        }
    }

    fn get_period(&self) -> Self::Time {
        self.get_period()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::get_duty(&self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPin::get_duty(&self.channels.channel2)
            }
            Self::Channel::Channel3 => {
                PwmPin::get_duty(&self.channels.channel3)
            }
        }
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.get_max_duty()
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        match channel {
            Self::Channel::Channel1 => {
                PwmPin::set_duty(&mut self.channels.channel1, duty)
            }
            Self::Channel::Channel2 => {
                PwmPin::set_duty(&mut self.channels.channel2, duty)
            }
            Self::Channel::Channel3 => {
                PwmPin::set_duty(&mut self.channels.channel3, duty)
            }
        }
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.set_period(period.into())
    }
}

impl PwmAlpha for CTIMER<Enabled, Attached, Detached, Detached> {
    type Error = Infallible;
    type Channel = Channels1;
    type Time = u32;
    type Duty = u32;

    fn disable(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::disable(&mut self.channels.channel1)
            }
        }
    }

    fn enable(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::enable(&mut self.channels.channel1)
            }
        }
    }

    fn get_period(&self) -> Result<Self::Time, Self::Error> {
        Ok(self.get_period())
    }

    fn get_duty(
        &self,
        channel: &Self::Channel,
    ) -> Result<Self::Duty, Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::get_duty(&self.channels.channel1)
            }
        }
    }

    fn get_max_duty(&self) -> Result<Self::Duty, Self::Error> {
        Ok(self.get_max_duty())
    }

    fn set_duty(
        &mut self,
        channel: &Self::Channel,
        duty: Self::Duty,
    ) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::set_duty(&mut self.channels.channel1, duty)
            }
        }
    }

    fn set_period<P>(&mut self, period: P) -> Result<(), Self::Error>
    where
        P: Into<Self::Time>,
    {
        Ok(self.set_period(period.into()))
    }
}

impl PwmAlpha for CTIMER<Enabled, Attached, Attached, Detached> {
    type Error = Infallible;
    type Channel = Channels12;
    type Time = u32;
    type Duty = u32;

    fn disable(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::disable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::disable(&mut self.channels.channel2)
            }
        }
    }

    fn enable(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::enable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::enable(&mut self.channels.channel2)
            }
        }
    }

    fn get_period(&self) -> Result<Self::Time, Self::Error> {
        Ok(self.get_period())
    }

    fn get_duty(
        &self,
        channel: &Self::Channel,
    ) -> Result<Self::Duty, Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::get_duty(&self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::get_duty(&self.channels.channel2)
            }
        }
    }

    fn get_max_duty(&self) -> Result<Self::Duty, Self::Error> {
        Ok(self.get_max_duty())
    }

    fn set_duty(
        &mut self,
        channel: &Self::Channel,
        duty: Self::Duty,
    ) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::set_duty(&mut self.channels.channel1, duty)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::set_duty(&mut self.channels.channel2, duty)
            }
        }
    }

    fn set_period<P>(&mut self, period: P) -> Result<(), Self::Error>
    where
        P: Into<Self::Time>,
    {
        Ok(self.set_period(period.into()))
    }
}

impl PwmAlpha for CTIMER<Enabled, Attached, Attached, Attached> {
    type Error = Infallible;
    type Channel = Channels123;
    type Time = u32;
    type Duty = u32;

    fn disable(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::disable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::disable(&mut self.channels.channel2)
            }
            Self::Channel::Channel3 => {
                PwmPinAlpha::disable(&mut self.channels.channel3)
            }
        }
    }

    fn enable(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::enable(&mut self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::enable(&mut self.channels.channel2)
            }
            Self::Channel::Channel3 => {
                PwmPinAlpha::enable(&mut self.channels.channel3)
            }
        }
    }

    fn get_period(&self) -> Result<Self::Time, Self::Error> {
        Ok(self.get_period())
    }

    fn get_duty(
        &self,
        channel: &Self::Channel,
    ) -> Result<Self::Duty, Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::get_duty(&self.channels.channel1)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::get_duty(&self.channels.channel2)
            }
            Self::Channel::Channel3 => {
                PwmPinAlpha::get_duty(&self.channels.channel3)
            }
        }
    }

    fn get_max_duty(&self) -> Result<Self::Duty, Self::Error> {
        Ok(self.get_max_duty())
    }

    fn set_duty(
        &mut self,
        channel: &Self::Channel,
        duty: Self::Duty,
    ) -> Result<(), Self::Error> {
        match channel {
            Self::Channel::Channel1 => {
                PwmPinAlpha::set_duty(&mut self.channels.channel1, duty)
            }
            Self::Channel::Channel2 => {
                PwmPinAlpha::set_duty(&mut self.channels.channel2, duty)
            }
            Self::Channel::Channel3 => {
                PwmPinAlpha::set_duty(&mut self.channels.channel3, duty)
            }
        }
    }

    fn set_period<P>(&mut self, period: P) -> Result<(), Self::Error>
    where
        P: Into<Self::Time>,
    {
        Ok(self.set_period(period.into()))
    }
}

/// The available channels, if only channel 1 is attached
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channels1 {
    /// Channel 1
    Channel1,
}

/// The available channels, if only channels 1 and 2 are attached
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channels12 {
    /// Channel 1
    Channel1,

    /// Channel 2
    Channel2,
}

/// The available channels, if channels 1, 2, and 2 are attached
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channels123 {
    /// Channel 1
    Channel1,

    /// Channel 2
    Channel2,

    /// Channel 3
    Channel3,
}

impl From<Channels1> for Channels123 {
    fn from(from: Channels1) -> Self {
        match from {
            Channels1::Channel1 => Self::Channel1,
        }
    }
}

impl From<Channels12> for Channels123 {
    fn from(from: Channels12) -> Self {
        match from {
            Channels12::Channel1 => Self::Channel1,
            Channels12::Channel2 => Self::Channel2,
        }
    }
}
