//! APIs related to DMA channels

use core::marker::PhantomData;

use crate::{
    init_state::{Disabled, Enabled},
    pac::{
        self,
        dma0::{
            channel::{CFG, XFERCFG},
            ACTIVE0, BUSY0, ENABLESET0, ERRINT0, INTA0, INTB0, SETTRIG0,
        },
    },
    reg_proxy::{Reg, RegProxy},
};

use super::descriptors::ChannelDescriptor;

/// A DMA channel
pub struct Channel<C, S>
where
    C: Instance,
{
    pub(super) ty: C,
    pub(super) _state: S,
    pub(super) descriptor: &'static mut ChannelDescriptor,

    // This channel's dedicated registers.
    pub(super) cfg: RegProxy<C::Cfg>,
    pub(super) xfercfg: RegProxy<C::Xfercfg>,
}

impl<C> Channel<C, Disabled>
where
    C: Instance,
{
    /// Enable the channel
    pub(super) fn enable(self) -> Channel<C, Enabled> {
        Channel {
            ty: self.ty,
            _state: Enabled(()),
            descriptor: self.descriptor,

            cfg: self.cfg,
            xfercfg: self.xfercfg,
        }
    }
}

impl<C> Channel<C, Enabled>
where
    C: Instance,
{
    /// Disable the channel
    pub(super) fn disable(self) -> Channel<C, Disabled> {
        Channel {
            ty: self.ty,
            _state: Disabled,
            descriptor: self.descriptor,

            cfg: self.cfg,
            xfercfg: self.xfercfg,
        }
    }
}

/// Implemented for each DMA channel
pub trait Instance {
    /// The index of the channel
    ///
    /// This is `0` for channel 0, `1` for channel 1, etc.
    const INDEX: usize;

    /// The flag for the channel
    ///
    /// This is `0x1` for channel 0, `0x2` for channel 2, `0x4` for channel 3,
    /// etc.
    const FLAG: u32;

    /// The type that represents this channel's CFG register
    type Cfg: Reg<Target = CFG>;

    /// The type that represents this channel's XFERCFG register
    type Xfercfg: Reg<Target = XFERCFG>;
}

pub(super) struct SharedRegisters<C> {
    active0: &'static ACTIVE0,
    busy0: &'static BUSY0,
    enableset0: &'static ENABLESET0,
    errint0: &'static ERRINT0,
    inta0: &'static INTA0,
    intb0: &'static INTB0,
    settrig0: &'static SETTRIG0,

    _channel: PhantomData<C>,
}

impl<C> SharedRegisters<C>
where
    C: Instance,
{
    pub(super) fn new() -> Self {
        // This is sound, for the following reasons:
        // - We only acccess stateless registers.
        // - Since we're dealing with MMIO registers, dereferencing and taking
        //   `'static` references is always okay.
        unsafe {
            let registers = pac::DMA0::ptr();

            Self {
                active0: &(*registers).active0,
                busy0: &(*registers).busy0,
                enableset0: &(*registers).enableset0,
                errint0: &(*registers).errint0,
                inta0: &(*registers).inta0,
                intb0: &(*registers).intb0,
                settrig0: &(*registers).settrig0,

                _channel: PhantomData,
            }
        }
    }

    pub(super) fn enable(&self) {
        self.enableset0.write(|w| {
            // Sound, as all values assigned to `C::FLAG` are valid here.
            unsafe { w.ena().bits(C::FLAG) }
        });
    }

    pub(super) fn trigger(&self) {
        self.settrig0.write(|w| {
            // Sound, as all values assigned to `C::FLAG` are valid here.
            unsafe { w.trig().bits(C::FLAG) }
        });
    }

    pub(super) fn is_active(&self) -> bool {
        self.active0.read().act().bits() & C::FLAG != 0
    }

    pub(super) fn is_busy(&self) -> bool {
        self.busy0.read().bsy().bits() & C::FLAG != 0
    }

    pub(super) fn error_interrupt_fired(&self) -> bool {
        self.errint0.read().err().bits() & C::FLAG != 0
    }

    pub(super) fn a_interrupt_fired(&self) -> bool {
        self.inta0.read().ia().bits() & C::FLAG != 0
    }

    pub(super) fn b_interrupt_fired(&self) -> bool {
        self.intb0.read().ib().bits() & C::FLAG != 0
    }
}
