use core::{
    fmt,
    sync::atomic::{compiler_fence, Ordering},
};

use crate::{
    init_state::Enabled,
    pac::dma0::channel::xfercfg::{DSTINC_A, SRCINC_A},
};

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

/// The source of a DMA transfer
///
/// This trait is intended for internal use only. It is implemented for
/// immutable static buffers and peripherals that support being read from using
/// DMA.
pub trait Source: crate::private::Sealed {
    /// Indicates whether the source is valid
    ///
    /// Buffers are valid, if they have a length of 1024 or less. Peripherals
    /// are always valid.
    fn is_valid(&self) -> bool;

    /// Indicates whether the source is empty
    ///
    /// Buffers are empty, if they have a length of 0. Peripherals are never
    /// empty.
    fn is_empty(&self) -> bool;

    /// The address increment during the transfer
    ///
    /// Buffers will return the word size here. Peripherals will indicate no
    /// increment.
    fn increment(&self) -> SRCINC_A;

    /// The transfer count, as defined by XFERCFG.XFERCOUNT
    ///
    /// Only buffers will return a value here, and only if `is_empty` returns
    /// false. Peripherals will always return `None`.
    fn transfer_count(&self) -> Option<u16>;

    /// The end address
    ///
    /// This is not the actual end of the buffer, but the starting address plus
    /// `transfer_count` times address increment. See LPC845 user manual,
    /// section 16.5.2, for example.
    fn end_addr(&self) -> *const u8;
}

/// A destination for a DMA transfer
///
/// This trait is intended for internal use only. It is implemented for mutable
/// static buffers and peripherals that support being written to using DMA.
pub trait Dest: crate::private::Sealed {
    /// The error that can occur while waiting for the destination to be idle
    type Error;

    /// Indicates whether the destination is valid
    ///
    /// Buffers are valid if they have a length of 1024 or less. Peripherals are
    /// always valid.
    fn is_valid(&self) -> bool;

    /// Indicates whether the destination is full
    ///
    /// Buffers are empty, if they have a length of 0. Peripherals are never
    /// empty.
    fn is_full(&self) -> bool;

    /// The address increment during the transfer
    ///
    /// Buffers will return the word size here. Peripherals will indicate no
    /// increment.
    fn increment(&self) -> DSTINC_A;

    /// The transfer count, as defined by XFERCFG.XFERCOUNT
    ///
    /// Only buffers will return a value here, and only if `if_full` returns
    /// `false`. Peripherals will always return `None`.
    fn transfer_count(&self) -> Option<u16>;

    /// Wait for the destination to be idle
    fn wait(&mut self) -> nb::Result<(), Self::Error>;

    /// The end address
    ///
    /// This is not the actual end of the buffer, but the starting address plus
    /// `transfer_count` times address increment. See LPC845 user manual,
    /// section 16.5.2, for example.
    fn end_addr(&mut self) -> *mut u8;
}
