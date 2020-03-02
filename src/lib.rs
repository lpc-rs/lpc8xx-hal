//! # LPC8xx HAL
//!
//! Hardware Abstraction Layer (HAL) for the NXP LPC800 series of ARM Cortex-M0+
//! microcontrollers.
//!
//!
//! ## Adding LPC8xx HAL as a dependency
//!
//! To use LPC8xx HAL in your project, you need to include it via Cargo, by
//! adding a dependency to you `Cargo.toml`:
//!
//! ``` toml
//! [dependencies.lpc8xx-hal]
//! version  = "0.6"
//! features = ["824m201jhi33"]
//! ```
//!
//! The above adds a dependency on the `lpc8xx-hal` crate and selects your
//! target hardware. To find out which targets are supported, please check out
//! the list of targets in our [`Cargo.toml`].
//!
//! In principle, there are two things you might want to do differently in your
//! project (besides selecting another target):
//!
//! 1. Select a less specific target.
//! 2. Enable runtime support.
//!
//! If you're writing an application or library that can work with (part of) a
//! family you can select that instead:
//!
//! ``` toml
//! [dependencies.lpc8xx-hal]
//! version  = "0.6"
//! features = ["82x"]
//! ```
//!
//! This selects the LPC82x family. Only the hardware resources available on all
//! targets within that family will be provided, while the actual target
//! hardware you're running on might have more peripherals or more memory.
//!
//! Again, check out [`Cargo.toml`] for a list of options.
//!
//! If you want to use LPC8xx HAL in an application (as opposed to a library),
//! you probably need to enable runtime support. You can do this by passing
//! the runtime feature for your selected family:
//!
//! ``` toml
//! [dependencies.lpc8xx-hal]
//! version  = "0.6"
//! features = ["824m201jhi33", "82x-rt"]
//! ```
//!
//! Again, the available options are listed in [`Cargo.toml`].
//!
//! Please note that LPC8xx HAL is an implementation of [embedded-hal]. If you
//! are writing code that is not specific to LPC800, please consider depending
//! on embedded-hal instead.
//!
//! That's it! Now you can start using the LPC8xx HAL APIs. Take a look at
//! [`Peripherals`], which is the entry point to the whole API.
//!
//! [`Cargo.toml`]: https://github.com/lpc-rs/lpc8xx-hal/blob/master/Cargo.toml
//!
//!
//! ## Examples
//!
//! There are a number of [examples in the repository]. A good place to start is
//! the [GPIO example].
//!
//! If you have an LPCXpresso824-MAX development board connected via USB, you
//! should be able to run any example like this:
//!
//! ``` ignore
//! cargo run --release --features=82x-rt --example gpio_simple
//! ```
//!
//!
//! ## Other documentation
//!
//! Please refer to the [Embedded Rust Book] for further documentation on how to
//! use embedded Rust. The book does not use LPC8xx HAL as an example, but most
//! of what you learn their will transfer over to this crate.
//!
//! [Embedded Rust Book]: https://rust-embedded.github.io/book/
//!
//!
//! # References
//!
//! Various places in this crate's documentation reference the LPC82x User
//! manual, which is [available from NXP].
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal
//! [examples in the repository]: https://github.com/lpc-rs/lpc8xx-hal/tree/master/examples
//! [GPIO example]: https://github.com/lpc-rs/lpc8xx-hal/blob/master/examples/gpio_delay.rs
//! [available from NXP]: https://www.nxp.com/docs/en/user-guide/UM10800.pdf

#![no_std]
#![deny(missing_docs)]

#[cfg(test)]
extern crate std;

pub extern crate cortex_m;
#[cfg(feature = "rt-selected")]
pub extern crate cortex_m_rt;
pub extern crate embedded_hal;
pub extern crate nb;

#[macro_use]
pub(crate) mod reg_proxy;

pub mod adc;
pub mod clock;
#[cfg(feature = "845")]
pub mod ctimer;
pub mod delay;
pub mod dma;
pub mod gpio;
pub mod i2c;
pub mod mrt;
pub mod pins;
pub mod pmu;
pub mod sleep;
pub mod spi;
pub mod swm;
pub mod syscon;
pub mod usart;
pub mod wkt;

/// Re-exports various traits that are required to use lpc8xx-hal
///
/// The purpose of this module is to improve convenience, by not requiring the
/// user to import traits separately. Just add the following to your code, and
/// you should be good to go:
///
/// ``` rust
/// use lpc82x_hal::prelude::*;
/// ```
///
/// The traits in this module have been renamed, to avoid collisions with other
/// imports.
pub mod prelude {
    pub use core::fmt::Write as _;

    pub use crate::clock::{Enabled as _, Frequency as _};
    pub use crate::hal::{digital::v2::*, prelude::*};
    pub use crate::sleep::Sleep as _;
}

#[cfg(feature = "82x")]
pub use lpc82x_pac as pac;
#[cfg(feature = "845")]
pub use lpc845_pac as pac;

pub use self::adc::ADC;
#[cfg(feature = "845")]
pub use self::ctimer::CTimer;
pub use self::dma::DMA;
pub use self::gpio::GPIO;
pub use self::i2c::I2C;
pub use self::mrt::MRT;
pub use self::pmu::PMU;
pub use self::spi::SPI;
pub use self::swm::SWM;
pub use self::syscon::SYSCON;
pub use self::usart::USART;
pub use self::wkt::WKT;

pub use pac::CorePeripherals;

use embedded_hal as hal;

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
    /// Pins that can be used for GPIO or other functions
    pub pins: pins::Pins,

    /// Analog-to-Digital Converter (ADC)
    pub ADC: ADC<init_state::Disabled>,

    /// Standard counter/timer (CTIMER)
    #[cfg(feature = "845")]
    pub CTIMER0: CTimer,

    /// DMA controller
    pub DMA: DMA,

    /// General-purpose I/O (GPIO)
    ///
    /// By default, the GPIO peripheral is enabled on the LPC82x and disabled on
    /// the LPC845.
    #[cfg(feature = "82x")]
    pub GPIO: GPIO<init_state::Enabled>,

    /// General-purpose I/O (GPIO)
    ///
    /// By default, the GPIO peripheral is enabled on the LPC82x and disabled on
    /// the LPC845.
    #[cfg(feature = "845")]
    pub GPIO: GPIO<init_state::Disabled>,

    /// I2C0-bus interface
    pub I2C0: I2C<pac::I2C0, init_state::Disabled>,

    /// Multi-Rate Timer (MRT)
    pub MRT0: MRT,

    /// Power Management Unit
    pub PMU: PMU,

    /// SPI0
    pub SPI0: SPI<pac::SPI0, init_state::Disabled>,

    /// SPI1
    pub SPI1: SPI<pac::SPI1, init_state::Disabled>,

    /// Switch matrix
    ///
    /// By default, the switch matrix is enabled on the LPC82x and disabled on
    /// the LPC845.
    ///
    /// The reference manual for the LPC845 suggests otherwise, but it seems to
    /// be wrong.
    #[cfg(feature = "82x")]
    pub SWM: SWM<init_state::Enabled>,

    /// Switch matrix
    ///
    /// By default, the switch matrix is enabled on the LPC82x and disabled on
    /// the LPC845.
    ///
    /// The reference manual for the LPC845 suggests otherwise, but it seems to
    /// be wrong.
    #[cfg(feature = "845")]
    pub SWM: SWM<init_state::Disabled>,

    /// System configuration
    pub SYSCON: SYSCON,

    /// USART0
    pub USART0: USART<pac::USART0, init_state::Disabled>,

    /// USART1
    pub USART1: USART<pac::USART1, init_state::Disabled>,

    /// USART2
    pub USART2: USART<pac::USART2, init_state::Disabled>,

    #[cfg(feature = "845")]
    /// USART3
    ///
    /// USART3 and PIN_INT6 share an interrupt, this may cause difficulties
    /// when trying to use both at the same time
    pub USART3: USART<pac::USART3, init_state::Disabled>,

    #[cfg(feature = "845")]
    /// USART4
    ///
    /// USART4 and PIN_INT7 share an interrupt, this may cause difficulties
    /// when trying to use both at the same time
    pub USART4: USART<pac::USART4, init_state::Disabled>,

    /// Self-wake-up timer (WKT)
    pub WKT: WKT<init_state::Disabled>,

    /// Analog comparator
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub ACOMP: pac::ACOMP,

    /// Capacitive Touch (CAPT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    #[cfg(feature = "845")]
    pub CAPT: pac::CAPT,

    /// CRC engine
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub CRC: pac::CRC,

    /// Digital-to-Analog Converter 0 (DAC0)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    #[cfg(feature = "845")]
    pub DAC0: pac::DAC0,

    /// Digital-to-Analog Converter 1 (DAC1)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    #[cfg(feature = "845")]
    pub DAC1: pac::DAC1,

    /// Flash controller
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub FLASH_CTRL: pac::FLASH_CTRL,

    /// I2C1-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C1: pac::I2C1,

    /// I2C2-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C2: pac::I2C2,

    /// I2C3-bus interface
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub I2C3: pac::I2C3,

    /// Input multiplexing
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub INPUTMUX: pac::INPUTMUX,

    /// I/O configuration
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub IOCON: pac::IOCON,

    /// Pin interrupt and pattern match engine
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub PINT: pac::PINT,

    /// State Configurable Timer (SCT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub SCT0: pac::SCT0,

    /// Windowed Watchdog Timer (WWDT)
    ///
    /// A HAL API for this peripheral has not been implemented yet. In the
    /// meantime, this field provides you with the raw register mappings, which
    /// allow you full, unprotected access to the peripheral.
    pub WWDT: pac::WWDT,
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
    /// use lpc82x_hal::Peripherals;
    ///
    /// // This code should be at the beginning of your program. As long as this
    /// // is the only place that calls `take`, the following should never
    /// // panic.
    /// let p = Peripherals::take().unwrap();
    /// ```
    pub fn take() -> Option<Self> {
        Some(Self::new(pac::Peripherals::take()?))
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
        Self::new(pac::Peripherals::steal())
    }

    fn new(p: pac::Peripherals) -> Self {
        Peripherals {
            pins: pins::Pins::new(),

            // HAL peripherals
            ADC: ADC::new(p.ADC0),
            #[cfg(feature = "845")]
            CTIMER0: CTimer::new(p.CTIMER0),
            DMA: DMA::new(p.DMA0),
            GPIO: GPIO::new(p.GPIO),
            I2C0: I2C::new(p.I2C0),
            MRT0: MRT::new(p.MRT0),
            PMU: PMU::new(p.PMU),
            SPI0: SPI::new(p.SPI0),
            SPI1: SPI::new(p.SPI1),
            SWM: SWM::new(p.SWM0),
            SYSCON: SYSCON::new(p.SYSCON),
            USART0: USART::new(p.USART0),
            USART1: USART::new(p.USART1),
            USART2: USART::new(p.USART2),
            #[cfg(feature = "845")]
            USART3: USART::new(p.USART3),
            #[cfg(feature = "845")]
            USART4: USART::new(p.USART4),
            WKT: WKT::new(p.WKT),

            // Raw peripherals
            ACOMP: p.ACOMP,
            #[cfg(feature = "845")]
            CAPT: p.CAPT,
            CRC: p.CRC,
            #[cfg(feature = "845")]
            DAC0: p.DAC0,
            #[cfg(feature = "845")]
            DAC1: p.DAC1,
            FLASH_CTRL: p.FLASH_CTRL,
            I2C1: p.I2C1,
            I2C2: p.I2C2,
            I2C3: p.I2C3,
            INPUTMUX: p.INPUTMUX,
            IOCON: p.IOCON,
            PINT: p.PINT,
            SCT0: p.SCT0,
            WWDT: p.WWDT,
        }
    }
}

/// Contains types that encode the state of hardware initialization
///
/// The types in this module are used by structs representing peripherals or
/// other hardware components, to encode the initialization state of the
/// underlying hardware as part of the type.
pub mod init_state {
    /// Indicates that the hardware component is enabled
    ///
    /// This usually indicates that the hardware has been initialized and can be
    /// used for its intended purpose. Contains an optional payload that APIs
    /// can use to keep data that is only available while enabled.
    pub struct Enabled<T = ()>(pub T);

    /// Indicates that the hardware component is disabled
    pub struct Disabled;
}
