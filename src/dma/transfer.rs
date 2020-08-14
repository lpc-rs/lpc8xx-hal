//! APIs related to DMA transfers

use core::{
    fmt,
    sync::atomic::{compiler_fence, Ordering},
};

use crate::{
    init_state::Enabled,
    pac::dma0::channel::xfercfg::{DSTINC_A, SRCINC_A},
};

use super::{
    channels::{Instance, SharedRegisters},
    Channel,
};

/// A DMA transfer
///
/// A `Transfer` instance is used to represent a DMA transfer that uses a
/// specific [`Channel`]. Instances of this can be acquired by calling a
/// `write_all` or `read_all` method of the peripheral that should be involved
/// in the transfer.
///
/// # Limitations
///
/// Currently, memory-to-memory transfers are not supported. If you need this
/// features, feel free to [comment on the respective GitHub issue].
///
/// [`Channel`]: ../struct.Channel.html
/// [comment on the respective GitHub issue]: https://github.com/lpc-rs/lpc8xx-hal/issues/125
pub struct Transfer<State, C, S, D>
where
    C: Instance,
{
    _state: State,
    payload: Payload<C, S, D>,
}

impl<C, S, D> Transfer<state::Ready, C, S, D>
where
    C: Instance,
    S: Source,
    D: Dest,
{
    /// Create a new DMA transfer
    ///
    /// # Panics
    ///
    /// Panics, if the length of any buffer passed to this function is 0 or
    /// larger than 1024.
    ///
    /// # Limitations
    ///
    /// The caller must make sure to call this method only for the correct
    /// combination of channel and target.
    pub(crate) fn new(
        channel: Channel<C, Enabled>,
        source: S,
        mut dest: D,
    ) -> Self {
        assert!(!source.is_empty());
        assert!(!dest.is_full());
        assert!(source.is_valid());
        assert!(dest.is_valid());

        compiler_fence(Ordering::SeqCst);

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

        Self {
            _state: state::Ready,
            payload: Payload {
                channel,
                source,
                dest,
            },
        }
    }

    /// Set INTA flag when this transfer is complete
    ///
    /// By default, the flag is not set. This method can be used to overwrite
    /// that setting. Setting the flag can be used to trigger an interrupt.
    ///
    /// This method is only available, if the `Transfer` is in the [`Ready`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Ready`]: state/struct.Ready.html
    pub fn set_a_when_complete(&mut self) {
        self.payload
            .channel
            .xfercfg
            .modify(|_, w| w.setinta().set())
    }

    /// Set INTB flag when this transfer is complete
    ///
    /// By default, the flag is not set. This method can be used to overwrite
    /// that setting. Setting the flag can be used to trigger an interrupt.
    ///
    /// This method is only available, if the `Transfer` is in the [`Ready`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Ready`]: state/struct.Ready.html
    pub fn set_b_when_complete(&mut self) {
        self.payload
            .channel
            .xfercfg
            .modify(|_, w| w.setintb().set())
    }

    /// Start the DMA transfer
    ///
    /// This method is only available, if the `Transfer` is in the [`Ready`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this `Transfer` instance and returns another one with its
    /// `State` parameter set to [`Started`].
    ///
    /// [`Ready`]: state/struct.Ready.html
    /// [`Started`]: state/struct.Started.html
    pub fn start(self) -> Transfer<state::Started, C, S, D> {
        let registers = SharedRegisters::<C>::new();

        // Reset all flags to make sure we don't still have one set from a
        // previous transfer.
        registers.reset_flags();

        // Enable channel
        // See user manual, section 12.6.4.
        registers.enable();

        // Trigger transfer
        registers.trigger();

        Transfer {
            _state: state::Started,
            payload: self.payload,
        }
    }
}

impl<C, S, D> Transfer<state::Started, C, S, D>
where
    C: Instance,
    S: Source,
    D: Dest,
{
    /// Indicates whether transfer is currently active
    ///
    /// Corresponds to the channel's flag in the ACTIVE0 register.
    ///
    /// This method is only available, if the `Transfer` is in the [`Started`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Started`]: state/struct.Started.html
    pub fn is_active(&self) -> bool {
        let registers = SharedRegisters::<C>::new();
        registers.is_active()
    }

    /// Indicates whether transfer is currently busy
    ///
    /// Corresponds to the channel's flag in the BUSY0 register.
    ///
    /// This method is only available, if the `Transfer` is in the [`Started`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Started`]: state/struct.Started.html
    pub fn is_busy(&self) -> bool {
        let registers = SharedRegisters::<C>::new();
        registers.is_busy()
    }

    /// Indicates whether the error interrupt fired
    ///
    /// Corresponds to the channel's flag in the ERRINT0 register.
    ///
    /// This method is only available, if the `Transfer` is in the [`Started`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Started`]: state/struct.Started.html
    pub fn error_interrupt_fired(&self) -> bool {
        let registers = SharedRegisters::<C>::new();
        registers.error_interrupt_fired()
    }

    /// Indicates whether interrupt A fired
    ///
    /// Corresponds to the channel's flag in the INTA0 register.
    ///
    /// This method is only available, if the `Transfer` is in the [`Started`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Started`]: state/struct.Started.html
    pub fn a_interrupt_fired(&self) -> bool {
        let registers = SharedRegisters::<C>::new();
        registers.a_interrupt_fired()
    }

    /// Indicates whether interrupt B fired
    ///
    /// Corresponds to the channel's flag in the INTB0 register.
    ///
    /// This method is only available, if the `Transfer` is in the [`Started`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// [`Started`]: state/struct.Started.html
    pub fn b_interrupt_fired(&self) -> bool {
        let registers = SharedRegisters::<C>::new();
        registers.b_interrupt_fired()
    }

    /// Waits for the transfer to finish
    ///
    /// This method will block until the transfer is finished. If this is not
    /// acceptable, you can enable an interrupt for the channel, and/or check
    /// the channel state with the [`is_active`] method.
    ///
    /// This method is only available, if the `Transfer` is in the [`Started`]
    /// state. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this instance of `Transfer` and returns the transfer payload,
    /// which contains all resources that were held by this transfer.
    ///
    /// [`is_active`]: #method.is_active
    /// [`Started`]: state/struct.Started.html
    pub fn wait(
        mut self,
    ) -> Result<Payload<C, S, D>, (Error<S::Error, D::Error>, Payload<C, S, D>)>
    {
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
            match self.payload.source.finish() {
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => break,

                Err(nb::Error::Other(error)) => {
                    compiler_fence(Ordering::SeqCst);
                    return Err((Error::Source(error), self.payload));
                }
            }
        }
        loop {
            match self.payload.dest.finish() {
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => break,

                Err(nb::Error::Other(error)) => {
                    compiler_fence(Ordering::SeqCst);
                    return Err((Error::Dest(error), self.payload));
                }
            }
        }

        compiler_fence(Ordering::SeqCst);

        Ok(self.payload)
    }
}

/// Error that can occur while waiting for the DMA transfer to finish
#[derive(Debug)]
pub enum Error<S, D> {
    /// An error occured while finishing the transfer at the source
    Source(S),

    /// An error occured while finishing the transfer at the destination
    Dest(D),
}

/// The payload of a [`Transfer`]
///
/// These are resources that must be moved into a [`Transfer`] while it is going
/// on, and will be returned to the user once it has finished.
///
/// [`Transfer`]: struct.Transfer.html
pub struct Payload<C, S, D>
where
    C: Instance,
{
    /// The channel used for this transfer
    pub channel: Channel<C, Enabled>,

    /// The source of the transfer
    ///
    /// Can be a peripheral or a buffer.
    pub source: S,

    /// The destination of the transfer
    ///
    /// Can be a peripheral or a buffer.
    pub dest: D,
}

impl<C, S, D> fmt::Debug for Payload<C, S, D>
where
    C: Instance,
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
/// This trait's methods are intended for internal use only. It is implemented
/// for immutable static buffers and peripherals that support being read from
/// using DMA.
pub trait Source: crate::private::Sealed {
    /// The error that can occur while finishing the transfer
    type Error;

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

    /// Tell the source to finish the transfer
    fn finish(&mut self) -> nb::Result<(), Self::Error>;
}

/// A destination for a DMA transfer
///
/// This trait's methods are intended for internal use only. It is implemented
/// for mutable static buffers and peripherals that support being written to
/// using DMA.
pub trait Dest: crate::private::Sealed {
    /// The error that can occur while finishing the transfer
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

    /// The end address
    ///
    /// This is not the actual end of the buffer, but the starting address plus
    /// `transfer_count` times address increment. See LPC845 user manual,
    /// section 16.5.2, for example.
    fn end_addr(&mut self) -> *mut u8;

    /// Tell the destination to finish the transfer
    fn finish(&mut self) -> nb::Result<(), Self::Error>;
}

/// Types representing the states of a DMA transfer
pub mod state {
    /// Indicates that a transfer is ready to be started
    ///
    /// Used for the `State` type parameter of [`Transfer`].
    ///
    /// [`Transfer`]: ../struct.Transfer.html
    pub struct Ready;

    /// Indicates that a transfer has been started
    ///
    /// Used for the `State` type parameter of [`Transfer`].
    ///
    /// [`Transfer`]: ../struct.Transfer.html
    pub struct Started;
}
