#![no_main]
#![no_std]

extern crate panic_rtt_target;

use lpc8xx_hal::{cortex_m_rt::entry, usart, Peripherals};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let p = Peripherals::take().unwrap();

    let swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    let dma = p.DMA.enable(&mut syscon.handle);
    let mut swm_handle = swm.handle.enable(&mut syscon.handle);

    let clock_config = usart::Clock::new_with_baudrate(115200);

    let (u0_rxd, _) = swm
        .movable_functions
        .u0_rxd
        .assign(p.pins.pio0_24.into_swm_pin(), &mut swm_handle);
    let (u0_txd, _) = swm
        .movable_functions
        .u0_txd
        .assign(p.pins.pio0_25.into_swm_pin(), &mut swm_handle);

    let mut serial = p.USART0.enable_async(
        &clock_config,
        &mut syscon.handle,
        u0_rxd,
        u0_txd,
        usart::Settings::default(),
    );

    let mut rx_channel = dma.channels.channel0;
    let mut tx_channel = dma.channels.channel1;

    static mut BUF: [u8; 4] = [0; 4];

    loop {
        {
            // Sound, as the mutable reference is dropped after this block.
            let rx_buf = unsafe { &mut BUF };

            let res = serial
                .rx
                .read_all(rx_buf, rx_channel)
                .start()
                .wait()
                .unwrap();
            rx_channel = res.channel;
            serial.rx = res.source;
        }

        {
            // Sound, as the mutable reference is dropped after this block.
            let tx_buf = unsafe { &BUF };

            let res = serial
                .tx
                .write_all(tx_buf, tx_channel)
                .start()
                .wait()
                .expect("USART write shouldn't fail");
            tx_channel = res.channel;
            serial.tx = res.dest;
        }
    }
}
