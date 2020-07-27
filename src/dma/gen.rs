use crate::{
    init_state::{Disabled, Enabled},
    pac::{
        self,
        dma0::channel::{CFG, XFERCFG},
    },
    reg_proxy::RegProxy,
};

use super::{
    channels::{self, Channel},
    descriptors::DescriptorTable,
};

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

            impl channels::Instance for $name {
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
