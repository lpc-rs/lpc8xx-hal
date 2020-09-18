use lpc8xx_hal::{
    Peripherals,
    pins::{
        self,
        Pin,
    },
    swm,
};


fn main() {
    let mut p = Peripherals::take().unwrap();

    let     swm    = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    #[cfg(feature = "82x")]
    let mut swm_handle = swm.handle;
    #[cfg(feature = "845")]
    let mut swm_handle = swm.handle.enable(&mut syscon.handle);

    let pio0_0: Pin<_, pins::state::Unused> = p.pins.pio0_0;
    let pio0_1: Pin<_, pins::state::Unused> = p.pins.pio0_1;

    let u0_rxd: swm::Function<_, swm::state::Unassigned> =
        swm.movable_functions.u0_rxd;

    let (u0_rxd, _) = u0_rxd.assign(pio0_0.into_swm_pin(), &mut swm_handle);
    // Should fail: Already assigned function to another pin.
    let (u0_rxd, _) = u0_rxd.assign(pio0_1.into_swm_pin(), &mut swm_handle);
}
