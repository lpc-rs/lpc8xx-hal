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

#[cfg(feature = "845")]
pub mod frg;

#[cfg(feature = "845")]
pub use self::frg::FRG;

pub mod clock_source;

#[cfg(feature = "82x")]
use crate::pac::syscon::{
    pdruncfg, presetctrl as presetctrl0, starterp1,
    sysahbclkctrl as sysahbclkctrl0, PDRUNCFG, PRESETCTRL as PRESETCTRL0,
    STARTERP1, SYSAHBCLKCTRL as SYSAHBCLKCTRL0, UARTCLKDIV, UARTFRGDIV,
    UARTFRGMULT,
};

#[cfg(feature = "845")]
use crate::pac::syscon::{
    pdruncfg, presetctrl0, starterp1, sysahbclkctrl0, FCLKSEL, PDRUNCFG,
    PRESETCTRL0, STARTERP1, SYSAHBCLKCTRL0,
};

use crate::{clock, init_state, pac, reg_proxy::RegProxy};

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
    syscon: pac::SYSCON,
}

impl SYSCON {
    pub(crate) fn new(syscon: pac::SYSCON) -> Self {
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
                presetctrl0: RegProxy::new(),
                starterp1: RegProxy::new(),
                sysahbclkctrl: RegProxy::new(),
                #[cfg(feature = "845")]
                fclksel: RegProxy::new(),
            },

            bod: BOD(()),
            flash: FLASH(()),
            iosc: IOSC(()),
            ioscout: IOSCOUT(()),
            mtb: MTB(()),
            ram0_1: RAM0_1(()),
            rom: ROM(()),
            sysosc: SYSOSC(()),
            syspll: SYSPLL(()),

            #[cfg(feature = "82x")]
            uartfrg: UARTFRG {
                uartclkdiv: RegProxy::new(),
                uartfrgdiv: RegProxy::new(),
                uartfrgmult: RegProxy::new(),
            },

            iosc_derived_clock: IoscDerivedClock::new(),
            #[cfg(feature = "845")]
            frg0: FRG::new(),
            #[cfg(feature = "845")]
            frg1: FRG::new(),
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
    pub fn free(self) -> pac::SYSCON {
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

    /// IRC/FRO
    pub iosc: IOSC,

    /// IRC/FRO output
    pub ioscout: IOSCOUT,

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

    /// The 750 kHz internal oscillator/IRC/FRO-derived clock
    pub iosc_derived_clock: IoscDerivedClock<init_state::Enabled>,

    #[cfg(feature = "845")]
    /// Fractional Baud Rate Generator 0
    pub frg0: FRG<frg::FRG0>,

    #[cfg(feature = "845")]
    /// Fractional Baud Rate Generator 1
    pub frg1: FRG<frg::FRG1>,
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
    presetctrl0: RegProxy<PRESETCTRL0>,
    starterp1: RegProxy<STARTERP1>,
    sysahbclkctrl: RegProxy<SYSAHBCLKCTRL0>,
    #[cfg(feature = "845")]
    pub(crate) fclksel: RegProxy<FCLKSEL>,
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
        self.presetctrl0.modify(|_, w| peripheral.assert_reset(w));
    }

    /// Clear peripheral reset
    ///
    /// Clears the reset for a peripheral or other hardware component. HAL users
    /// usually won't have to call this method directly, as other peripheral
    /// APIs will do this for them.
    pub fn clear_reset<P: ResetControl>(&mut self, peripheral: &P) {
        self.presetctrl0.modify(|_, w| peripheral.clear_reset(w));
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
#[derive(Debug)]
pub struct BOD(());

/// Flash memory
///
/// Can be used to control flash memory using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct FLASH(());

/// IOSC
///
/// Can be used to control the IRC/FRO using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct IOSC(());

/// IOSC output
///
/// Can be used to control IRC/FRO output using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct IOSCOUT(());

/// Micro Trace Buffer
///
/// Can be used to control the Micro Trace Buffer using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct MTB(());

/// Random access memory
///
/// Can be used to control the RAM using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct RAM0_1(());

/// Read-only memory
///
/// Can be used to control the ROM using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct ROM(());

/// System oscillator
///
/// Can be used to control the system oscillator using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct SYSOSC(());

/// PLL
///
/// Can be used to control the PLL using various methods on [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
#[derive(Debug)]
pub struct SYSPLL(());

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
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`syscon::Handle::enable_clock`] and
/// [`syscon::Handle::disable_clock`] for the public API that uses this trait.
///
/// [`syscon::Handle::enable_clock`]: struct.Handle.html#method.enable_clock
/// [`syscon::Handle::disable_clock`]: struct.Handle.html#method.disable_clock
pub trait ClockControl {
    /// Internal method to enable a peripheral clock
    fn enable_clock<'w>(
        &self,
        w: &'w mut sysahbclkctrl0::W,
    ) -> &'w mut sysahbclkctrl0::W;

    /// Internal method to disable a peripheral clock
    fn disable_clock<'w>(
        &self,
        w: &'w mut sysahbclkctrl0::W,
    ) -> &'w mut sysahbclkctrl0::W;
}

macro_rules! impl_clock_control {
    ($clock_control:ty, $clock:ident) => {
        impl ClockControl for $clock_control {
            fn enable_clock<'w>(
                &self,
                w: &'w mut sysahbclkctrl0::W,
            ) -> &'w mut sysahbclkctrl0::W {
                w.$clock().set_bit()
            }

            fn disable_clock<'w>(
                &self,
                w: &'w mut sysahbclkctrl0::W,
            ) -> &'w mut sysahbclkctrl0::W {
                w.$clock().clear_bit()
            }
        }
    };
}

impl_clock_control!(ROM, rom);
impl_clock_control!(RAM0_1, ram0_1);
#[cfg(feature = "82x")]
impl_clock_control!(pac::FLASH_CTRL, flashreg);
#[cfg(feature = "845")]
impl_clock_control!(pac::FLASH_CTRL, flash);
impl_clock_control!(FLASH, flash);
impl_clock_control!(pac::I2C0, i2c0);
#[cfg(feature = "82x")]
impl_clock_control!(pac::GPIO, gpio);
impl_clock_control!(pac::SWM0, swm);
impl_clock_control!(pac::SCT0, sct);
impl_clock_control!(pac::WKT, wkt);
impl_clock_control!(pac::MRT0, mrt);
#[cfg(feature = "845")]
impl_clock_control!(pac::CTIMER0, ctimer);
impl_clock_control!(pac::SPI0, spi0);
impl_clock_control!(pac::SPI1, spi1);
impl_clock_control!(pac::CRC, crc);
impl_clock_control!(pac::USART0, uart0);
impl_clock_control!(pac::USART1, uart1);
impl_clock_control!(pac::USART2, uart2);
#[cfg(feature = "845")]
impl_clock_control!(pac::USART3, uart3);
#[cfg(feature = "845")]
impl_clock_control!(pac::USART4, uart4);
impl_clock_control!(pac::WWDT, wwdt);
impl_clock_control!(pac::IOCON, iocon);
impl_clock_control!(pac::ACOMP, acmp);
impl_clock_control!(pac::I2C1, i2c1);
impl_clock_control!(pac::I2C2, i2c2);
impl_clock_control!(pac::I2C3, i2c3);
impl_clock_control!(pac::ADC0, adc);
impl_clock_control!(MTB, mtb);
impl_clock_control!(pac::DMA0, dma);
#[cfg(feature = "845")]
impl_clock_control!(pac::PINT, gpio_int);

#[cfg(feature = "845")]
impl ClockControl for pac::GPIO {
    fn enable_clock<'w>(
        &self,
        w: &'w mut sysahbclkctrl0::W,
    ) -> &'w mut sysahbclkctrl0::W {
        w.gpio0().enable().gpio1().enable()
    }

    fn disable_clock<'w>(
        &self,
        w: &'w mut sysahbclkctrl0::W,
    ) -> &'w mut sysahbclkctrl0::W {
        w.gpio0().disable().gpio1().disable()
    }
}

/// Internal trait for controlling peripheral reset
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`syscon::Handle::assert_reset`] and
/// [`syscon::Handle::clear_reset`] for the public API that uses this trait.
///
/// [`syscon::Handle::assert_reset`]: struct.Handle.html#method.assert_reset
/// [`syscon::Handle::clear_reset`]: struct.Handle.html#method.clear_reset
pub trait ResetControl {
    /// Internal method to assert peripheral reset
    fn assert_reset<'w>(
        &self,
        w: &'w mut presetctrl0::W,
    ) -> &'w mut presetctrl0::W;

    /// Internal method to clear peripheral reset
    fn clear_reset<'w>(
        &self,
        w: &'w mut presetctrl0::W,
    ) -> &'w mut presetctrl0::W;
}

macro_rules! impl_reset_control {
    ($reset_control:ty, $field:ident) => {
        impl<'a> ResetControl for $reset_control {
            fn assert_reset<'w>(
                &self,
                w: &'w mut presetctrl0::W,
            ) -> &'w mut presetctrl0::W {
                w.$field().clear_bit()
            }

            fn clear_reset<'w>(
                &self,
                w: &'w mut presetctrl0::W,
            ) -> &'w mut presetctrl0::W {
                w.$field().set_bit()
            }
        }
    };
}

impl_reset_control!(pac::SPI0, spi0_rst_n);
impl_reset_control!(pac::SPI1, spi1_rst_n);
#[cfg(feature = "82x")]
impl_reset_control!(UARTFRG, uartfrg_rst_n);
impl_reset_control!(pac::USART0, uart0_rst_n);
impl_reset_control!(pac::USART1, uart1_rst_n);
impl_reset_control!(pac::USART2, uart2_rst_n);
#[cfg(feature = "845")]
impl_reset_control!(pac::USART3, uart3_rst_n);
#[cfg(feature = "845")]
impl_reset_control!(pac::USART4, uart4_rst_n);
impl_reset_control!(pac::I2C0, i2c0_rst_n);
impl_reset_control!(pac::MRT0, mrt_rst_n);
impl_reset_control!(pac::SCT0, sct_rst_n);
impl_reset_control!(pac::WKT, wkt_rst_n);
#[cfg(feature = "845")]
impl_reset_control!(pac::CTIMER0, ctimer_rst_n);
#[cfg(feature = "82x")]
impl_reset_control!(pac::GPIO, gpio_rst_n);
impl_reset_control!(pac::FLASH_CTRL, flash_rst_n);
impl_reset_control!(pac::ACOMP, acmp_rst_n);
impl_reset_control!(pac::I2C1, i2c1_rst_n);
impl_reset_control!(pac::I2C2, i2c2_rst_n);
impl_reset_control!(pac::I2C3, i2c3_rst_n);
impl_reset_control!(pac::ADC0, adc_rst_n);
impl_reset_control!(pac::DMA0, dma_rst_n);
#[cfg(feature = "845")]
impl_reset_control!(pac::PINT, gpioint_rst_n);

#[cfg(feature = "845")]
impl<'a> ResetControl for pac::GPIO {
    fn assert_reset<'w>(
        &self,
        w: &'w mut presetctrl0::W,
    ) -> &'w mut presetctrl0::W {
        w.gpio0_rst_n().clear_bit();
        w.gpio1_rst_n().clear_bit()
    }

    fn clear_reset<'w>(
        &self,
        w: &'w mut presetctrl0::W,
    ) -> &'w mut presetctrl0::W {
        w.gpio0_rst_n().set_bit();
        w.gpio1_rst_n().set_bit()
    }
}

/// Internal trait for powering analog blocks
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
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
            fn power_up<'w>(
                &self,
                w: &'w mut pdruncfg::W,
            ) -> &'w mut pdruncfg::W {
                w.$field().clear_bit()
            }

            fn power_down<'w>(
                &self,
                w: &'w mut pdruncfg::W,
            ) -> &'w mut pdruncfg::W {
                w.$field().set_bit()
            }
        }
    };
}

#[cfg(feature = "82x")]
impl_analog_block!(IOSCOUT, ircout_pd);
#[cfg(feature = "82x")]
impl_analog_block!(IOSC, irc_pd);
#[cfg(feature = "845")]
impl_analog_block!(IOSCOUT, froout_pd);
#[cfg(feature = "845")]
impl_analog_block!(IOSC, fro_pd);
impl_analog_block!(FLASH, flash_pd);
impl_analog_block!(BOD, bod_pd);
impl_analog_block!(pac::ADC0, adc_pd);
impl_analog_block!(SYSOSC, sysosc_pd);
impl_analog_block!(pac::WWDT, wdtosc_pd);
impl_analog_block!(SYSPLL, syspll_pd);
impl_analog_block!(pac::ACOMP, acmp);

/// The 750 kHz IRC/FRO-derived clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
#[derive(Debug)]
pub struct IoscDerivedClock<State = init_state::Enabled> {
    _state: State,
}

impl IoscDerivedClock<init_state::Enabled> {
    pub(crate) fn new() -> Self {
        Self {
            _state: init_state::Enabled(()),
        }
    }
}

impl IoscDerivedClock<init_state::Disabled> {
    /// Enable the IRC/FRO-derived clock
    ///
    /// This method is only available, if `IoscDerivedClock` is in the
    /// [`Disabled`] state. Code that attempts to call this method when the
    /// clock is already enabled will not compile.
    ///
    /// Consumes this instance of `IoscDerivedClock` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`]. That new
    /// instance implements [`clock::Enabled`], which might be required by APIs
    /// that need an enabled clock.
    ///
    /// Also consumes the handles to [`IOSC`] and [`IOSCOUT`], to make it
    /// impossible (outside of unsafe code) to break API guarantees.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(
        self,
        syscon: &mut Handle,
        iosc: IOSC,
        ioscout: IOSCOUT,
    ) -> IoscDerivedClock<init_state::Enabled> {
        syscon.power_up(&iosc);
        syscon.power_up(&ioscout);

        IoscDerivedClock {
            _state: init_state::Enabled(()),
        }
    }
}

impl<State> clock::Frequency for IoscDerivedClock<State> {
    fn hz(&self) -> u32 {
        750_000
    }
}

impl clock::Enabled for IoscDerivedClock<init_state::Enabled> {}

/// Internal trait used to configure interrupt wake-up
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
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

reg!(PDRUNCFG, PDRUNCFG, pac::SYSCON, pdruncfg);
#[cfg(feature = "82x")]
reg!(PRESETCTRL0, PRESETCTRL0, pac::SYSCON, presetctrl);
#[cfg(feature = "845")]
reg!(PRESETCTRL0, PRESETCTRL0, pac::SYSCON, presetctrl0);
reg!(STARTERP1, STARTERP1, pac::SYSCON, starterp1);
#[cfg(feature = "82x")]
reg!(SYSAHBCLKCTRL0, SYSAHBCLKCTRL0, pac::SYSCON, sysahbclkctrl);
#[cfg(feature = "845")]
reg!(SYSAHBCLKCTRL0, SYSAHBCLKCTRL0, pac::SYSCON, sysahbclkctrl0);
#[cfg(feature = "845")]
reg!(FCLKSEL, [FCLKSEL; 11], pac::SYSCON, fclksel);

#[cfg(feature = "82x")]
reg!(UARTCLKDIV, UARTCLKDIV, pac::SYSCON, uartclkdiv);
#[cfg(feature = "82x")]
reg!(UARTFRGDIV, UARTFRGDIV, pac::SYSCON, uartfrgdiv);
#[cfg(feature = "82x")]
reg!(UARTFRGMULT, UARTFRGMULT, pac::SYSCON, uartfrgmult);
