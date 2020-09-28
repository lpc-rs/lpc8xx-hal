#![no_main]
#![no_std]

extern crate panic_rtt_target;

use core::marker::PhantomData;

use lpc8xx_hal::{
    i2c, init_state::Enabled, pac::I2C0, prelude::*, syscon::IOSC, Peripherals,
};
use rtt_target::rprintln;

const ADDRESS: u8 = 0x24;

#[rtic::app(device = lpc8xx_hal::pac)]
const APP: () = {
    struct Resources {
        i2c_master: i2c::Master<I2C0, Enabled<PhantomData<IOSC>>, Enabled>,
        i2c_slave: i2c::Slave<I2C0, Enabled<PhantomData<IOSC>>, Enabled>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtt_target::rtt_init_print!();

        let p = Peripherals::take().unwrap();

        let mut syscon = p.SYSCON.split();
        let swm = p.SWM.split();

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

        init::LateResources {
            i2c_master: i2c.master,
            i2c_slave: i2c.slave,
        }
    }

    #[idle(resources = [i2c_master])]
    fn idle(context: idle::Context) -> ! {
        let data = 0x14;

        let i2c = context.resources.i2c_master;

        loop {
            rprintln!("MASTER: Starting I2C transaction...");

            // Write data to slave
            i2c.write(ADDRESS, &[data]).unwrap();

            rprintln!("MASTER: Data written.");

            // Read data from slave
            let mut reply = [0; 1];
            i2c.read(ADDRESS, &mut reply).unwrap();

            rprintln!("MASTER: Reply read.");

            // Verify that slave replied with the correct data
            assert_eq!(reply[0], data * 2);

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
};
