#![no_main]
#![no_std]

extern crate panic_rtt_target;

#[rtic::app(device = lpc8xx_hal::pac, peripherals = false)]
mod app {
    use core::marker::PhantomData;

    use lpc8xx_hal::{
        dma, i2c, init_state::Enabled, pac::I2C0, syscon::IOSC, Peripherals,
    };
    use rtt_target::rprintln;

    const ADDRESS: u8 = 0x24;

    #[resources]
    struct Resources {
        #[lock_free]
        i2c_master:
            Option<i2c::Master<I2C0, Enabled<PhantomData<IOSC>>, Enabled>>,

        #[lock_free]
        i2c_slave: i2c::Slave<I2C0, Enabled<PhantomData<IOSC>>, Enabled>,

        #[lock_free]
        dma_channel:
            Option<dma::Channel<<I2C0 as i2c::Instance>::MstChannel, Enabled>>,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        rtt_target::rtt_init_print!();

        let p = Peripherals::take().unwrap();

        let mut syscon = p.SYSCON.split();
        let swm = p.SWM.split();
        let dma = p.DMA.enable(&mut syscon.handle);

        let mut swm_handle = swm.handle.enable(&mut syscon.handle);

        let (i2c0_scl, _) = swm
            .fixed_functions
            .i2c0_scl
            .assign(p.pins.pio0_10.into_swm_pin(), &mut swm_handle);
        let (i2c0_sda, _) = swm
            .fixed_functions
            .i2c0_sda
            .assign(p.pins.pio0_11.into_swm_pin(), &mut swm_handle);

        let mut i2c = p
            .I2C0
            .enable(&syscon.iosc, i2c0_scl, i2c0_sda, &mut syscon.handle)
            .enable_master_mode(&i2c::Clock::new_400khz())
            .enable_slave_mode(ADDRESS)
            .expect("`ADDRESS` not a valid 7-bit address");

        i2c.enable_interrupts(i2c::Interrupts {
            slave_pending: true,
            ..Default::default()
        });

        (
            init::LateResources {
                i2c_master: Some(i2c.master),
                i2c_slave: i2c.slave,
                dma_channel: Some(dma.channels.channel15),
            },
            init::Monotonics(),
        )
    }

    #[idle(resources = [i2c_master, dma_channel])]
    fn idle(context: idle::Context) -> ! {
        static mut RX_BUF: [u8; 1] = [0; 1];

        let mut rx_buf = &mut RX_BUF[..];

        static TX_BUF: [u8; 1] = [0x14];

        // The `.take().unwrap()` workaround is required, because RTIC won't
        // allow us to move resources in here directly.
        let mut i2c = context.resources.i2c_master.take().unwrap();
        let mut channel = context.resources.dma_channel.take().unwrap();

        loop {
            rprintln!("MASTER: Starting I2C transaction...");

            // Write data to slave
            let payload = i2c
                .write_all(ADDRESS, &TX_BUF[..], channel)
                .unwrap()
                .start()
                .wait()
                .unwrap();

            channel = payload.channel;
            i2c = payload.dest;

            rprintln!("MASTER: Data written.");

            // Read data from slave
            rx_buf[0] = 0;
            let payload = i2c
                .read_all(ADDRESS, rx_buf, channel)
                .unwrap()
                .start()
                .wait()
                .unwrap();

            channel = payload.channel;
            i2c = payload.source;
            rx_buf = payload.dest;

            rprintln!("MASTER: Reply read.");

            // Verify that slave replied with the correct data
            assert_eq!(rx_buf[0], TX_BUF[0] * 2);

            rprintln!("MASTER: Reply verified.");
        }
    }

    #[task(binds = I2C0, resources = [i2c_slave])]
    fn i2c0(context: i2c0::Context) {
        static mut DATA: Option<u8> = None;

        let i2c = context.resources.i2c_slave;

        rprintln!("SLAVE: Handling interrupt...");

        match i2c.wait() {
            Ok(i2c::slave::State::AddressMatched(i2c)) => {
                rprintln!("SLAVE: Address matched.");

                i2c.ack().unwrap();

                rprintln!("SLAVE: Ack'ed address.");
            }
            Ok(i2c::slave::State::RxReady(i2c)) => {
                rprintln!("SLAVE: Ready to receive.");

                *DATA = Some(i2c.read().unwrap());
                i2c.ack().unwrap();

                rprintln!("SLAVE: Received and ack'ed.");
            }
            Ok(i2c::slave::State::TxReady(i2c)) => {
                rprintln!("SLAVE: Ready to transmit.");

                if let Some(data) = *DATA {
                    i2c.transmit(data << 1).unwrap();
                    rprintln!("SLAVE: Transmitted.");
                }
            }
            Err(nb::Error::WouldBlock) => {
                // I2C not ready; nothing to do
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        }
    }
}
