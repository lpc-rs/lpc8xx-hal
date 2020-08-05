use core::marker::PhantomData;

use void::Void;

use crate::{
    dma::{
        self,
        transfer::state::{Ready, Started},
    },
    init_state::Enabled,
    pac::dma0::channel::xfercfg::{DSTINC_A, SRCINC_A},
};

use super::{Instance, Master, SPI};

/// An SPI/DMA transfer
///
/// Since the SPI peripheral is capable of sending and receiving at the same
/// time, using the same buffer, it needs this bespoke `Transfer` struct, which
/// wraps and manages two `dma::Transfer` structs under the hood.
pub struct Transfer<State, I: Instance> {
    spi: SPI<I, Enabled<Master>>,
    buffer: &'static mut [u8],
    rx_transfer: dma::Transfer<State, I::RxChannel, Rx<I>, dma::Buffer>,
    tx_transfer: dma::Transfer<State, I::TxChannel, dma::Buffer, Tx<I>>,
}

impl<I> Transfer<Ready, I>
where
    I: Instance,
{
    pub(super) fn new(
        spi: SPI<I, Enabled<Master>>,
        buffer: &'static mut [u8],
        rx_channel: dma::Channel<I::RxChannel, Enabled>,
        tx_channel: dma::Channel<I::TxChannel, Enabled>,
    ) -> Self {
        let ptr = buffer.as_mut_ptr();
        let len = buffer.len();

        // This is sound, since we know that the SPI peripheral will not access
        // the buffers concurrently, due to the way the protocol works:
        // - An SPI master will never receive a word unless it sends a word at
        //   the same time. That means the peripheral will always be ready to
        //   send a word _before_ it has received one.
        // - Once a word has been received, it will overwrite the word in the
        //   buffer that was sent during the same clock cycle. At that point,
        //   that part of the buffer will no longer be relevant for the sending
        //   side.
        let rx_buffer = unsafe { dma::Buffer::new(ptr, len) };
        let tx_buffer = unsafe { dma::Buffer::new(ptr, len) };

        let rx_transfer =
            dma::Transfer::new(rx_channel, Rx(PhantomData), rx_buffer);
        let tx_transfer =
            dma::Transfer::new(tx_channel, tx_buffer, Tx(PhantomData));

        Self {
            spi,
            buffer,
            rx_transfer,
            tx_transfer,
        }
    }

    /// Start the transfer
    ///
    /// Starts both DMA transfers that are part of this SPI transfer.
    pub fn start(self) -> Transfer<Started, I> {
        Transfer {
            spi: self.spi,
            buffer: self.buffer,
            rx_transfer: self.rx_transfer.start(),
            tx_transfer: self.tx_transfer.start(),
        }
    }
}

impl<I> Transfer<Started, I>
where
    I: Instance,
{
    /// Wait for the transfer to finish
    ///
    /// Waits until both underlying DMA transfers have finished.
    pub fn wait(
        self,
    ) -> (
        SPI<I, Enabled<Master>>,
        &'static mut [u8],
        dma::Channel<I::RxChannel, Enabled>,
        dma::Channel<I::TxChannel, Enabled>,
    ) {
        let rx_payload = match self.rx_transfer.wait() {
            Ok(payload) => payload,
            // can't happen, as error type is `Void`
            Err(_) => unreachable!(),
        };
        let tx_payload = match self.tx_transfer.wait() {
            Ok(payload) => payload,
            // can't happen, as error type is `Void`
            Err(_) => unreachable!(),
        };

        (
            self.spi,
            self.buffer,
            rx_payload.channel,
            tx_payload.channel,
        )
    }
}

/// Represents the receiving portion of the DMA peripheral
struct Rx<I>(PhantomData<I>);

impl<I> crate::private::Sealed for Rx<I> {}

impl<I> dma::Source for Rx<I>
where
    I: Instance,
{
    type Error = Void;

    fn is_valid(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn increment(&self) -> SRCINC_A {
        SRCINC_A::NO_INCREMENT
    }

    fn transfer_count(&self) -> Option<u16> {
        None
    }

    fn end_addr(&self) -> *const u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).rxdat }) as *const _ as *mut u8
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

/// Represents the sending portion of the DMA peripheral
struct Tx<I>(PhantomData<I>);

impl<I> crate::private::Sealed for Tx<I> {}

impl<I> dma::Dest for Tx<I>
where
    I: Instance,
{
    type Error = Void;

    fn is_valid(&self) -> bool {
        true
    }

    fn is_full(&self) -> bool {
        false
    }

    fn increment(&self) -> DSTINC_A {
        DSTINC_A::NO_INCREMENT
    }

    fn transfer_count(&self) -> Option<u16> {
        None
    }

    fn end_addr(&mut self) -> *mut u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).txdat }) as *const _ as *mut u8
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}
