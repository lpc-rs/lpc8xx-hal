//! # LPC845 Hardware Abstraction Layer

#![deny(missing_docs)]
#![no_std]

#[cfg(test)]
extern crate std;

pub extern crate lpc845_pac as raw;
pub use lpc8xx_hal_common::*;

pub use self::gpio::GPIO;
pub use self::swm::SWM;
pub use self::syscon::SYSCON;

/// Re-exports various traits that are required to use lpc845-hal
///
/// The purpose of this module is to improve convenience, by not requiring the
/// user to import traits separately. Just add the following to your code, and
/// you should be good to go:
///
/// ``` rust
/// use lpc845_hal::prelude::*;
/// ```
///
/// The traits in this module have been renamed, to avoid collisions with other
/// imports.
pub mod prelude {
    pub use lpc8xx_hal_common::prelude::*;
}

/// Provides access to all peripherals
///
/// This is the entry point to the HAL API. Before you can do anything else, you
/// need to get an instance of this struct via [`Peripherals::take`] or
/// [`Peripherals::steal`].
///
/// The HAL API tracks the state of peripherals at compile-time, to prevent
/// potential bugs before the program can even run. Many parts of this
/// documentation call this "type state". The peripherals available in this
/// struct are set to their initial state (i.e. their state after a system
/// reset). See user manual, section 5.6.14.
///
/// # Safe Use of the API
///
/// Since it should be impossible (outside of unsafe code) to access the
/// peripherals before this struct is initialized, you can rely on the
/// peripheral states being correct, as long as there's no bug in the API, and
/// you're not using unsafe code to do anything that the HAL API can't account
/// for.
///
/// If you directly use unsafe code to access peripherals or manipulate this
/// API, this will be really obvious from the code. But please note that if
/// you're using other APIs to access the hardware, such conflicting hardware
/// access might not be obvious, as the other API might use unsafe code under
/// the hood to access the hardware (just like this API does).
///
/// If you do access the peripherals in any way not intended by this API, please
/// make sure you know what you're doing. In specific terms, this means you
/// should be fully aware of what your code does, and whether that is a valid
/// use of the hardware.
#[allow(non_snake_case)]
pub struct Peripherals {
    /// General-purpose I/O (GPIO)
    ///
    /// The GPIO peripheral is enabled by default. See user manual, section
    /// 5.6.14.
    pub GPIO: GPIO<init_state::Disabled>,

    /// Switch matrix
    pub SWM: SWM,

    /// System configuration
    pub SYSCON: SYSCON,

    /// Self-wake-up timer (WKT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub WKT: raw::WKT,

    /// Analog comparator
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub ACOMP: raw::ACOMP,

    /// Analog-to-Digital Converter (ADC)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub ADC0: raw::ADC0,

    /// Capacitive Touch (CAPT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub CAPT: raw::CAPT,

    /// CRC engine
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub CRC: raw::CRC,

    /// Standard counter/timer (CTIMER)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub CTIMER0: raw::CTIMER0,

    /// Digital-to-Analog Converter 0 (DAC0)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub DAC0: raw::DAC0,

    /// Digital-to-Analog Converter 1 (DAC1)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub DAC1: raw::DAC1,

    /// DMA controller
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub DMA0: raw::DMA0,

    /// Flash controller
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub FLASH_CTRL: raw::FLASH_CTRL,

    /// I2C0-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C0: raw::I2C0,

    /// I2C1-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C1: raw::I2C1,

    /// I2C2-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C2: raw::I2C2,

    /// I2C3-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C3: raw::I2C3,

    /// Input multiplexing
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub INPUTMUX: raw::INPUTMUX,

    /// I/O configuration
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub IOCON: raw::IOCON,

    /// Multi-Rate Timer (MRT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub MRT0: raw::MRT0,

    /// Pin interrupt and pattern match engine
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub PINT: raw::PINT,

    /// Power Management Unit
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub PMU: raw::PMU,

    /// State Configurable Timer (SCT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub SCT0: raw::SCT0,

    /// SPI0
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub SPI0: raw::SPI0,

    /// SPI1
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub SPI1: raw::SPI1,

    /// USART0
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub USART0: raw::USART0,

    /// USART1
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub USART1: raw::USART1,

    /// USART2
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub USART2: raw::USART2,

    /// USART3
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub USART3: raw::USART3,

    /// USART4
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub USART4: raw::USART4,

    /// Windowed Watchdog Timer (WWDT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub WWDT: raw::WWDT,

    /// CPUID
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub CPUID: raw::CPUID,

    /// Debug Control Block (DCB)
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub DCB: raw::DCB,

    /// Data Watchpoint and Trace unit (DWT)
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub DWT: raw::DWT,

    /// Memory Protection Unit (MPU)
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub MPU: raw::MPU,

    /// Nested Vector Interrupt Controller (NVIC)
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub NVIC: raw::NVIC,

    /// System Control Block (SCB)
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub SCB: raw::SCB,

    /// SysTick: System Timer
    ///
    /// This is a core peripherals that's available on all ARM Cortex-M0+ cores.
    pub SYST: raw::SYST,
}

impl Peripherals {
    /// Take the peripherals safely
    ///
    /// This method can only be called one time to access the peripherals. It
    /// will return `Some(Peripherals)` when called for the first time, then
    /// `None` on any subsequent calls.
    ///
    /// Applications should call this method once, at the beginning of their
    /// main method, to get access to the full API. Any other parts of the
    /// program should just expect to be passed whatever parts of the HAL API
    /// they need.
    ///
    /// Calling this method from a library is considered an anti-pattern.
    /// Libraries should just require whatever they need to be passed as
    /// arguments and leave the initialization to the application that calls
    /// them.
    ///
    /// For an alternative way to gain access to the hardware, please take a
    /// look at [`Peripherals::steal`].
    ///
    /// # Example
    ///
    /// ``` no_run
    /// use lpc845_hal::Peripherals;
    ///
    /// // This code should be at the beginning of your program. As long as this
    /// // is the only place that calls `take`, the following should never
    /// // panic.
    /// let p = Peripherals::take().unwrap();
    /// ```
    pub fn take() -> Option<Self> {
        Some(Self::new(
            raw::Peripherals::take()?,
            raw::CorePeripherals::take()?,
        ))
    }

    /// Steal the peripherals
    ///
    /// This function returns an instance of `Peripherals`, whether or not such
    /// an instance exists somewhere else. This is highly unsafe, as it can lead
    /// to conflicting access of the hardware, mismatch between actual hardware
    /// state and peripheral state as tracked by this API at compile-time, and
    /// in general a full nullification of all safety guarantees that this API
    /// would normally make.
    ///
    /// If at all possible, you should always prefer `Peripherals::take` to this
    /// method. The only legitimate use of this API is code that can't access
    /// `Peripherals` the usual way, like a panic handler, or maybe temporary
    /// debug code in an interrupt handler.
    ///
    /// # Safety
    ///
    /// This method returns an instance of `Peripherals` that might conflict
    /// with either other instances of `Peripherals` that exist in the program,
    /// or other means of accessing the hardware. This is only sure, if you make
    /// sure of the following:
    /// 1. No other code can access the hardware at the same time.
    /// 2. You don't change the hardware state in any way that could invalidate
    ///    the type state of other `Peripherals` instances.
    /// 3. The type state in your `Peripherals` instance matches the actual
    ///    state of the hardware.
    ///
    /// Items 1. and 2. are really tricky, so it is recommended to avoid any
    /// situations where they apply, and restrict the use of this method to
    /// situations where the program has effectively ended and the hardware will
    /// be reset right after (like a panic handler).
    ///
    /// Item 3. applies to all uses of this method, and is generally very tricky
    /// to get right. The best way to achieve that is probably to force the API
    /// into a type state that allows you to execute operations that are known
    /// to put the hardware in a safe state. Like forcing the type state for a
    /// peripheral API to the "disabled" state, then enabling it, to make sure
    /// it is enabled, regardless of wheter it was enabled before.
    ///
    /// Since there are no means within this API to forcibly change type state,
    /// you will need to resort to something like [`core::mem::transmute`].
    pub unsafe fn steal() -> Self {
        Self::new(raw::Peripherals::steal(), raw::CorePeripherals::steal())
    }

    fn new(p: raw::Peripherals, cp: raw::CorePeripherals) -> Self {
        Peripherals {
            // HAL peripherals
            // NOTE(unsafe) The init state of the gpio peripheral is enabled,
            // thus it's safe to create an already initialized gpio port
            GPIO: GPIO::new(p.GPIO),
            SWM: SWM::new(p.SWM0),
            SYSCON: SYSCON::new(p.SYSCON),

            // Raw peripherals
            ACOMP: p.ACOMP,
            ADC0: p.ADC0,
            CAPT: p.CAPT,
            CRC: p.CRC,
            CTIMER0: p.CTIMER0,
            DAC0: p.DAC0,
            DAC1: p.DAC1,
            DMA0: p.DMA0,
            FLASH_CTRL: p.FLASH_CTRL,
            I2C0: p.I2C0,
            I2C1: p.I2C1,
            I2C2: p.I2C2,
            I2C3: p.I2C3,
            INPUTMUX: p.INPUTMUX,
            IOCON: p.IOCON,
            MRT0: p.MRT0,
            PINT: p.PINT,
            PMU: p.PMU,
            SCT0: p.SCT0,
            SPI0: p.SPI0,
            SPI1: p.SPI1,
            USART0: p.USART0,
            USART1: p.USART1,
            USART2: p.USART2,
            USART3: p.USART3,
            USART4: p.USART4,
            WKT: p.WKT,
            WWDT: p.WWDT,

            // Core peripherals
            CPUID: cp.CPUID,
            DCB: cp.DCB,
            DWT: cp.DWT,
            MPU: cp.MPU,
            NVIC: cp.NVIC,
            SCB: cp.SCB,
            SYST: cp.SYST,
        }
    }
}
