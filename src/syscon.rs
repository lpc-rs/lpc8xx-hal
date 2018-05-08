//! API for system configuration (SYSCON)
//!
//! The SYSCON peripheral is described in the user manual, chapter 5.
//!
//! [`lpc82x::SYSCON`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.SYSCON.html


use core::marker::PhantomData;

use clock;
use init_state::{
    self,
    InitState,
};
use raw;
use raw::syscon::{
    pdruncfg,
    presetctrl,
    sysahbclkctrl,
    PDRUNCFG,
    PRESETCTRL,
    SYSAHBCLKCTRL,
    UARTCLKDIV,
    UARTFRGDIV,
    UARTFRGMULT,
};


/// Entry point to the SYSCON API.
///
/// Provides access to all types that make up the SYSCON API.
pub struct SYSCON<'syscon> {
    /// The handle to the SYSCON peripheral
    pub handle: Handle<'syscon>,

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

    /// UART Fractional Baud Rate Generator
    pub uartfrg: UARTFRG<'syscon>,

    /// The 750 kHz IRC-derived clock
    ///
    /// # Limitations
    ///
    /// This field currently assumes that the IRC-derived clock always starts
    /// out being disabled, but since we don't know what happened before the HAL
    /// API was initialized, this might not be the case. Please make sure you
    /// haven't enabled the IRC-derived clock, or called any code that might
    /// have, before using this field.
    pub irc_derived_clock: IrcDerivedClock<init_state::Disabled>,
}

impl<'syscon> SYSCON<'syscon> {
    /// Create an instance of `SYSCON`
    pub fn new(syscon: &'syscon mut raw::SYSCON) -> Self {
        SYSCON {
            handle: Handle {
                pdruncfg     : &syscon.pdruncfg,
                presetctrl   : &syscon.presetctrl,
                sysahbclkctrl: &syscon.sysahbclkctrl,
            },

            bod    : BOD(PhantomData),
            flash  : FLASH(PhantomData),
            irc    : IRC(PhantomData),
            ircout : IRCOUT(PhantomData),
            mtb    : MTB(PhantomData),
            ram0_1 : RAM0_1(PhantomData),
            rom    : ROM(PhantomData),
            sysosc : SYSOSC(PhantomData),
            syspll : SYSPLL(PhantomData),
            uartfrg: UARTFRG {
                uartclkdiv : &syscon.uartclkdiv,
                uartfrgdiv : &syscon.uartfrgdiv,
                uartfrgmult: &syscon.uartfrgmult,

            },

            irc_derived_clock: IrcDerivedClock::new(),
        }
    }
}


/// The handle to the SYSCON peripheral
pub struct Handle<'syscon> {
    pdruncfg     : &'syscon PDRUNCFG,
    presetctrl   : &'syscon PRESETCTRL,
    sysahbclkctrl: &'syscon SYSAHBCLKCTRL,
}

impl<'r> Handle<'r> {
    /// Enable peripheral clock
    ///
    /// Enables the clock for a peripheral or other hardware component. HAL
    /// users usually won't have to call this method directly, as other
    /// peripheral APIs will do this for them.
    pub fn enable_clock<P: ClockControl>(&mut self, peripheral: &mut P) {
        self.sysahbclkctrl.modify(|_, w| peripheral.enable_clock(w));
    }

    /// Disable peripheral clock
    pub fn disable_clock<P: ClockControl>(&mut self, peripheral: &mut P) {
        self.sysahbclkctrl.modify(|_, w| peripheral.disable_clock(w));
    }

    /// Assert peripheral reset
    pub fn assert_reset<P: ResetControl>(&mut self, peripheral: &mut P) {
        self.presetctrl.modify(|_, w| peripheral.assert_reset(w));
    }

    /// Clear peripheral reset
    ///
    /// Clears the reset for a peripheral or other hardware component. HAL users
    /// usually won't have to call this method directly, as other peripheral
    /// APIs will do this for them.
    pub fn clear_reset<P: ResetControl>(&mut self, peripheral: &mut P) {
        self.presetctrl.modify(|_, w| peripheral.clear_reset(w));
    }

    /// Provide power to an analog block
    ///
    /// HAL users usually won't have to call this method themselves, as other
    /// peripheral APIs will do this for them.
    pub fn power_up<P: AnalogBlock>(&mut self, peripheral: &mut P) {
        self.pdruncfg.modify(|_, w| peripheral.power_up(w));
    }

    /// Remove power from an analog block
    pub fn power_down<P: AnalogBlock>(&mut self, peripheral: &mut P) {
        self.pdruncfg.modify(|_, w| peripheral.power_down(w));
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

/// UART Fractional Baud Rate Generator
///
/// Controls the common clock for all UART peripherals (U_PCLK).
///
/// Can also be used to control the UART FRG using various methods on
/// [`syscon::Handle`].
///
/// [`syscon::Handle`]: struct.Handle.html
pub struct UARTFRG<'syscon> {
    uartclkdiv : &'syscon UARTCLKDIV,
    uartfrgdiv : &'syscon UARTFRGDIV,
    uartfrgmult: &'syscon UARTFRGMULT,
}

impl<'syscon> UARTFRG<'syscon> {
    /// Set UART clock divider value (UARTCLKDIV)
    ///
    /// See user manual, section 5.6.15.
    pub fn set_clkdiv(&mut self, value: u8) {
        self.uartclkdiv.write(|w|
            unsafe { w.div().bits(value) }
        );
    }

    /// Set UART fractional generator multiplier value (UARTFRGMULT)
    ///
    /// See user manual, section 5.6.20.
    pub fn set_frgmult(&mut self, value: u8) {
        self.uartfrgmult.write(|w|
            unsafe { w.mult().bits(value) }
        );
    }

    /// Set UART fractional generator divider value (UARTFRGDIV)
    ///
    /// See user manual, section 5.6.19.
    pub fn set_frgdiv(&mut self, value: u8) {
        self.uartfrgdiv.write(|w|
            unsafe { w.div().bits(value) }
        );
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
    fn enable_clock<'w>(&mut self, w: &'w mut sysahbclkctrl::W)
        -> &'w mut sysahbclkctrl::W;

    /// Internal method to disable a peripheral clock
    fn disable_clock<'w>(&mut self, w: &'w mut sysahbclkctrl::W)
        -> &'w mut sysahbclkctrl::W;
}

macro_rules! impl_clock_control {
    ($clock_control:ty, $clock:ident) => {
        impl ClockControl for $clock_control {
            fn enable_clock<'w>(&mut self, w: &'w mut sysahbclkctrl::W)
                -> &'w mut sysahbclkctrl::W
            {
                w.$clock().enable()
            }

            fn disable_clock<'w>(&mut self, w: &'w mut sysahbclkctrl::W)
                -> &'w mut sysahbclkctrl::W
            {
                w.$clock().disable()
            }
        }
    }
}

impl_clock_control!(ROM           , rom     );
impl_clock_control!(RAM0_1        , ram0_1  );
impl_clock_control!(raw::FLASHCTRL, flashreg);
impl_clock_control!(FLASH         , flash   );
impl_clock_control!(raw::I2C0     , i2c0    );
impl_clock_control!(raw::GPIO_PORT, gpio    );
impl_clock_control!(raw::SWM      , swm     );
impl_clock_control!(raw::SCT      , sct     );
impl_clock_control!(raw::WKT      , wkt     );
impl_clock_control!(raw::MRT      , mrt     );
impl_clock_control!(raw::SPI0     , spi0    );
impl_clock_control!(raw::SPI1     , spi1    );
impl_clock_control!(raw::CRC      , crc     );
impl_clock_control!(raw::USART0   , uart0   );
impl_clock_control!(raw::USART1   , uart1   );
impl_clock_control!(raw::USART2   , uart2   );
impl_clock_control!(raw::WWDT     , wwdt    );
impl_clock_control!(raw::IOCON    , iocon   );
impl_clock_control!(raw::CMP      , acmp    );
impl_clock_control!(raw::I2C1     , i2c1    );
impl_clock_control!(raw::I2C2     , i2c2    );
impl_clock_control!(raw::I2C3     , i2c3    );
impl_clock_control!(raw::ADC      , adc     );
impl_clock_control!(MTB           , mtb     );
impl_clock_control!(raw::DMA      , dma     );


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
    fn assert_reset<'w>(&mut self, w: &'w mut presetctrl::W)
        -> &'w mut presetctrl::W;

    /// Internal method to clear peripheral reset
    fn clear_reset<'w>(&mut self, w: &'w mut presetctrl::W)
        -> &'w mut presetctrl::W;
}

macro_rules! impl_reset_control {
    ($reset_control:ty, $field:ident) => {
        impl<'a> ResetControl for $reset_control {
            fn assert_reset<'w>(&mut self, w: &'w mut presetctrl::W)
                -> &'w mut presetctrl::W
            {
                w.$field().clear_bit()
            }

            fn clear_reset<'w>(&mut self, w: &'w mut presetctrl::W)
                -> &'w mut presetctrl::W
            {
                w.$field().set_bit()
            }
        }
    }
}

impl_reset_control!(raw::SPI0     , spi0_rst_n   );
impl_reset_control!(raw::SPI1     , spi1_rst_n   );
impl_reset_control!(UARTFRG<'a>   , uartfrg_rst_n);
impl_reset_control!(raw::USART0   , uart0_rst_n  );
impl_reset_control!(raw::USART1   , uart1_rst_n  );
impl_reset_control!(raw::USART2   , uart2_rst_n  );
impl_reset_control!(raw::I2C0     , i2c0_rst_n   );
impl_reset_control!(raw::MRT      , mrt_rst_n    );
impl_reset_control!(raw::SCT      , sct_rst_n    );
impl_reset_control!(raw::WKT      , wkt_rst_n    );
impl_reset_control!(raw::GPIO_PORT, gpio_rst_n   );
impl_reset_control!(raw::FLASHCTRL, flash_rst_n  );
impl_reset_control!(raw::CMP      , acmp_rst_n   );
impl_reset_control!(raw::I2C1     , i2c1_rst_n   );
impl_reset_control!(raw::I2C2     , i2c2_rst_n   );
impl_reset_control!(raw::I2C3     , i2c3_rst_n   );
impl_reset_control!(raw::ADC      , adc_rst_n    );
impl_reset_control!(raw::DMA      , dma_rst_n    );


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
    fn power_up<'w>(&mut self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W;

    /// Internal method to power down an analog block
    fn power_down<'w>(&mut self, w: &'w mut pdruncfg::W) -> &'w mut pdruncfg::W;
}

macro_rules! impl_analog_block {
    ($analog_block:ty, $field:ident) => {
        impl<'a> AnalogBlock for $analog_block {
            fn power_up<'w>(&mut self, w: &'w mut pdruncfg::W)
                -> &'w mut pdruncfg::W
            {
                w.$field().powered()
            }

            fn power_down<'w>(&mut self, w: &'w mut pdruncfg::W)
                -> &'w mut pdruncfg::W
            {
                w.$field().powered_down()
            }
        }
    }
}

impl_analog_block!(IRCOUT       , ircout_pd );
impl_analog_block!(IRC          , irc_pd    );
impl_analog_block!(FLASH        , flash_pd  );
impl_analog_block!(BOD          , bod_pd    );
impl_analog_block!(&'a raw::ADC , adc_pd    );
impl_analog_block!(SYSOSC       , sysosc_pd );
impl_analog_block!(&'a raw::WWDT, wdtosc_pd );
impl_analog_block!(SYSPLL       , syspll_pd );
impl_analog_block!(&'a raw::CMP , acmp      );


/// The 750 kHz IRC-derived clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
pub struct IrcDerivedClock<State: InitState = init_state::Enabled> {
    _state: State,
}

impl IrcDerivedClock<init_state::Disabled> {
    pub(crate) fn new() -> Self {
        IrcDerivedClock {
            _state: init_state::Disabled,
        }
    }
}

impl<State> IrcDerivedClock<State> where State: init_state::NotEnabled {
    /// Enable the IRC-derived clock
    ///
    /// This method is only available if the IRC-derived clock is not already
    /// enabled. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this instance of `IrcDerivedClock` and returns a new instance
    /// whose state indicates that the clock is enabled. That new instance
    /// implements [`clock::Enabled`], which might be required by APIs that need
    /// an enabled clock.
    ///
    /// Also consumes the handles to IRC and IRCOUT, to make it impossible
    /// (outside of unsafe code) to break API guarantees by disabling the
    /// IRC-derived clock again.
    ///
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, syscon: &mut Handle, mut irc: IRC, mut ircout: IRCOUT)
        -> IrcDerivedClock<init_state::Enabled>
    {
        syscon.power_up(&mut irc);
        syscon.power_up(&mut ircout);

        IrcDerivedClock {
            _state: init_state::Enabled,
        }
    }
}

impl<State> clock::Frequency for IrcDerivedClock<State>
    where State: InitState
{
    fn hz(&self) -> u32 { 750_000 }
}

impl clock::Enabled for IrcDerivedClock<init_state::Enabled> {}
