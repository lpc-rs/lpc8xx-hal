use core::{
    fmt,
    sync::atomic::{compiler_fence, Ordering},
};

use crate::init_state::Enabled;

use super::{channels::ChannelTrait, Channel, Handle};

/// A DMA transfer
pub struct Transfer<'dma, C, S, D>
where
    C: ChannelTrait,
{
    payload: Payload<'dma, C, S, D>,
}

impl<'dma, C, S, D> Transfer<'dma, C, S, D>
where
    C: ChannelTrait,
    D: Dest,
{
    pub(super) fn new(
        channel: Channel<C, Enabled<&'dma Handle>>,
        source: S,
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
    pub fn wait(
        mut self,
    ) -> Result<Payload<'dma, C, S, D>, (D::Error, Payload<'dma, C, S, D>)>
    {
        // There's an error interrupt status register. Maybe we should check
        // this here, but I have no idea whether that actually makes sense:
        // 1. As of this writing, we're not enabling any interrupts. I don't
        //    know if the flag would still be set in that case.
        // 2. The documentation is quiet about what could cause an error in the
        //    first place.
        //
        // This needs some further looking into.

        while self.payload.channel.active0.read().act().bits() & C::FLAG != 0 {}

        loop {
            match self.payload.dest.wait() {
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => break,

                Err(nb::Error::Other(error)) => {
                    compiler_fence(Ordering::SeqCst);
                    return Err((error, self.payload));
                }
            }
        }

        compiler_fence(Ordering::SeqCst);

        Ok(self.payload)
    }
}

/// The payload of a `Transfer`
pub struct Payload<'dma, C, S, D>
where
    C: ChannelTrait,
{
    /// The channel used for this transfer
    pub channel: Channel<C, Enabled<&'dma Handle>>,

    /// The source of the transfer
    pub source: S,

    /// The destination of the transfer
    pub dest: D,
}

impl<'dma, C, S, D> fmt::Debug for Payload<'dma, C, S, D>
where
    C: ChannelTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Placeholder implementation. Trying to do this properly runs into many
        // hurdles in many places, mainly because `Debug` isn't available for
        // many svd2rust-generated types.
        write!(f, "Payload")
    }
}

/// A destination for a DMA transfer
pub trait Dest: crate::private::Sealed {
    /// The error that can occur while waiting for the destination to be idle
    type Error;

    /// Wait for the destination to be idle
    fn wait(&mut self) -> nb::Result<(), Self::Error>;

    /// The last byte of the destination's memory range
    fn end_addr(&mut self) -> *mut u8;
}
