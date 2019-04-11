//! API for system configuration (SYSCON)
//!
//! The entry point to this API is [`SYSCON`]. Please refer to [`SYSCON`]'s
//! documentation for additional information.
//!
//! This module mostly provides infrastructure required by other parts of the
//! HAL API. For this reason, only a small subset of SYSCON functionality is
//! currently implemented.
//!
//! The SYSCON peripheral is described in the user manual, chapter 5.

use core::marker::PhantomData;

#[cfg(feature = "82x")]
use crate::raw::syscon::{
    pdruncfg, presetctrl, starterp1, sysahbclkctrl, PDRUNCFG, PRESETCTRL, STARTERP1, SYSAHBCLKCTRL,
    UARTCLKDIV, UARTFRGDIV, UARTFRGMULT,
};

#[cfg(feature = "845")]
use crate::raw::syscon::{
    pdruncfg, presetctrl0 as presetctrl, starterp1, sysahbclkctrl0 as sysahbclkctrl, PDRUNCFG,
    PRESETCTRL0 as PRESETCTRL, STARTERP1, SYSAHBCLKCTRL0 as SYSAHBCLKCTRL,
};

use crate::reg;
// TODO Remove when FRO is implemented for lpc845
#[allow(unused_imports)]
use crate::{clock, init_state, raw, raw_compat, reg_proxy::RegProxy};

/// Entry point to the SYSCON API
///
/// The SYSCON API is split into multiple parts, which are all available through
/// [`syscon::Parts`]. You can use [`SYSCON::split`] to gain access to
/// [`syscon::Parts`].
///
/// You can also use this struct to gain access to the raw peripheral using
/// [`SYSCON::free`]. This is the main reason this struct exists, as it's no
/// longer possible to do this after the API has been split.
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`syscon::Parts`]: struct.Parts.html
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct SYSCON {
    syscon: raw::SYSCON,
}

impl SYSCON {
    pub fn new(syscon: raw::SYSCON) -> Self {
        SYSCON { syscon }
    }

    /// Splits the SYSCON API into its component parts
    ///
    /// This is the regular way to access the SYSCON API. It exists as an
    /// explicit step, as it's no longer possible to gain access to the raw
    /// peripheral using [`SYSCON::free`] after you've called this method.
    pub fn split(self) -> Parts {
        Parts {
            handle: Handle {
                pdruncfg: RegProxy::new(),
                presetctrl: RegProxy::new(),
                starterp1: RegProxy::new(),
                sysahbclkctrl: RegProxy::new(),
            },

            bod: BOD(PhantomData),
            flash: FLASH(PhantomData),
            irc: IRC(PhantomData),
            ircout: IRCOUT(PhantomData),
            mtb: MTB(PhantomData),
            ram0_1: RAM0_1(PhantomData),
            rom: ROM(PhantomData),
            sysosc: SYSOSC(PhantomData),
            syspll: SYSPLL(PhantomData),

            #[cfg(feature = "82x")]
            uartfrg: UARTFRG {
                uartclkdiv: RegProxy::new(),
                uartfrgdiv: RegProxy::new(),
                uartfrgmult: RegProxy::new(),
            },

            #[cfg(feature = "82x")]
            irc_derived_clock: IrcDerivedClock::new(),
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
    pub fn free(self) -> raw::SYSCON {
        self.syscon
    }
}

/// The main API for the SYSCON peripheral
///
/// Provides access to all types that make up the SYSCON API. Please refer to
/// the [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts {
    /// The handle to the SYSCON peripheral
    pub handle: Handle,

    /// Brown-out detection
    pub bod: BOD,

    /// Flash memory
    pub flash: FLASH,

    /// IRC
    pub irc: IRC,

    /// IRC output
    pub ircout: IRCOUT,

    /// Micro Trace Buffer
    pub mtb: MTB,

    /// Random access memory
    pub ram0_1: RAM0_1,

    /// Read-only memory
    pub rom: ROM,

    /// System oscillator
    pub sysosc: SYSOSC,

    /// PLL
    pub syspll: SYSPLL,

    #[cfg(feature = "82x")]
    /// UART Fractional Baud Rate Generator
    pub uartfrg: UARTFRG,

    #[cfg(feature = "82x")]
    /// The 750 kHz IRC-derived clock
    pub irc_derived_clock: IrcDerivedClock<init_state::Enabled>,
}

/// Handle to the SYSCON peripheral
///
/// This handle to the SYSCON peripheral provides access to the main part of the
/// SYSCON API. It is also required by other parts of the HAL API to synchronize
/// access the the underlying registers, wherever this is required.
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [module documentation]: index.html
pub struct Handle {
    pdruncfg: RegProxy<PDRUNCFG>,
    presetctrl: RegProxy<PRESETCTRL>,
    starterp1: RegProxy<STARTERP1>,
    sysahbclkctrl: RegProxy<SYSAHBCLKCTRL>,
}

impl Handle {
    /// Enable peripheral clock
    ///
    /// Enables the clock for a peripheral or other hardware component. HAL
    /// users usually won't have to call this method directly, as other
    /// peripheral APIs will do this for them.
    pub fn enable_clock<P: ClockControl>(&mut self, peripheral: &P) {
        self.sysahbclkctrl.modify(|_, w| peripheral.enable_clock(w));
    }

    /// Disable peripheral clock
    pub fn disable_clock<P: ClockControl>(&mut self, peripheral: &P) {
        self.sysahbclkctrl
            .modify(|_, w| peripheral.disable_clock(w));
    }

    /// Assert peripheral reset
    pub fn assert_reset<P: ResetControl>(&mut self, peripheral: &P) {
        self.presetctrl.modify(|_, w| peripheral.assert_reset(w));
    }

    /// Clear peripheral reset
    ///
    /// Clears the reset for a peripheral or other hardware component. HAL users
    /// usually won't have to call this method directly, as other peripheral
    /// APIs will do this for them.
    pub fn clear_reset<P: ResetControl>(&mut self, peripheral: &P) {
        self.presetctrl.modify(|_, w| peripheral.clear_reset(w));
    }

    /// Provide power to an analog block
    ///
    /// HAL users usually won't have to call this method themselves, as other
    /// peripheral APIs will do this for them.
    pub fn power_up<P: AnalogBlock>(&mut self, peripheral: &P) {
        self.pdruncfg.modify(|_, w| peripheral.power_up(w));
    }

    /// Remove power from an analog block
    pub fn power_down<P: AnalogBlock>(&mut self, peripheral: &P) {
        self.pdruncfg.modify(|_, w| peripheral.power_down(w));
    }

    /// Enable interrupt wake-up from deep-sleep and power-down modes
    ///
    /// To use an interrupt for waking up the system from the deep-sleep and
    /// power-down modes, it needs to be enabled using this method, in addition
    /// to being enabled in the NVIC.
    ///
    /// This method is not required when using the regular sleep mode.
    pub fn enable_interrupt_wakeup<I>(&mut self)
    where
        I: WakeUpInterrupt,
    {
        self.starterp1.modify(|_, w| I::enable(w));
    }

    /// Disable interrupt wake-up from deep-sleep and power-down modes
    pub fn disable_interrupt_wakeup<I>(&mut self)
    where
        I: WakeUpInterrupt,
    {
        self.starterp1.modify(|_, w| I::disable(w));
    }
}

/// Brown-out detection
///
/// Can be used to control brown-out detection using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct BOD(PhantomData<*const ()>);

/// Flash memory
///
/// Can be used to control flash memory using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct FLASH(PhantomData<*const ()>);

/// IRC
///
/// Can be used to control the IRC using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct IRC(PhantomData<*const ()>);

/// IRC output
///
/// Can be used to control IRC output using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct IRCOUT(PhantomData<*const ()>);

/// Micro Trace Buffer
///
/// Can be used to control the Micro Trace Buffer using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct MTB(PhantomData<*const ()>);

/// Random access memory
///
/// Can be used to control the RAM using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[allow(non_camel_case_types)]
pub struct RAM0_1(PhantomData<*const ()>);

/// Read-only memory
///
/// Can be used to control the ROM using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct ROM(PhantomData<*const ()>);

/// System oscillator
///
/// Can be used to control the system oscillator using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct SYSOSC(PhantomData<*const ()>);

/// PLL
///
/// Can be used to control the PLL using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct SYSPLL(PhantomData<*const ()>);

#[cfg(feature = "82x")]
/// UART Fractional Baud Rate Generator
///
/// Controls the common clock for all UART peripherals (U_PCLK).
///
/// Can also be used to control the UART FRG using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct UARTFRG {
    uartclkdiv: RegProxy<UARTCLKDIV>,
    uartfrgdiv: RegProxy<UARTFRGDIV>,
    uartfrgmult: RegProxy<UARTFRGMULT>,
}

#[cfg(feature = "82x")]
impl UARTFRG {
    /// Set UART clock divider value (UARTCLKDIV)
    ///
    /// See user manual, section 5.6.15.
    pub fn set_clkdiv(&mut self, value: u8) {
        self.uartclkdiv.write(|w| unsafe { w.div().bits(value) });
    }

    /// Set UART fractional generator multiplier value (UARTFRGMULT)
    ///
    /// See user manual, section 5.6.20.
    pub fn set_frgmult(&mut self, value: u8) {
        self.uartfrgmult.write(|w| unsafe { w.mult().bits(value) });
    }

    /// Set UART fractional generator divider value (UARTFRGDIV)
    ///
    /// See user manual, section 5.6.19.
    pub fn set_frgdiv(&mut self, value: u8) {
        self.uartfrgdiv.write(|w| unsafe { w.div().bits(value) });
    }
}

/// Internal trait for controlling peripheral clocks
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::enable_clock`] and
/// [`syscon::Handle::disable_clock`] for the public API that uses this trait.
///
/// [`syscon::Handle::enable_clock`]: struct.Handle.html#method.enable_clock
/// [`syscon::Handle::disable_clock`]: struct.Handle.html#method.disable_clock
pub trait ClockControl {
    /// Internal method to enable a peripheral clock
    fn enable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W;

    /// Internal method to disable a peripheral clock
    fn disable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W;
}

macro_rules! impl_clock_control {
    ($clock_control:ty, $clock:ident) => {
        impl ClockControl for $clock_control {
            fn enable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
                w.$clock().enable()
            }

            fn disable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
                w.$clock().disable()
            }
        }
    };
}

impl_clock_control!(ROM, rom);
impl_clock_control!(RAM0_1, ram0_1);
#[cfg(feature = "82x")]
impl_clock_control!(raw_compat::FLASH_CTRL, flashreg);
#[cfg(feature = "845")]
impl_clock_control!(raw_compat::FLASH_CTRL, flash);
impl_clock_control!(FLASH, flash);
impl_clock_control!(raw::I2C0, i2c0);
#[cfg(feature = "82x")]
impl_clock_control!(raw_compat::GPIO, gpio);
impl_clock_control!(raw_compat::SWM0, swm);
impl_clock_control!(raw_compat::SCT0, sct);
impl_clock_control!(raw::WKT, wkt);
impl_clock_control!(raw_compat::MRT0, mrt);
impl_clock_control!(raw::SPI0, spi0);
impl_clock_control!(raw::SPI1, spi1);
impl_clock_control!(raw::CRC, crc);
impl_clock_control!(raw::USART0, uart0);
impl_clock_control!(raw::USART1, uart1);
impl_clock_control!(raw::USART2, uart2);
impl_clock_control!(raw::WWDT, wwdt);
impl_clock_control!(raw::IOCON, iocon);
impl_clock_control!(raw_compat::ACOMP, acmp);
impl_clock_control!(raw::I2C1, i2c1);
impl_clock_control!(raw::I2C2, i2c2);
impl_clock_control!(raw::I2C3, i2c3);
impl_clock_control!(raw_compat::ADC0, adc);
impl_clock_control!(MTB, mtb);
impl_clock_control!(raw_compat::DMA0, dma);
#[cfg(feature = "845")]
impl ClockControl for raw_compat::GPIO {
    fn enable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
        w.gpio0().enable().gpio1().enable()
    }

    fn disable_clock<'w>(&self, w: &'w mut sysahbclkctrl::W) -> &'w mut sysahbclkctrl::W {
        w.gpio0().disable().gpio1().disable()
    }
}

/// Internal trait for controlling peripheral reset
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`syscon::Handle::assert_reset`] and
/// [`syscon::Handle::clear_reset`] for the public API that uses this trait.
///
/// [`syscon::Handle::assert_reset`]: struct.Handle.html#method.assert_reset
/// [`syscon::Handle::clear_reset`]: struct.Handle.html#method.clear_reset
pub trait ResetControl {
    /// Internal method to assert peripheral reset
    fn assert_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W;

    /// Internal method to clear peripheral reset
    fn clear_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W;
}

macro_rules! impl_reset_control {
    ($reset_control:ty, $field:ident) => {
        impl<'a> ResetControl for $reset_control {
            fn assert_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
                w.$field().clear_bit()
            }

            fn clear_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
                w.$field().set_bit()
            }
        }
    };
}

impl_reset_control!(raw::SPI0, spi0_rst_n);
impl_reset_control!(raw::SPI1, spi1_rst_n);
#[cfg(feature = "82x")]
impl_reset_control!(UARTFRG, uartfrg_rst_n);
impl_reset_control!(raw::USART0, uart0_rst_n);
impl_reset_control!(raw::USART1, uart1_rst_n);
impl_reset_control!(raw::USART2, uart2_rst_n);
impl_reset_control!(raw::I2C0, i2c0_rst_n);
impl_reset_control!(raw_compat::MRT0, mrt_rst_n);
impl_reset_control!(raw_compat::SCT0, sct_rst_n);
impl_reset_control!(raw::WKT, wkt_rst_n);
#[cfg(feature = "82x")]
impl_reset_control!(raw_compat::GPIO, gpio_rst_n);
impl_reset_control!(raw_compat::FLASH_CTRL, flash_rst_n);
impl_reset_control!(raw_compat::ACOMP, acmp_rst_n);
impl_reset_control!(raw::I2C1, i2c1_rst_n);
impl_reset_control!(raw::I2C2, i2c2_rst_n);
impl_reset_control!(raw::I2C3, i2c3_rst_n);
impl_reset_control!(raw_compat::ADC0, adc_rst_n);
impl_reset_control!(raw_compat::DMA0, dma_rst_n);

#[cfg(feature = "845")]
impl<'a> ResetControl for raw_compat::GPIO {
    fn assert_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
        w.gpio0_rst_n().clear_bit().gpio1_rst_n().clear_bit()
    }

    fn clear_reset<'w>(&self, w: &'w mut presetctrl::W) -> &'w mut presetctrl::W {
        w.gpio0_rst_n().set_bit().gpio1_rst_n().set_bit()
    }
}

/// Internal trait for powering analog blocks
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::power_up`] and
/// [`syscon::Handle::power_down`] for the public API that uses this trait.
///
/// [`syscon::Handle::power_up`]: struct.Handle.html#method.power_up
/// [`syscon::Handle::power_down`]: struct.Handle.html#method.power_down
pub trait AnalogBlock {
    /// Internal method to power up an analog block
    fn power_up<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W;

    /// Internal method to power down an analog block
    fn power_down<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W;
}

macro_rules! impl_analog_block {
    ($analog_block:ty, $field:ident) => {
        impl<'a> AnalogBlock for $analog_block {
            fn power_up<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W {
                w.$field().clear_bit()
            }

            fn power_down<'w>(&self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W {
                w.$field().set_bit()
            }
        }
    };
}

#[cfg(feature = "82x")]
impl_analog_block!(IRCOUT, ircout_pd);
#[cfg(feature = "82x")]
impl_analog_block!(IRC, irc_pd);
impl_analog_block!(FLASH, flash_pd);
impl_analog_block!(BOD, bod_pd);
impl_analog_block!(raw_compat::ADC0, adc_pd);
impl_analog_block!(SYSOSC, sysosc_pd);
impl_analog_block!(raw::WWDT, wdtosc_pd);
impl_analog_block!(SYSPLL, syspll_pd);
impl_analog_block!(raw_compat::ACOMP, acmp);

#[cfg(feature = "82x")]
/// The 750 kHz IRC-derived clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
pub struct IrcDerivedClock<State = init_state::Enabled> {
    _state: State,
}

#[cfg(feature = "82x")]
impl IrcDerivedClock<init_state::Enabled> {
    pub(crate) fn new() -> Self {
        IrcDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

#[cfg(feature = "82x")]
impl IrcDerivedClock<init_state::Disabled> {
    /// Enable the IRC-derived clock
    ///
    /// This method is only available, if `IrcDerivedClock` is in the
    /// [`Disabled`] state. Code that attempts to call this method when the
    /// clock is already enabled will not compile.
    ///
    /// Consumes this instance of `IrcDerivedClock` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`]. That new
    /// instance implements [`clock::Enabled`], which might be required by APIs
    /// that need an enabled clock.
    ///
    /// Also consumes the handles to [`IRC`] and [`IRCOUT`], to make it
    /// impossible (outside of unsafe code) to break API guarantees.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(
        self,
        syscon: &mut Handle,
        mut irc: IRC,
        mut ircout: IRCOUT,
    ) -> IrcDerivedClock<init_state::Enabled> {
        syscon.power_up(&mut irc);
        syscon.power_up(&mut ircout);

        IrcDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

#[cfg(feature = "82x")]
impl<State> clock::Frequency for IrcDerivedClock<State> {
    fn hz(&self) -> u32 {
        750_000
    }
}

#[cfg(feature = "82x")]
impl clock::Enabled for IrcDerivedClock<init_state::Enabled> {}

/// Internal trait used to configure interrupt wake-up
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::enable_interrupt_wakeup`] and
/// [`syscon::Handle::disable_interrupt_wakeup`] for the public API that uses
/// this trait.
///
/// [`syscon::Handle::enable_interrupt_wakeup`]: struct.Handle.html#method.enable_interrupt_wakeup
/// [`syscon::Handle::disable_interrupt_wakeup`]: struct.Handle.html#method.disable_interrupt_wakeup
pub trait WakeUpInterrupt {
    /// Internal method to configure interrupt wakeup behavior
    fn enable(w: &mut starterp1::W) -> &mut starterp1::W;

    /// Internal method to configure interrupt wakeup behavior
    fn disable(w: &mut starterp1::W) -> &mut starterp1::W;
}

macro_rules! wakeup_interrupt {
    ($name:ident, $field:ident) => {
        /// Can be used to enable/disable interrupt wake-up behavior
        ///
        /// See [`syscon::Handle::enable_interrupt_wakeup`] and
        /// [`syscon::Handle::disable_interrupt_wakeup`].
        ///
        /// [`syscon::Handle::enable_interrupt_wakeup`]: struct.Handle.html#method.enable_interrupt_wakeup
        /// [`syscon::Handle::disable_interrupt_wakeup`]: struct.Handle.html#method.disable_interrupt_wakeup
        pub struct $name;

        impl WakeUpInterrupt for $name {
            fn enable(w: &mut starterp1::W) -> &mut starterp1::W {
                w.$field().enabled()
            }

            fn disable(w: &mut starterp1::W) -> &mut starterp1::W {
                w.$field().disabled()
            }
        }
    };
}

wakeup_interrupt!(Spi0Wakeup, spi0);
wakeup_interrupt!(Spi1Wakeup, spi1);
wakeup_interrupt!(Usart0Wakeup, usart0);
wakeup_interrupt!(Usart1Wakeup, usart1);
wakeup_interrupt!(Usart2Wakeup, usart2);
wakeup_interrupt!(I2c1Wakeup, i2c1);
wakeup_interrupt!(I2c0Wakeup, i2c0);
wakeup_interrupt!(WwdtWakeup, wwdt);
wakeup_interrupt!(BodWakeup, bod);
wakeup_interrupt!(WktWakeup, wkt);
wakeup_interrupt!(I2c2Wakeup, i2c2);
wakeup_interrupt!(I2c3Wakeup, i2c3);

reg!(PDRUNCFG, PDRUNCFG, raw::SYSCON, pdruncfg);
#[cfg(feature = "82x")]
reg!(PRESETCTRL, PRESETCTRL, raw::SYSCON, presetctrl);
#[cfg(feature = "845")]
reg!(PRESETCTRL, PRESETCTRL, raw::SYSCON, presetctrl0);
reg!(STARTERP1, STARTERP1, raw::SYSCON, starterp1);
#[cfg(feature = "82x")]
reg!(SYSAHBCLKCTRL, SYSAHBCLKCTRL, raw::SYSCON, sysahbclkctrl);
#[cfg(feature = "845")]
reg!(SYSAHBCLKCTRL, SYSAHBCLKCTRL, raw::SYSCON, sysahbclkctrl0);

#[cfg(feature = "82x")]
reg!(UARTCLKDIV, UARTCLKDIV, raw::SYSCON, uartclkdiv);
#[cfg(feature = "82x")]
reg!(UARTFRGDIV, UARTFRGDIV, raw::SYSCON, uartfrgdiv);
#[cfg(feature = "82x")]
reg!(UARTFRGMULT, UARTFRGMULT, raw::SYSCON, uartfrgmult);
