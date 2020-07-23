#![no_main]
#![no_std]

extern crate panic_rtt_target;

use lpc8xx_hal::{cortex_m_rt::entry, dma, usart, Peripherals};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let p = Peripherals::take().unwrap();

    static mut DMA_DESCRIPTOR: dma::DescriptorTable =
        dma::DescriptorTable::new();
    // Sound, as this is the only place where we do this.
    let dma_descriptors = unsafe { &mut DMA_DESCRIPTOR };

    let swm = p.SWM.split();
    let dma = p.DMA.split(dma_descriptors);
    let mut syscon = p.SYSCON.split();

    let dma_handle = dma.handle.enable(&mut syscon.handle);
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

    let mut serial =
        p.USART0
            .enable(&clock_config, &mut syscon.handle, u0_rxd, u0_txd);

    let mut rx_channel = dma.channels.channel0.enable(&dma_handle);
    let mut tx_channel = dma.channels.channel1.enable(&dma_handle);

    static mut BUF: [u8; 4] = [0; 4];

    loop {
        {
            // Sound, as the mutable reference is dropped after this block.
            let rx_buf = unsafe { &mut BUF };

            let res = serial.rx.read_all(rx_buf, rx_channel).wait().unwrap();
            rx_channel = res.channel;
            serial.rx = res.source;
        }

        {
            // Sound, as the mutable reference is dropped after this block.
            let tx_buf = unsafe { &BUF };

            let res = serial
                .tx
                .write_all(tx_buf, tx_channel)
                .wait()
                .expect("USART write shouldn't fail");
            tx_channel = res.channel;
            serial.tx = res.dest;
        }
    }
}
