//! APIs for system configuration (SYSCON)
//!
//! See user manual, chapter 5.


use core::marker::PhantomData;

use lpc82x;
use lpc82x::syscon::{
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

use clock;
use clock::state::ClockState;


/// Interface to system configuration (SYSCON)
///
/// This API expects to be the sole owner of the SYSCON interface. Don't use
/// [`lpc82x::SYSCON`] directly, unless you know what you're doing.
///
/// [`lpc82x::SYSCON`]: ../../lpc82x/struct.SYSCON.html
pub struct SYSCON<'syscon> {
    /// Main SYSCON API
    pub api: Api<'syscon>,

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
    /// Can be used to run the self-wake-up timer (WKT).
    pub irc_derived_clock: IrcDerivedClock<clock::state::Disabled>,
}

impl<'syscon> SYSCON<'syscon> {
    pub(crate) fn new(syscon: &'syscon lpc82x::SYSCON) -> Self {
        SYSCON {
            api: Api {
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


/// Main API of the SYSCON peripheral
pub struct Api<'syscon> {
    pdruncfg     : &'syscon PDRUNCFG,
    presetctrl   : &'syscon PRESETCTRL,
    sysahbclkctrl: &'syscon SYSAHBCLKCTRL,
}

impl<'r> Api<'r> {
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
/// Can be used to control brown-out detection using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct BOD(PhantomData<*const ()>);

/// Flash memory
///
/// Can be used to control the flash memory using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct FLASH(PhantomData<*const ()>);

/// IRC
///
/// Can be used to control the IRC using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct IRC(PhantomData<*const ()>);

/// IRC output
///
/// Can be used to control IRC output using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct IRCOUT(PhantomData<*const ()>);

/// Micro Trace Buffer
///
/// Can be used to control the Micro Trace Buffer using various [`SYSCON`]
/// methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct MTB(PhantomData<*const ()>);

/// Random access memory
///
/// Can be used to control the RAM using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
#[allow(non_camel_case_types)]
pub struct RAM0_1(PhantomData<*const ()>);

/// Read-only memory
///
/// Can be used to control the ROM using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct ROM(PhantomData<*const ()>);

/// System oscillator
///
/// Can be used to control the system oscillator using various [`SYSCON`]
/// methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct SYSOSC(PhantomData<*const ()>);

/// PLL
///
/// Can be used to control the PLL using various [`SYSCON`] methods.
///
/// [`SYSCON`]: struct.SYSCON.html
pub struct SYSPLL(PhantomData<*const ()>);

/// UART Fractional Baud Rate Generator
///
/// Controls the clock for all USART peripherals (U_PCLK).
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


/// Implemented for peripherals that have a clock that can be enabled
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`SYSCON::enable_clock`] and [`SYSCON::disable_clock`] for
/// the public API that uses this trait.
///
/// [`SYSCON::enable_clock`]: struct.SYSCON.html#method.enable_clock
/// [`SYSCON::disable_clock`]: struct.SYSCON.html#method.disable_clock
pub trait ClockControl {
    /// Internal method to enable a peripheral clock
    fn enable_clock<'w>(&mut self, w: &'w mut sysahbclkctrl::W)
        -> &'w mut sysahbclkctrl::W;

    /// Internal method to disable a peripheral clock
    fn disable_clock<'w>(&mut self, w: &'w mut sysahbclkctrl::W)
        -> &'w mut sysahbclkctrl::W;
}

macro_rules! impl_enable_clock {
    ($clock_control:ty, $clock:ident) => {
        impl<'a> ClockControl for $clock_control {
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

impl_enable_clock!(ROM                  , rom     );
impl_enable_clock!(RAM0_1               , ram0_1  );
impl_enable_clock!(&'a lpc82x::FLASHCTRL, flashreg);
impl_enable_clock!(FLASH                , flash   );
impl_enable_clock!(&'a lpc82x::I2C0     , i2c0    );
impl_enable_clock!(&'a lpc82x::GPIO_PORT, gpio    );
impl_enable_clock!(&'a lpc82x::SWM      , swm     );
impl_enable_clock!(&'a lpc82x::SCT      , sct     );
impl_enable_clock!(&'a lpc82x::WKT      , wkt     );
impl_enable_clock!(&'a lpc82x::MRT      , mrt     );
impl_enable_clock!(&'a lpc82x::SPI0     , spi0    );
impl_enable_clock!(&'a lpc82x::SPI1     , spi1    );
impl_enable_clock!(&'a lpc82x::CRC      , crc     );
impl_enable_clock!(&'a lpc82x::USART0   , uart0   );
impl_enable_clock!(&'a lpc82x::USART1   , uart1   );
impl_enable_clock!(&'a lpc82x::USART2   , uart2   );
impl_enable_clock!(&'a lpc82x::WWDT     , wwdt    );
impl_enable_clock!(&'a lpc82x::IOCON    , iocon   );
impl_enable_clock!(&'a lpc82x::CMP      , acmp    );
impl_enable_clock!(&'a lpc82x::I2C1     , i2c1    );
impl_enable_clock!(&'a lpc82x::I2C2     , i2c2    );
impl_enable_clock!(&'a lpc82x::I2C3     , i2c3    );
impl_enable_clock!(&'a lpc82x::ADC      , adc     );
impl_enable_clock!(MTB                  , mtb     );
impl_enable_clock!(&'a lpc82x::DMA      , dma     );


/// Implemented for peripherals that can be reset
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`SYSCON::assert_reset`] and [`SYSCON::clear_reset`] for the
/// public API that uses this trait.
///
/// [`SYSCON::assert_reset`]: struct.SYSCON.html#method.assert_reset
/// [`SYSCON::clear_reset`]: struct.SYSCON.html#method.clear_reset
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

impl_reset_control!(&'a lpc82x::SPI0     , spi0_rst_n   );
impl_reset_control!(&'a lpc82x::SPI1     , spi1_rst_n   );
impl_reset_control!(UARTFRG<'a>          , uartfrg_rst_n);
impl_reset_control!(&'a lpc82x::USART0   , uart0_rst_n  );
impl_reset_control!(&'a lpc82x::USART1   , uart1_rst_n  );
impl_reset_control!(&'a lpc82x::USART2   , uart2_rst_n  );
impl_reset_control!(&'a lpc82x::I2C0     , i2c0_rst_n   );
impl_reset_control!(&'a lpc82x::MRT      , mrt_rst_n    );
impl_reset_control!(&'a lpc82x::SCT      , sct_rst_n    );
impl_reset_control!(&'a lpc82x::WKT      , wkt_rst_n    );
impl_reset_control!(&'a lpc82x::GPIO_PORT, gpio_rst_n   );
impl_reset_control!(&'a lpc82x::FLASHCTRL, flash_rst_n  );
impl_reset_control!(&'a lpc82x::CMP      , acmp_rst_n   );


/// Implemented for analog blocks whose power can be controlled
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`SYSCON::power_up`] and [`SYSCON::power_down`] for the
/// public API that uses this trait.
///
/// [`SYSCON::power_up`]: struct.SYSCON.html#method.power_up
/// [`SYSCON::power_down`]: struct.SYSCON.html#method.power_down
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

impl_analog_block!(IRCOUT          , ircout_pd );
impl_analog_block!(IRC             , irc_pd    );
impl_analog_block!(FLASH           , flash_pd  );
impl_analog_block!(BOD             , bod_pd    );
impl_analog_block!(&'a lpc82x::ADC , adc_pd    );
impl_analog_block!(SYSOSC          , sysosc_pd );
impl_analog_block!(&'a lpc82x::WWDT, wdtosc_pd );
impl_analog_block!(SYSPLL          , syspll_pd );
impl_analog_block!(&'a lpc82x::CMP , acmp      );


/// The 750 kHz IRC-derived clock that can run the WKT
///
/// See user manual, section 18.5.1.
pub struct IrcDerivedClock<State: ClockState = clock::state::Enabled> {
    _state: State,
}

impl IrcDerivedClock<clock::state::Disabled> {
    pub(crate) fn new() -> Self {
        IrcDerivedClock {
            _state: clock::state::Disabled,
        }
    }

    /// Enable the clock
    ///
    /// This method consumes this instance of `IrcDerivedClock` and returns an
    /// instance that implements [`clock::Enabled`].
    ///
    /// This function consumes the handles to IRC and IRCOUT, to make it
    /// impossible (outside of unsafe code) to break API guarantees by disabling
    /// the IRC-derived clock again.
    ///
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, syscon: &mut Api, mut irc: IRC, mut ircout: IRCOUT)
        -> IrcDerivedClock<clock::state::Enabled>
    {
        syscon.power_up(&mut irc);
        syscon.power_up(&mut ircout);

        IrcDerivedClock {
            _state: clock::state::Enabled,
        }
    }
}

impl<State> clock::Frequency for IrcDerivedClock<State>
    where State: ClockState
{
    fn hz(&self) -> u32 { 750_000 }
}

impl clock::Enabled for IrcDerivedClock<clock::state::Enabled> {}
