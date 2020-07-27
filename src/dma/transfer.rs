use core::{
    fmt,
    sync::atomic::{compiler_fence, Ordering},
};

use crate::{
    init_state::Enabled,
    pac::dma0::channel::xfercfg::{DSTINC_A, SRCINC_A},
};

use super::{
    channels::{ChannelTrait, SharedRegisters},
    Channel,
};

/// A DMA transfer
pub struct Transfer<C, S, D>
where
    C: ChannelTrait,
{
    payload: Payload<C, S, D>,
}

impl<C, S, D> Transfer<C, S, D>
where
    C: ChannelTrait,
    S: Source,
    D: Dest,
{
    pub(super) fn new(
        channel: Channel<C, Enabled>,
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

    /// Start a DMA transfer
    ///
    /// # Panics
    ///
    /// Panics, if any buffer passed to this function has a length larger than
    /// 1024.
    ///
    /// # Limitations
    ///
    /// The caller must make sure to call this method only for the correct
    /// combination of channel and target.
    pub(crate) fn start(
        channel: Channel<C, Enabled>,
        source: S,
        mut dest: D,
    ) -> Self {
        assert!(source.is_valid());
        assert!(dest.is_valid());

        let registers = SharedRegisters::<C>::new();

        compiler_fence(Ordering::SeqCst);

        // To compute the transfer count, source or destination buffers need to
        // subtract 1 from their length. This early return makes sure that
        // this won't lead to an underflow.
        if source.is_empty() || dest.is_full() {
            return Transfer::new(channel, source, dest);
        }

        // Currently we don't support memory-to-memory transfers, which means
        // exactly one participant is providing the transfer count.
        let source_count = source.transfer_count();
        let dest_count = dest.transfer_count();
        let transfer_count = match (source_count, dest_count) {
            (Some(transfer_count), None) => transfer_count,
            (None, Some(transfer_count)) => transfer_count,
            _ => {
                panic!("Unsupported transfer type");
            }
        };

        // Configure channel
        // See user manual, section 12.6.16.
        channel.cfg.write(|w| {
            w.periphreqen().enabled();
            w.hwtrigen().disabled();
            unsafe { w.chpriority().bits(0) }
        });

        // Set channel transfer configuration
        // See user manual, section 12.6.18.
        channel.xfercfg.write(|w| {
            w.cfgvalid().valid();
            w.reload().disabled();
            w.swtrig().not_set();
            w.clrtrig().cleared();
            w.setinta().no_effect();
            w.setintb().no_effect();
            w.width().bit_8();
            w.srcinc().variant(source.increment());
            w.dstinc().variant(dest.increment());
            unsafe { w.xfercount().bits(transfer_count) }
        });

        // Configure channel descriptor
        // See user manual, sections 12.5.2 and 12.5.3.
        channel.descriptor.source_end = source.end_addr();
        channel.descriptor.dest_end = dest.end_addr();

        // Enable channel
        // See user manual, section 12.6.4.
        registers.enable();

        // Trigger transfer
        registers.trigger();

        Transfer::new(channel, source, dest)
    }

    /// Waits for the transfer to finish
    pub fn wait(
        mut self,
    ) -> Result<Payload<C, S, D>, (D::Error, Payload<C, S, D>)> {
        // There's an error interrupt status register. Maybe we should check
        // this here, but I have no idea whether that actually makes sense:
        // 1. As of this writing, we're not enabling any interrupts. I don't
        //    know if the flag would still be set in that case.
        // 2. The documentation is quiet about what could cause an error in the
        //    first place.
        //
        // This needs some further looking into.

        let registers = SharedRegisters::<C>::new();

        while registers.is_active() {}

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
pub struct Payload<C, S, D>
where
    C: ChannelTrait,
{
    /// The channel used for this transfer
    pub channel: Channel<C, Enabled>,

    /// The source of the transfer
    pub source: S,

    /// The destination of the transfer
    pub dest: D,
}

impl<C, S, D> fmt::Debug for Payload<C, S, D>
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
