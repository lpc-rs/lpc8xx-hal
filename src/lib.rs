//! # LPC82x Hardware Abstraction Layer
//!
//! Hardware Abstraction Layer (HAL) for the NXP LPC82x series of ARM Cortex-M0+
//! microcontrollers.
//!
//! ## Using LPC82x HAL in a Library
//!
//! Writing a library on top of LPC82x HAL is pretty simple. All you need to do
//! is include it via Cargo, by adding the following to you `Cargo.toml`:
//!
//! ``` toml
//! [dependencies.lpc82x-hal]
//! git = "https://github.com/braun-robotics/rust-lpc82x-hal.git"
//! ```
//!
//! With that in place, you can just reference the crate in you Rust code, like
//! this:
//!
//! ```rust
//! // lib.rs
//!
//! extern crate lpc82x_hal;
//! ```
//!
//! That's it! Now you can just start using the APIs from LPC82x HAL. For
//! libraries, it is recommended to just take (mutable) references to anything
//! that's needed, and leave initialization to the user.
//!
//! ## Using LPC82x HAL in an Application
//!
//! To use LPC82x HAL in your application, you need to go through a bit of
//! additional trouble. This section tries to walk you through some of the
//! basics, but it's not a complete tutorial. Please refer to a more complete
//! example, like [cortex-m-quickstart], for additional details.
//!
//! ### Runtime Support
//!
//! Including LPC82x HAL in your application via Cargo is mostly the same as it
//! is for libraries, but with one addition. You need to enable runtime support
//! when including the crate in your `Cargo.toml`:
//!
//! ``` toml
//! [dependencies.lpc82x-hal]
//! git      = "https://github.com/braun-robotics/rust-lpc82x-hal.git"
//! features = ["rt"]
//! ```
//!
//! The runtime support will provide you with some basics that are required for
//! your program to run correctly. However, it needs to know how the memory on
//! your microcontroller is set up.
//!
//! You can get that information from the user manual. To provide it to LPC82x
//! HAL, create a file called `memory.x` in your project root (the directory
//! where `Cargo.toml` is located). `memory.x` should look something like this:
//!
//! ``` ignore
//! MEMORY
//! {
//!     FLASH : ORIGIN = 0x00000000, LENGTH = 32K
//!     RAM   : ORIGIN = 0x10000000, LENGTH = 8K
//! }
//! ```
//!
//! Runtime support is provided by the [cortex-m-rt] crate. Please refer to the
//! cortex-m-rt documentation for additional details.
//!
//! ### Build System
//!
//! The build system needs to be set up to compile and link a binary for LPC82x.
//! Cargo alone is not enough for this, as its support for embedded development
//! is currently limited. [Xargo] exists to fill the gap in the meantime. You
//! can install it via `cargo install`:
//!
//! ``` ignore
//! $ cargo install xargo
//! ```
//!
//! Add a new file, `Xargo.toml` right next to your `Cargo.toml`, with the
//! following contents:
//!
//! ``` toml
//! [dependencies.core]
//! stage = 0
//!
//! [dependencies.compiler_builtins]
//! stage    = 1
//! features = ["c", "mem"]
//! ```
//!
//! You might not need all those optional features of `compiler_builtin`, so
//! feel free to experiment.
//!
//! Additionally, you need to tell Cargo how to link your project. Create the
//! file `.cargo/config` in your project directory, and add the following
//! contents:
//!
//! ``` toml
//! [target.thumbv6m-none-eabi]
//! rustflags = [
//!     "-C", "link-arg=-Tlink.x",
//!     "-C", "linker=arm-none-eabi-ld",
//!     "-Z", "linker-flavor=ld"
//! ]
//! ```
//!
//! This tells Cargo to use the arm-none-eabi-gcc toolchain for linking. You
//! need to install this separately. How to do so is dependent on your platform
//! and left as an exercise to the reader.
//!
//! If everything is set up correctly, you can build your project with the
//! following command:
//!
//! ``` ignore
//! $ xargo build --release --target=thumbv6m-none-eabi
//! ```
//!
//! ### Uploading the Binary
//!
//! There are many ways to upload the binary to the microcontroller. How to do
//! this is currently beyond the scope of this documentation, but
//! [this fork of lpc21isp] is known to work.
//!
//! ## Example
//!
//! The following is an example of a simple application that blinks an LED.
//!
//! ``` no_run
//! use lpc82x_hal::{
//!     PIO0_3,
//!     System,
//! };
//! use lpc82x_hal::clock::Ticks;
//! use lpc82x_hal::sleep::{
//!     self,
//!     Sleep,
//! };
//!
//! // Initialize the system. This is unsafe, because we're only allowed to
//! // create one instance on `System`.
//! let system = unsafe { System::new() };
//!
//! // Let's save some peripherals in local variables for convenience. This one
//! // here doesn't require initialization.
//! let mut syscon = system.peripherals.syscon;
//!
//! // Other peripherals need to be initialized. Trying to use the API before
//! // initializing it will actually lead to compile-time errors.
//! let mut gpio = system.peripherals.gpio.init(&mut syscon);
//! let mut swm  = system.peripherals.swm.init(&mut syscon);
//! let mut wkt  = system.peripherals.wkt.init(&mut syscon);
//!
//! // We're going to need a clock for sleeping. Let's use the IRC-derived clock
//! // that runs at 750 kHz.
//! let clock = system.clocks.irc_derived_clock.enable(
//!     &mut syscon,
//!     system.resources.irc,
//!     system.resources.ircout,
//! );
//!
//! // Set pin direction to output, so we can use it to blink an LED.
//! gpio.set_pin_to_output::<PIO0_3>(&mut swm);
//!
//! // Let's already initialize the durations that we're going to sleep for
//! // between changing the LED state. We do this by specifying the number of
//! // clock ticks directly, but a real program could use a library that allows
//! // us to specify the time in milliseconds.
//! // Each duration also keeps a reference to the clock, as to prevent other
//! // parts of the program from accidentally disabling the clock, or changing
//! // its settings.
//! let high_time = Ticks { value:  37_500, clock: &clock }; //  50 ms
//! let low_time  = Ticks { value: 712_500, clock: &clock }; // 950 ms
//!
//! // Since this is a simple example, we don't want to deal with interrupts
//! // here. Let's just use busy waiting as a sleeping strategy.
//! let mut sleep = sleep::Busy::prepare(&mut wkt);
//!
//! // Blink the LED
//! loop {
//!     gpio.set_high::<PIO0_3>();
//!     sleep.sleep(high_time);
//!     gpio.set_low::<PIO0_3>();
//!     sleep.sleep(low_time);
//! }
//! ```
//!
//! # References
//!
//! Various places in this crate's documentation reference the LPC82x User
//! manual, which is [available from NXP].
//!
//! [cortex-m-quickstart]: https://github.com/japaric/cortex-m-quickstart
//! [cortex-m-rt]: https://crates.io/crates/cortex-m-rt
//! [Xargo]: https://crates.io/crates/xargo
//! [This fork of lpc21isp]: https://github.com/hannobraun/lpc21isp
//! [`System`]: struct.System.html
//! [available from NXP]: https://www.nxp.com/docs/en/user-guide/UM10800.pdf


#![feature(const_refcell_new)]
#![feature(macro_reexport)]
#![feature(never_type)]

#![deny(warnings)]
#![deny(missing_docs)]

#![no_std]


#[cfg(test)]
extern crate std;

extern crate cortex_m;
extern crate embedded_hal;
#[cfg_attr(feature = "rt",
    macro_reexport(default_handler, exception, interrupt))]
extern crate lpc82x;
#[macro_use]
extern crate nb;


pub mod clock;
pub mod gpio;
pub mod pmu;
pub mod sleep;
pub mod swm;
pub mod syscon;
pub mod usart;
pub mod wkt;


pub use lpc82x::{
    CPUID,
    DCB,
    DWT,
    FPB,
    FPU,
    ITM,
    MPU,
    NVIC,
    SCB,
    SYST,
    TPIU,
    Interrupt,
};

pub use self::gpio::GPIO;
pub use self::pmu::Pmu;
pub use self::swm::Swm;
pub use self::syscon::SYSCON;
pub use self::usart::USART;
pub use self::wkt::WKT;


/// Entry point to the HAL API
///
/// This struct provides access to all peripherals and other system resources.
/// It consists of multiple sub-structs for each category of system resource.
///
/// Only one instance of this struct must exist in your program.
///
/// # Limitations
///
/// Currently not all system resources are actually available from this struct
/// and its sub-structs. Many of them are modelled as unit-like structs that can
/// be used and instantiated freely. This is a temporary state of affairs, and
/// work to integrate those types into this struct is impending.
pub struct System<'system> {
    /// System clocks
    pub clocks: Clocks,

    /// System peripherals
    pub peripherals: Peripherals<'system>,

    /// Other system resources
    pub resources: Resources,
}

impl<'system> System<'system> {
    /// Creates an instance of `System`
    ///
    /// Only one instance of `System` must exist in your program. Use this
    /// method at the start of your program, to create a single `System` that
    /// will serve as an entry point to the HAL API.
    ///
    /// # Safety
    ///
    /// You must guarantee to only use this method to create a single instance
    /// of `System`. Usually this means you call this method once, at the
    /// beginning of your program. But technically, you can call it again to
    /// create another instance, if the previous one has been dropped.
    pub unsafe fn new() -> Self {
        let peripherals = lpc82x::Peripherals::all();

        System {
            clocks: Clocks {
                irc_derived_clock: syscon::IrcDerivedClock::new(),
                low_power_clock  : pmu::LowPowerClock::new(),
            },
            peripherals: Peripherals {
                cpuid: peripherals.CPUID,
                dcb  : peripherals.DCB,
                dwt  : peripherals.DWT,
                nvic : peripherals.NVIC,
                scb  : peripherals.SCB,
                syst : peripherals.SYST,

                adc       : peripherals.ADC,
                cmp       : peripherals.CMP,
                crc       : peripherals.CRC,
                dma       : peripherals.DMA,
                dmatrigmux: peripherals.DMATRIGMUX,
                flashctrl : peripherals.FLASHCTRL,
                i2c0      : peripherals.I2C0,
                i2c1      : peripherals.I2C1,
                i2c2      : peripherals.I2C2,
                i2c3      : peripherals.I2C3,
                inputmux  : peripherals.INPUTMUX,
                iocon     : peripherals.IOCON,
                mrt       : peripherals.MRT,
                pin_int   : peripherals.PIN_INT,
                sct       : peripherals.SCT,
                spi0      : peripherals.SPI0,
                spi1      : peripherals.SPI1,
                wwdt      : peripherals.WWDT,

                gpio  : GPIO::new(peripherals.GPIO_PORT),
                pmu   : Pmu::new(peripherals.PMU),
                swm   : Swm::new(peripherals.SWM),
                syscon: SYSCON::new(peripherals.SYSCON),
                usart0: USART::new(peripherals.USART0),
                usart1: USART::new(peripherals.USART1),
                usart2: USART::new(peripherals.USART2),
                wkt   : WKT::new(peripherals.WKT),
            },
            resources: Resources {
                bod    : syscon::BOD::new(),
                flash  : syscon::FLASH::new(),
                irc    : syscon::IRC::new(),
                ircout : syscon::IRCOUT::new(),
                mtb    : syscon::MTB::new(),
                ram0_1 : syscon::RAM0_1::new(),
                rom    : syscon::ROM::new(),
                sysosc : syscon::SYSOSC::new(),
                syspll : syscon::SYSPLL::new(),
                uartfrg: syscon::UARTFRG::new(),
            },
        }
    }
}


/// Provides access to clocks in the system
///
/// This struct is part of [`System`].
///
/// [`System`]: struct.System.html
pub struct Clocks {
    /// The 750 kHz IRC-derived clock
    ///
    /// Can be used to run the self-wake-up timer (WKT).
    pub irc_derived_clock: syscon::IrcDerivedClock<clock::state::Disabled>,

    /// The 10 kHz low-power clock
    ///
    /// Can be used to run the self-wake-up timer (WKT).
    pub low_power_clock: pmu::LowPowerClock<clock::state::Disabled>,
}


/// Provides access to all peripherals
///
/// This struct is part of [`System`].
///
/// [`System`]: struct.System.html
pub struct Peripherals<'system> {
    /// CPUID register
    ///
    /// This peripheral is available in all Cortex-M0+ microcontrollers and has
    /// been inherited from the [cortex-m] crate.
    ///
    /// [cortex-m]: https://crates.io/crates/cortex-m
    pub cpuid: &'system lpc82x::CPUID,

    /// Debug Control Block
    ///
    /// This peripheral is available in all Cortex-M0+ microcontrollers and has
    /// been inherited from the [cortex-m] crate.
    ///
    /// [cortex-m]: https://crates.io/crates/cortex-m
    pub dcb: &'system lpc82x::DCB,

    /// Data Watchpoint and Trace
    ///
    /// This peripheral is available in all Cortex-M0+ microcontrollers and has
    /// been inherited from the [cortex-m] crate.
    ///
    /// [cortex-m]: https://crates.io/crates/cortex-m
    pub dwt: &'system lpc82x::DWT,

    /// Nested Vectored Interrupt Controller
    ///
    /// This peripheral is available in all Cortex-M0+ microcontrollers and has
    /// been inherited from the [cortex-m] crate.
    ///
    /// [cortex-m]: https://crates.io/crates/cortex-m
    pub nvic: &'system lpc82x::NVIC,

    /// System Control Block
    ///
    /// This peripheral is available in all Cortex-M0+ microcontrollers and has
    /// been inherited from the [cortex-m] crate.
    ///
    /// [cortex-m]: https://crates.io/crates/cortex-m
    pub scb: &'system lpc82x::SCB,

    /// SysTick timer
    ///
    /// This peripheral is available in all Cortex-M0+ microcontrollers and has
    /// been inherited from the [cortex-m] crate.
    ///
    /// [cortex-m]: https://crates.io/crates/cortex-m
    pub syst: &'system lpc82x::SYST,

    /// Analog-to-Digital Converter (ADC)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub adc: &'system lpc82x::ADC,

    /// Analog comparator
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub cmp: &'system lpc82x::CMP,

    /// CRC engine
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub crc: &'system lpc82x::CRC,

    /// DMA controller
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub dma: &'system lpc82x::DMA,

    /// DMA trigger multiplexing (DMA TRIGMUX)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub dmatrigmux: &'system lpc82x::DMATRIGMUX,

    /// Flash controller
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub flashctrl: &'system lpc82x::FLASHCTRL,

    /// I2C0
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub i2c0: &'system lpc82x::I2C0,

    /// I2C1
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub i2c1: &'system lpc82x::I2C1,

    /// I2C2
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub i2c2: &'system lpc82x::I2C2,

    /// I2C3
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub i2c3: &'system lpc82x::I2C3,

    /// Input multiplexing (INPUT MUX)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub inputmux: &'system lpc82x::INPUTMUX,

    /// I/O configuration (IOCON)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub iocon: &'system lpc82x::IOCON,

    /// Multi-Rate Timer (MRT)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub mrt: &'system lpc82x::MRT,

    /// Pin interrupts/pattern match engine
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub pin_int: &'system lpc82x::PIN_INT,

    /// State Configurable Timer (SCT)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub sct: &'system lpc82x::SCT,

    /// SPI0
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub spi0: &'system lpc82x::SPI0,

    /// SPI1
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub spi1: &'system lpc82x::SPI1,

    /// Windowed Watchdog Timer (WWDT)
    ///
    /// A hardware abstraction layer has not yet been implemented for this
    /// peripheral. This field provides access to the raw registers from the
    /// [lpc82x] crate.
    ///
    /// [lpc82x]: https://crates.io/crates/lpc82x
    pub wwdt: &'system lpc82x::WWDT,

    /// General Purpose I/O (GPIO)
    pub gpio: GPIO<'system, init_state::Unknown>,

    /// Power Management Unit (PMU)
    pub pmu: Pmu<'system>,

    /// Switch matrix (SWM)
    pub swm: Swm<'system, init_state::Unknown>,

    /// System configuration (SYSCON)
    pub syscon: SYSCON<'system>,

    /// USART0
    pub usart0: USART<'system, lpc82x::USART0, init_state::Unknown>,

    /// USART1
    pub usart1: USART<'system, lpc82x::USART1, init_state::Unknown>,

    /// USART2
    pub usart2: USART<'system, lpc82x::USART2, init_state::Unknown>,

    /// Self-wake-up timer (WKT)
    pub wkt: WKT<'system, init_state::Unknown>,
}


/// Provides access to other system resources
///
/// This struct is part of [`System`].
///
/// [`System`]: struct.System.html
pub struct Resources {
    /// Brown-out detection
    pub bod: syscon::BOD,

    /// Flash memory
    pub flash: syscon::FLASH,

    /// IRC
    pub irc: syscon::IRC,

    /// IRC output
    pub ircout: syscon::IRCOUT,

    /// Micro Trace Buffer
    pub mtb: syscon::MTB,

    /// Random access memory
    pub ram0_1: syscon::RAM0_1,

    /// Read-only memory
    pub rom: syscon::ROM,

    /// System oscillator
    pub sysosc: syscon::SYSOSC,

    /// PLL
    pub syspll: syscon::SYSPLL,

    /// UART Fractional Baud Rate Generator
    pub uartfrg: syscon::UARTFRG,
}


/// Represents a pin
///
/// This trait is implemented by all types that represent a pin. HAL users
/// shouldn't need to implement this trait themselves.
///
/// It also should not be necessary for HAL users to use the methods of this
/// trait directly, unless compensating for missing pieces of HAL functionality.
/// Ideally, there should be higher-level peripheral methods that take pins as
/// parameters and use the methods of this trait to take care of the low-level
/// details.
pub trait Pin {
    /// Returns a number that identifies the pin
    ///
    /// This is `0` for [`PIO0_0`], `1` for [`PIO0_1`] and so forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    fn id() -> u8;

    /// Returns the pin's mask
    ///
    /// This is `0x00000001` for [`PIO0_0`], `0x00000002` for [`PIO0_1`] and so
    /// forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    fn mask() -> u32;
}

macro_rules! impl_pin {
    ($pin:ident, $id:expr) => {
        /// Represents the pin this struct is named after
        ///
        /// # Limitations
        ///
        /// Currently, nothing prevents users of this HAL from creating any
        /// number of instances of this struct and using them for all kinds of
        /// purposes. Until this shortcoming is rectified, it is your own
        /// responsibility to make sure you are using the pin correctly.
        #[allow(non_camel_case_types)]
        pub struct $pin;

        impl Pin for $pin {
            fn id() -> u8 {
                $id
            }

            fn mask() -> u32 {
                0x1 << $id
            }
        }
    }
}

impl_pin!(PIO0_0 , 0x00);
impl_pin!(PIO0_1 , 0x01);
impl_pin!(PIO0_2 , 0x02);
impl_pin!(PIO0_3 , 0x03);
impl_pin!(PIO0_4 , 0x04);
impl_pin!(PIO0_5 , 0x05);
impl_pin!(PIO0_6 , 0x06);
impl_pin!(PIO0_7 , 0x07);
impl_pin!(PIO0_8 , 0x08);
impl_pin!(PIO0_9 , 0x09);
impl_pin!(PIO0_10, 0x0a);
impl_pin!(PIO0_11, 0x0b);
impl_pin!(PIO0_12, 0x0c);
impl_pin!(PIO0_13, 0x0d);
impl_pin!(PIO0_14, 0x0e);
impl_pin!(PIO0_15, 0x0f);
impl_pin!(PIO0_16, 0x10);
impl_pin!(PIO0_17, 0x11);
impl_pin!(PIO0_18, 0x12);
impl_pin!(PIO0_19, 0x13);
impl_pin!(PIO0_20, 0x14);
impl_pin!(PIO0_21, 0x15);
impl_pin!(PIO0_22, 0x16);
impl_pin!(PIO0_23, 0x17);
impl_pin!(PIO0_24, 0x18);
impl_pin!(PIO0_25, 0x19);
impl_pin!(PIO0_26, 0x1a);
impl_pin!(PIO0_27, 0x1b);
impl_pin!(PIO0_28, 0x1c);


/// Contains types that mark the state of peripheral initialization
pub mod init_state {
    /// Implemented by types that indicate peripheral initialization state
    ///
    /// This type is used as a trait bound for type paramters that indicate a
    /// peripheral's initialization state. HAL users should never need to
    /// implement this trait, nor use it directly.
    pub trait InitState {}

    /// Marks a peripherals initialization state as being unknown
    ///
    /// This is usually the initial state after system initialization.
    pub struct Unknown;
    impl InitState for Unknown {}

    /// Marks a peripherals as being initialized
    pub struct Initialized;
    impl InitState for Initialized {}
}
