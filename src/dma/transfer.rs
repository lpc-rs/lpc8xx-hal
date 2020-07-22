use core::sync::atomic::{compiler_fence, Ordering};

use crate::init_state::Enabled;

use super::{channels::ChannelTrait, Channel, Handle};

/// A DMA transfer
pub struct Transfer<'dma, T, D>
where
    T: ChannelTrait,
{
    pub(super) channel: Channel<T, Enabled<&'dma Handle>>,
    pub(super) source: &'static mut [u8],
    pub(super) dest: D,
}

impl<'dma, T, D> Transfer<'dma, T, D>
where
    T: ChannelTrait,
    D: Dest,
{
    /// Waits for the transfer to finish
    pub fn wait(
        mut self,
    ) -> Result<
        (Channel<T, Enabled<&'dma Handle>>, &'static mut [u8], D),
        D::Error,
    > {
        // There's an error interrupt status register. Maybe we should check
        // this here, but I have no idea whether that actually makes sense:
        // 1. As of this writing, we're not enabling any interrupts. I don't
        //    know if the flag would still be set in that case.
        // 2. The documentation is quiet about what could cause an error in the
        //    first place.
        //
        // This needs some further looking into.

        while self.channel.active0.read().act().bits() & T::FLAG != 0 {}

        loop {
            match self.dest.wait() {
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => break,

                Err(nb::Error::Other(error)) => {
                    compiler_fence(Ordering::SeqCst);
                    return Err(error);
                }
            }
        }

        compiler_fence(Ordering::SeqCst);

        Ok((self.channel, self.source, self.dest))
    }
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
