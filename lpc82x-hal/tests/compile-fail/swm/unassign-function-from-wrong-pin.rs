extern crate lpc82x_hal;


use lpc82x_hal::Peripherals;
use lpc82x_hal::swm::{
    self,
    pin_state,
    Pin,
};


fn main() {
    let mut p = Peripherals::take().unwrap();

    let     swm    = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    let pio0_0: Pin<_, pin_state::Unused> = swm.pins.pio0_0;
    let pio0_1: Pin<_, pin_state::Unused> = swm.pins.pio0_1;

    let u0_rxd: swm::Function<_, swm::state::Unassigned> =
        swm.movable_functions.u0_rxd;

    let (u0_rxd, _) = u0_rxd.assign(pio0_0.into_swm_pin(), &mut swm.handle);
    let (u0_rxd, _) = u0_rxd.unassign(pio0_1.into_swm_pin(), &mut swm.handle);
    //~^ ERROR mismatched types [E0308]
}
