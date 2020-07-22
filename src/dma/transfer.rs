use core::sync::atomic::{compiler_fence, Ordering};

use crate::init_state::Enabled;

use super::{channels::ChannelTrait, Channel, Handle};

/// A DMA transfer
pub struct Transfer<'dma, T, D>
where
    T: ChannelTrait,
{
    payload: Payload<'dma, T, D>,
}

impl<'dma, T, D> Transfer<'dma, T, D>
where
    T: ChannelTrait,
    D: Dest,
{
    pub(super) fn new(
        channel: Channel<T, Enabled<&'dma Handle>>,
        source: &'static mut [u8],
        dest: D,
    ) -> Self {
        Self {
            payload: Payload {
                channel,
                source,
                dest,
            },
        }
    }

    /// Waits for the transfer to finish
    pub fn wait(mut self) -> Result<Payload<'dma, T, D>, D::Error> {
        // There's an error interrupt status register. Maybe we should check
        // this here, but I have no idea whether that actually makes sense:
        // 1. As of this writing, we're not enabling any interrupts. I don't
        //    know if the flag would still be set in that case.
        // 2. The documentation is quiet about what could cause an error in the
        //    first place.
        //
        // This needs some further looking into.

        while self.payload.channel.active0.read().act().bits() & T::FLAG != 0 {}

        loop {
            match self.payload.dest.wait() {
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => break,

                Err(nb::Error::Other(error)) => {
                    compiler_fence(Ordering::SeqCst);
                    return Err(error);
                }
            }
        }

        compiler_fence(Ordering::SeqCst);

        Ok(self.payload)
    }
}

/// The payload of a `Transfer`
pub struct Payload<'dma, T, D>
where
    T: ChannelTrait,
{
    /// The channel used for this transfer
    pub channel: Channel<T, Enabled<&'dma Handle>>,

    /// The source of the transfer
    pub source: &'static mut [u8],

    /// The destination of the transfer
    pub dest: D,
}

/// A destination for a DMA transfer
pub trait Dest {
    /// The error that can occur while waiting for the destination to be idle
    type Error;

    /// Wait for the destination to be idle
    fn wait(&mut self) -> nb::Result<(), Self::Error>;

    /// The last byte of the destination's memory range
    fn end_addr(&mut self) -> *mut u8;
}
