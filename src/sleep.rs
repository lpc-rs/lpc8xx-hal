//! Higher-level sleep API
//!
//! This module provides a higher-level API layer that can be used to put the
//! microcontroller to sleep for an amount of time.


use cortex_m::asm;
use embedded_hal::prelude::*;
use lpc82x;
use nb;

use PMU;
use clock::{
    self,
    Ticks,
};
use wkt::{
    self,
    WKT,
};


/// Trait for putting the processor to sleep
///
/// There will typically one implementation of `Sleep` per sleep mode that is
/// available on a given microcontroller.
pub trait Sleep<Clock> where Clock: clock::Enabled {
    /// Puts the processor to sleep for the given number of ticks of the clock
    fn sleep<'clock, T>(&mut self, ticks: T)
        where
            Clock: 'clock,
            T    : Into<Ticks<'clock, Clock>>;
}


/// Sleep mode based on busy waiting
///
/// Provides a [`Sleep`] implementation for based on busy waiting and uses the
/// [WKT] to measure the time. An interrupt handler is not required.
///
/// Only clocks that the WKT supports can be used. See [`wkt::Clock`] for more
/// details.
///
/// Since this sleep mode waits busily, which is very energy-inefficient, it is
/// only suitable for simple examples, or very short wait times.
///
/// [`Sleep`]: trait.Sleep.html
/// [WKT]: ../wkt/struct.WKT.html
/// [`wkt::Clock`]: ../wkt/trait.Clock.html
pub struct Busy<'wkt> {
    wkt: &'wkt mut WKT<'wkt>,
}

impl<'wkt> Busy<'wkt> {
    /// Prepare busy sleep mode
    pub fn prepare(wkt: &'wkt mut WKT<'wkt>) -> Self {
        Busy {
            wkt: wkt,
        }
    }
}

impl<'wkt, Clock> Sleep<Clock> for Busy<'wkt>
    where Clock: clock::Enabled + wkt::Clock
{
    fn sleep<'clock, T>(&mut self, ticks: T)
        where
            Clock: 'clock,
            T    : Into<Ticks<'clock, Clock>>
    {
        let ticks: Ticks<Clock> = ticks.into();

        // If we try to sleep for zero cycles, we'll never wake up again.
        if ticks.value == 0 {
            return;
        }

        self.wkt.set_timeout(ticks.value);
        while let Err(nb::Error::WouldBlock) = self.wkt.wait() {
            asm::nop();
        }
    }
}


/// Regular sleep mode
///
/// Provides a [`Sleep`] implementation for the regular sleep mode and uses the
/// [WKT] to wake the microcontroller up again, at the right time.
///
/// The user must [handle the WKT interrupt], or the program won't wake up
/// again.
///
/// Only clocks that the WKT supports can be used. See [`wkt::Clock`] for more
/// details.
///
/// [`Sleep`]: trait.Sleep.html
/// [WKT]: ../wkt/struct.WKT.html
/// [handle the WKT interrupt]: ../wkt/struct.WKT.html#method.handle_interrupt
/// [`wkt::Clock`]: ../wkt/trait.Clock.html
pub struct Regular<'r, 'pmu, 'wkt> {
    nvic: &'r lpc82x::NVIC,
    pmu : &'pmu mut PMU<'pmu>,
    scb : &'r lpc82x::SCB,
    wkt : &'wkt mut WKT<'wkt>,
}

impl<'r, 'pmu, 'wkt> Regular<'r, 'pmu, 'wkt> {
    /// Prepare regular sleep mode
    pub fn prepare(
        nvic: &'r lpc82x::NVIC,
        pmu : &'pmu mut PMU<'pmu>,
        scb : &'r lpc82x::SCB,
        wkt : &'wkt mut WKT<'wkt>,
    )
        -> Self
    {
        Regular {
            nvic: nvic,
            pmu : pmu,
            scb : scb,
            wkt : wkt,
        }
    }
}

impl<'r, 'pmu, 'wkt, Clock> Sleep<Clock> for Regular<'r, 'pmu, 'wkt>
    where Clock: clock::Enabled + wkt::Clock
{
    fn sleep<'clock, T>(&mut self, ticks: T)
        where
            Clock: 'clock,
            T: Into<Ticks<'clock, Clock>>
    {
        let ticks: Ticks<Clock> = ticks.into();

        // If we try to sleep for zero cycles, we'll never wake up again.
        if ticks.value == 0 {
            return;
        }

        self.wkt.select_clock::<Clock>();
        self.wkt.enable_interrupt(self.nvic);
        self.wkt.set_timeout(ticks.value);

        while let Err(nb::Error::WouldBlock) = self.wkt.wait() {
            self.pmu.enter_sleep_mode(self.scb);
        }
    }
}
