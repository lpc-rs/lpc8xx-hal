use core::marker::PhantomData;

use crate::{
    init_state::{Disabled, Enabled},
    pac::{
        self,
        dma0::{
            channel::{CFG, XFERCFG},
            ACTIVE0, ENABLESET0, SETTRIG0,
        },
    },
    reg_proxy::{Reg, RegProxy},
};

use super::descriptors::{ChannelDescriptor, DescriptorTable};

/// A DMA channel
pub struct Channel<C, S>
where
    C: ChannelTrait,
{
    ty: C,
    _state: S,
    pub(super) descriptor: &'static mut ChannelDescriptor,

    // This channel's dedicated registers.
    pub(super) cfg: RegProxy<C::Cfg>,
    pub(super) xfercfg: RegProxy<C::Xfercfg>,
}

impl<C> Channel<C, Disabled>
where
    C: ChannelTrait,
{
    /// Enable the channel
    fn enable(self) -> Channel<C, Enabled> {
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
    C: ChannelTrait,
{
    /// Disable the channel
    fn disable(self) -> Channel<C, Disabled> {
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
pub trait ChannelTrait {
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

macro_rules! channels {
    ($($field:ident, $name:ident, $index:expr, $cfg:ident, $xfercfg:ident;)*) => {
        /// Provides access to all channels
        #[allow(missing_docs)]
        pub struct Channels<State> {
            $(pub $field: Channel<$name, State>,)*
        }

        impl Channels<Disabled> {
            pub(super) fn new(descriptors: &'static mut DescriptorTable)
                -> Self
            {
                let mut descriptors = (&mut descriptors.0).into_iter();

                Channels {
                    $(
                        $field: Channel {
                            ty        : $name(()),
                            _state    : Disabled,
                            descriptor: descriptors.next().unwrap(),

                            cfg    : RegProxy::new(),
                            xfercfg: RegProxy::new(),
                        },
                    )*
                }
            }

            pub(super) fn enable(self) -> Channels<Enabled> {
                Channels {
                    $(
                        $field: self.$field.enable(),
                    )*
                }
            }
        }

        impl Channels<Enabled> {
            pub(super) fn disable(self) -> Channels<Disabled> {
                Channels {
                    $(
                        $field: self.$field.disable(),
                    )*
                }
            }
        }


        $(
            /// This struct is an implementation detail that shouldn't be used by user
            pub struct $xfercfg;

            reg_cluster!($xfercfg, XFERCFG, pac::DMA0, $field, xfercfg);

            /// This struct is an implementation detail that shouldn't be used by user
            pub struct $cfg;

            reg_cluster!($cfg, CFG, pac::DMA0, $field, cfg);

            /// Identifies a DMA channel
            pub struct $name(());

            impl ChannelTrait for $name {
                const INDEX: usize = $index;
                const FLAG : u32   = 0x1 << Self::INDEX;

                type Cfg     = $cfg;
                type Xfercfg = $xfercfg;
            }
        )*
    }
}

#[cfg(feature = "82x")]
// The channels must always be specified in order, from lowest to highest, as
// the channel descriptors are assigned based on that order.
channels!(
    channel0 , Channel0 ,  0, CFG0 , XFERCFG0 ;
    channel1 , Channel1 ,  1, CFG1 , XFERCFG1 ;
    channel2 , Channel2 ,  2, CFG2 , XFERCFG2 ;
    channel3 , Channel3 ,  3, CFG3 , XFERCFG3 ;
    channel4 , Channel4 ,  4, CFG4 , XFERCFG4 ;
    channel5 , Channel5 ,  5, CFG5 , XFERCFG5 ;
    channel6 , Channel6 ,  6, CFG6 , XFERCFG6 ;
    channel7 , Channel7 ,  7, CFG7 , XFERCFG7 ;
    channel8 , Channel8 ,  8, CFG8 , XFERCFG8 ;
    channel9 , Channel9 ,  9, CFG9 , XFERCFG9 ;
    channel10, Channel10, 10, CFG10, XFERCFG10;
    channel11, Channel11, 11, CFG11, XFERCFG11;
    channel12, Channel12, 12, CFG12, XFERCFG12;
    channel13, Channel13, 13, CFG13, XFERCFG13;
    channel14, Channel14, 14, CFG14, XFERCFG14;
    channel15, Channel15, 15, CFG15, XFERCFG15;
    channel16, Channel16, 16, CFG16, XFERCFG16;
    channel17, Channel17, 17, CFG17, XFERCFG17;
);

#[cfg(feature = "845")]
// The channels must always be specified in order, from lowest to highest, as
// the channel descriptors are assigned based on that order.
channels!(
    channel0 , Channel0 ,  0, CFG0 , XFERCFG0 ;
    channel1 , Channel1 ,  1, CFG1 , XFERCFG1 ;
    channel2 , Channel2 ,  2, CFG2 , XFERCFG2 ;
    channel3 , Channel3 ,  3, CFG3 , XFERCFG3 ;
    channel4 , Channel4 ,  4, CFG4 , XFERCFG4 ;
    channel5 , Channel5 ,  5, CFG5 , XFERCFG5 ;
    channel6 , Channel6 ,  6, CFG6 , XFERCFG6 ;
    channel7 , Channel7 ,  7, CFG7 , XFERCFG7 ;
    channel8 , Channel8 ,  8, CFG8 , XFERCFG8 ;
    channel9 , Channel9 ,  9, CFG9 , XFERCFG9 ;
    channel10, Channel10, 10, CFG10, XFERCFG10;
    channel11, Channel11, 11, CFG11, XFERCFG11;
    channel12, Channel12, 12, CFG12, XFERCFG12;
    channel13, Channel13, 13, CFG13, XFERCFG13;
    channel14, Channel14, 14, CFG14, XFERCFG14;
    channel15, Channel15, 15, CFG15, XFERCFG15;
    channel16, Channel16, 16, CFG16, XFERCFG16;
    channel17, Channel17, 17, CFG17, XFERCFG17;
    channel18, Channel18, 18, CFG18, XFERCFG18;
    channel19, Channel19, 19, CFG19, XFERCFG19;
    channel20, Channel20, 20, CFG20, XFERCFG20;
    channel21, Channel21, 21, CFG21, XFERCFG21;
    channel22, Channel22, 22, CFG22, XFERCFG22;
    channel23, Channel23, 23, CFG23, XFERCFG23;
    channel24, Channel24, 24, CFG24, XFERCFG24;
);

pub(super) struct SharedRegisters<C> {
    active0: &'static ACTIVE0,
    enableset0: &'static ENABLESET0,
    settrig0: &'static SETTRIG0,

    _channel: PhantomData<C>,
}

impl<C> SharedRegisters<C>
where
    C: ChannelTrait,
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
                enableset0: &(*registers).enableset0,
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
}
