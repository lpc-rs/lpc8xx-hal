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
//! That's it! Now you can just start using the LPC82x HAL APIs.
//!
//! Please note that LPC82x HAL is an implementation of [embedded-hal]. If your
//! library is not specific to LPC82x, please consider depending on embedded-hal
//! instead. Doing so means that your library will work on top of all
//! implementations of embedded-hal.
//!
//! ## Using LPC82x HAL in an Application
//!
//! To use LPC82x HAL in your application, you need to go through a bit of
//! additional trouble. This section tries to walk you through some of the
//! basics, but it's not a complete tutorial. Please refer to
//! [cortex-m-quickstart] for additional details.
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
//! can install it using Cargo:
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
//! and is left as an exercise to the reader.
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
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//!
//! use lpc82x_hal::prelude::*;
//! use lpc82x_hal::{
//!     GPIO,
//!     SWM,
//!     SYSCON,
//!     WKT,
//! };
//! use lpc82x_hal::clock::Ticks;
//! use lpc82x_hal::gpio::PIO0_3;
//! use lpc82x_hal::sleep::{
//!     self,
//!     Sleep,
//! };
//!
//! // Create the struct we're going to use to access all the peripherals. This
//! // is unsafe, because we're only allowed to create one instance.
//! let mut peripherals = lpc82x::Peripherals::take().unwrap();
//!
//! // Create the peripheral interfaces.
//! let     gpio   = GPIO::new(&mut peripherals.GPIO_PORT);
//! let     swm    = SWM::new(&mut peripherals.SWM);
//! let mut syscon = SYSCON::new(&mut peripherals.SYSCON);
//! let     wkt    = WKT::new(&mut peripherals.WKT);
//!
//! // Other peripherals need to be initialized. Trying to use the API before
//! // initializing them will actually lead to compile-time errors.
//! let mut gpio_handle = gpio.handle.init(&mut syscon.handle);
//! let mut swm_handle  = swm.handle.init(&mut syscon.handle);
//! let mut wkt         = wkt.init(&mut syscon.handle);
//!
//! // We're going to need a clock for sleeping. Let's use the IRC-derived clock
//! // that runs at 750 kHz.
//! let clock = syscon.irc_derived_clock.enable(
//!     &mut syscon.handle,
//!     syscon.irc,
//!     syscon.ircout,
//! );
//!
//! // In the next step, we need to configure the pin PIO0_3 and its fixed
//! // function SWCLK. The API tracks the state of both of those, to prevent any
//! // mistakes on our side. However, since we could have changed the state of
//! // the hardware before initializing the API, the API can't know what state
//! // it is currently in.
//! // Let's affirm that we haven't changed anything, and that PIO0_3 and SWCLK
//! // are still in their initial states.
//! let pio0_3 = unsafe { gpio.pins.pio0_3.affirm_default_state()          };
//! let swclk  = unsafe { swm.fixed_functions.swclk.affirm_default_state() };
//!
//! // Configure PIO0_3 as GPIO output, so we can use it to blink an LED.
//! let (pio0_3, _) = pio0_3
//!     .disable_output_function(swclk, &mut swm_handle);
//! let mut pio0_3 = pio0_3
//!     .as_unused_pin()
//!     .as_gpio_pin(&gpio_handle)
//!     .as_output();
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
//!     pio0_3.set_high();
//!     sleep.sleep(high_time);
//!     pio0_3.set_low();
//!     sleep.sleep(low_time);
//! }
//! ```
//!
//! # References
//!
//! Various places in this crate's documentation reference the LPC82x User
//! manual, which is [available from NXP].
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal
//! [cortex-m-quickstart]: https://github.com/japaric/cortex-m-quickstart
//! [cortex-m-rt]: https://crates.io/crates/cortex-m-rt
//! [Xargo]: https://crates.io/crates/xargo
//! [This fork of lpc21isp]: https://github.com/hannobraun/lpc21isp
//! [available from NXP]: https://www.nxp.com/docs/en/user-guide/UM10800.pdf


#![feature(const_fn)]
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
extern crate nb;


pub mod clock;
pub mod gpio;
pub mod pmu;
pub mod sleep;
pub mod swm;
pub mod syscon;
pub mod usart;
pub mod wkt;


/// Re-exports various traits that are required to use lpc82x-hal
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
    pub use embedded_hal::prelude::*;

    pub use clock::Enabled as _lpc82x_hal_clock_Enabled;
    pub use clock::Frequency as _lpc82x_hal_clock_Frequency;
    pub use sleep::Sleep as _lpc82x_hal_sleep_Sleep;
}


pub use lpc82x::{
    CPUID,
    DCB,
    DWT,
    MPU,
    NVIC,
    SCB,
    SYST,
    Interrupt,
};

pub use self::gpio::GPIO;
pub use self::pmu::PMU;
pub use self::swm::SWM;
pub use self::syscon::SYSCON;
pub use self::usart::USART;
pub use self::wkt::WKT;


/// Contains types that encode the state hardware initialization
///
/// The types in this module are used by structs representing peripherals or
/// other hardware components, to encode the initialization state of the
/// underlying hardware as part of the type.
pub mod init_state {
    /// Implemented by the types that represent the initialization states
    ///
    /// This type is used as a trait bound for type paramters that represent
    /// initialization states. This is done for the purpose of documentation.
    /// HAL users should never need to implement this trait, nor use it
    /// directly.
    pub trait InitState {}

    /// Indicates that the hardware's state is currently unknown
    ///
    /// This is usually the initial state after the HAL API has been
    /// initialized, as we don't know what happened before that.
    pub struct Unknown;
    impl InitState for Unknown {}

    /// Indicates that the hardware component is enabled
    ///
    /// This usually indicates that the hardware has been initialized and can be
    /// used for its intended purpose.
    pub struct Enabled;
    impl InitState for Enabled {}
}
