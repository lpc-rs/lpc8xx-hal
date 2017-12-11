//! APIs for system configuration (SYSCON)
//!
//! See user manual, chapter 5.


use core::marker::PhantomData;

use lpc82x;
use lpc82x::syscon::{
    pdruncfg,
    presetctrl,
    sysahbclkctrl,
};

use clock;
use clock::state::ClockState;


/// Interface to system configuration (SYSCON)
///
/// This API expects to be the sole owner of the SYSCON interface. Don't use
/// [`lpc82x::SYSCON`] directly, unless you know what you're doing.
///
/// [`lpc82x::SYSCON`]: ../../lpc82x/struct.SYSCON.html
pub struct Syscon<'syscon>(&'syscon lpc82x::SYSCON);

impl<'syscon> Syscon<'syscon> {
    pub(crate) fn new(syscon: &'syscon lpc82x::SYSCON) -> Self {
        Syscon(syscon)
    }

    /// Enable peripheral clock
    ///
    /// Enables the clock for a peripheral or other hardware component. HAL
    /// users usually won't have to call this method directly, as other
    /// peripheral APIs will do this for them.
    pub fn enable_clock<C: ClockControl>(&mut self, peripheral: &mut C) {
        self.0.sysahbclkctrl.modify(|_, w| peripheral.enable_clock(w));
    }

    /// Disable peripheral clock
    pub fn disable_clock<C: ClockControl>(&mut self, peripheral: &mut C) {
        self.0.sysahbclkctrl.modify(|_, w| peripheral.disable_clock(w));
    }

    /// Assert peripheral reset
    pub fn assert_reset<R: ResetControl>(&mut self, peripheral: &mut R) {
        self.0.presetctrl.modify(|_, w| peripheral.assert_reset(w));
    }

    /// Clear peripheral reset
    ///
    /// Clears the reset for a peripheral or other hardware component. HAL users
    /// usually won't have to call this method directly, as other peripheral
    /// APIs will do this for them.
    pub fn clear_reset<R: ResetControl>(&mut self, peripheral: &mut R) {
        self.0.presetctrl.modify(|_, w| peripheral.clear_reset(w));
    }

    /// Provide power to an analog block
    ///
    /// HAL users usually won't have to call this method themselves, as other
    /// peripheral APIs will do this for them.
    pub fn power_up<A: AnalogBlock>(&mut self) {
        self.0.pdruncfg.modify(|_, w| A::power_up(w));
    }

    /// Remove power from an analog block
    pub fn power_down<A: AnalogBlock>(&mut self) {
        self.0.pdruncfg.modify(|_, w| A::power_down(w));
    }

    /// Sets the clock for all USART peripherals (U_PCLK)
    ///
    /// HAL users usually won't have to call this method directly, as the
    /// [`Usart`] API will handle this.
    ///
    /// # Limitations
    ///
    /// This method can be used to overwrite the settings for USARTs that are
    /// currently in use. Please make sure not to do that.
    ///
    /// [`Usart`]: ../usart/struct.Usart.html
    pub fn set_uart_clock(&mut self,
        uart_clk_div : &UartClkDiv,
        uart_frg_mult: &UartFrgMult,
        uart_frg_div : &UartFrgDiv,
    ) {
        unsafe {
            self.0.uartclkdiv.write(|w| w.div().bits(uart_clk_div.0));

            self.0.uartfrgmult.write(|w| w.mult().bits(uart_frg_mult.0));
            self.0.uartfrgdiv.write(|w| w.div().bits(uart_frg_div.0));
        }
    }
}


/// Brown-out detection
///
/// Can be used to control brown-out detection using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct BOD(PhantomData<*const ()>);

impl BOD {
    pub(crate) fn new() -> Self {
        BOD(PhantomData)
    }
}


/// Flash memory
///
/// Can be used to control the flash memory using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct FLASH(PhantomData<*const ()>);

impl FLASH {
    pub(crate) fn new() -> Self {
        FLASH(PhantomData)
    }
}


/// IRC
///
/// Can be used to control the IRC using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct IRC(PhantomData<*const ()>);

impl IRC {
    pub(crate) fn new() -> Self {
        IRC(PhantomData)
    }
}


/// IRC output
///
/// Can be used to control IRC output using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct IRCOUT(PhantomData<*const ()>);

impl IRCOUT {
    pub(crate) fn new() -> Self {
        IRCOUT(PhantomData)
    }
}


/// Micro Trace Buffer
///
/// Can be used to control the Micro Trace Buffer using various [`Syscon`]
/// methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct MTB(PhantomData<*const ()>);

impl MTB {
    pub(crate) fn new() -> Self {
        MTB(PhantomData)
    }
}


/// Random access memory
///
/// Can be used to control the RAM using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
#[allow(non_camel_case_types)]
pub struct RAM0_1(PhantomData<*const ()>);

impl RAM0_1 {
    pub(crate) fn new() -> Self {
        RAM0_1(PhantomData)
    }
}


/// Read-only memory
///
/// Can be used to control the ROM using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct ROM(PhantomData<*const ()>);

impl ROM {
    pub(crate) fn new() -> Self {
        ROM(PhantomData)
    }
}


/// System oscillator
///
/// Can be used to control the system oscillator using various [`Syscon`]
/// methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct SYSOSC(PhantomData<*const ()>);

impl SYSOSC {
    pub(crate) fn new() -> Self {
        SYSOSC(PhantomData)
    }
}


/// PLL
///
/// Can be used to control the PLL using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct SYSPLL(PhantomData<*const ()>);

impl SYSPLL {
    pub(crate) fn new() -> Self {
        SYSPLL(PhantomData)
    }
}


/// UART Fractional Baud Rate Generator
///
/// Can be used to control the UART FRG using various [`Syscon`] methods.
///
/// [`Syscon`]: struct.Syscon.html
pub struct UARTFRG(PhantomData<*const ()>);

impl UARTFRG {
    pub(crate) fn new() -> Self {
        UARTFRG(PhantomData)
    }
}



/// Implemented for peripherals that have a clock that can be enabled
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`Syscon::enable_clock`] and [`Syscon::disable_clock`] for
/// the public API that uses this trait.
///
/// [`Syscon::enable_clock`]: struct.Syscon.html#method.enable_clock
/// [`Syscon::disable_clock`]: struct.Syscon.html#method.disable_clock
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
/// Please refer to [`Syscon::assert_reset`] and [`Syscon::clear_reset`] for the
/// public API that uses this trait.
///
/// [`Syscon::assert_reset`]: struct.Syscon.html#method.assert_reset
/// [`Syscon::clear_reset`]: struct.Syscon.html#method.clear_reset
pub trait ResetControl {
    /// Internal method to assert peripheral reset
    fn assert_reset<'w>(&mut self, w: &'w mut presetctrl::W)
        -> &'w mut presetctrl::W;

    /// Internal method to clear peripheral reset
    fn clear_reset<'w>(&mut self, w: &'w mut presetctrl::W)
        -> &'w mut presetctrl::W;
}

macro_rules! impl_clear_reset {
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

impl_clear_reset!(&'a lpc82x::SPI0     , spi0_rst_n   );
impl_clear_reset!(&'a lpc82x::SPI1     , spi1_rst_n   );
impl_clear_reset!(UARTFRG              , uartfrg_rst_n);
impl_clear_reset!(&'a lpc82x::USART0   , uart0_rst_n  );
impl_clear_reset!(&'a lpc82x::USART1   , uart1_rst_n  );
impl_clear_reset!(&'a lpc82x::USART2   , uart2_rst_n  );
impl_clear_reset!(&'a lpc82x::I2C0     , i2c0_rst_n   );
impl_clear_reset!(&'a lpc82x::MRT      , mrt_rst_n    );
impl_clear_reset!(&'a lpc82x::SCT      , sct_rst_n    );
impl_clear_reset!(&'a lpc82x::WKT      , wkt_rst_n    );
impl_clear_reset!(&'a lpc82x::GPIO_PORT, gpio_rst_n   );
impl_clear_reset!(&'a lpc82x::FLASHCTRL, flash_rst_n  );
impl_clear_reset!(&'a lpc82x::CMP      , acmp_rst_n   );


/// Implemented for analog blocks whose power can be controlled
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC82x HAL. Any incompatible changes to this
/// trait won't be considered breaking changes.
///
/// Please refer to [`Syscon::power_up`] and [`Syscon::power_down`] for the
/// public API that uses this trait.
///
/// [`Syscon::power_up`]: struct.Syscon.html#method.power_up
/// [`Syscon::power_down`]: struct.Syscon.html#method.power_down
pub trait AnalogBlock {
    /// Internal method to power up an analog block
    fn power_up(w: &mut pdruncfg::W) -> &mut pdruncfg::W;

    /// Internal method to power down an analog block
    fn power_down(w: &mut pdruncfg::W) -> &mut pdruncfg::W;
}

macro_rules! impl_analog_block {
    ($analog_block:ty, $field:ident) => {
        impl<'a> AnalogBlock for $analog_block {
            fn power_up(w: &mut pdruncfg::W) -> &mut pdruncfg::W {
                w.$field().powered()
            }

            fn power_down(w: &mut pdruncfg::W) -> &mut pdruncfg::W {
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


/// UART clock divider value
///
/// See [`Syscon::set_uart_clock`].
///
/// [`Syscon::set_uart_clock`]: struct.Syscon.html#method.set_uart_clock
pub struct UartClkDiv(pub u8);

/// UART fractional generator multiplier value
///
/// See [`Syscon::set_uart_clock`].
///
/// [`Syscon::set_uart_clock`]: struct.Syscon.html#method.set_uart_clock
pub struct UartFrgMult(pub u8);

/// UART fractional generator divider value
///
/// See [`Syscon::set_uart_clock`].
///
/// [`Syscon::set_uart_clock`]: struct.Syscon.html#method.set_uart_clock
pub struct UartFrgDiv(pub u8);


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
    pub fn enable(self, syscon: &mut Syscon, _irc: IRC, _ircout: IRCOUT)
        -> IrcDerivedClock<clock::state::Enabled>
    {
        syscon.power_up::<IRC>();
        syscon.power_up::<IRCOUT>();

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
