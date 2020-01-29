#![no_main]
#![no_std]

extern crate panic_halt;

use lpc8xx_hal::{
    cortex_m_rt::entry, prelude::*, syscon::clocksource::SpiClock, Peripherals,
};

use embedded_hal::spi::{Mode, Phase, Polarity};

#[entry]
fn main() -> ! {
    const MODE: Mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };
    let p = Peripherals::take().unwrap();

    let swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    #[cfg(feature = "82x")]
    let mut handle = swm.handle;
    #[cfg(feature = "845")]
    let mut handle = swm.handle.enable(&mut syscon.handle); // SWM isn't enabled by default on LPC845.

    let sck_pin = swm.pins.pio0_9.into_swm_pin();
    let mosi_pin = swm.pins.pio0_10.into_swm_pin();
    let miso_pin = swm.pins.pio0_11.into_swm_pin();

    let (spi0_sck, _) =
        swm.movable_functions.spi0_sck.assign(sck_pin, &mut handle);
    let (spi0_mosi, _) = swm
        .movable_functions
        .spi0_mosi
        .assign(mosi_pin, &mut handle);
    let (spi0_miso, _) = swm
        .movable_functions
        .spi0_miso
        .assign(miso_pin, &mut handle);

    #[cfg(feature = "82x")]
    let spi_clock = SpiClock::new(0);
    #[cfg(feature = "845")]
    let spi_clock = SpiClock::new(&syscon.iosc, 0);

    // Enable SPI0
    let mut spi = p.SPI0.enable(
        &spi_clock,
        &mut syscon.handle,
        MODE,
        spi0_sck,
        spi0_mosi,
        spi0_miso,
    );

    // We're done. Let's do nothing until someone resets the microcontroller.
    loop {
        // Cycle through colors on 16 chained APA102C LEDs
        loop {
            for r in 0..255 {
                let _ = spi.write(&[0, 0, 0, 0]);
                for _i in 0..16 {
                    let _ = spi.write(&[0b1110_0001, 0, 0, r]);
                }
                let _ = spi.write(&[0xFF, 0xFF, 0xFF, 0xFF]);
            }
            for b in 0..255 {
                let _ = spi.write(&[0, 0, 0, 0]);
                for _i in 0..16 {
                    let _ = spi.write(&[0b1110_0001, b, 0, 0]);
                }
                let _ = spi.write(&[0xFF, 0xFF, 0xFF, 0xFF]);
            }
            for g in 0..255 {
                let _ = spi.write(&[0, 0, 0, 0]);
                for _i in 0..16 {
                    let _ = spi.write(&[0b1110_0001, 0, g, 0]);
                }
                let _ = spi.write(&[0xFF, 0xFF, 0xFF, 0xFF]);
            }
        }
    }
}
