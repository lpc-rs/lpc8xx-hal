//! API for Direct Memory Access (DMA)
//!
//! The DMA controller is described in the user manual, chapter 12.


use core::ptr;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};

use nb;

use init_state;
use raw;
use raw::dma::{
    ACTIVE0,
    CFG,
    ENABLESET0,
    SETTRIG0,
    XFERCFG,
};
use reg_proxy::{
    Reg,
    RegProxy,
};
use syscon;


/// Entry point to the DMA API
pub struct DMA {
    dma: raw::DMA,
}

impl DMA {
    pub(crate) fn new(dma: raw::DMA) -> Self {
        DMA { dma }
    }

    /// Splits the DMA API into its component parts
    ///
    /// This is the regular way to access the DMA API. It exists as an explicit
    /// step, as it's no longer possible to gain access to the raw peripheral
    /// using [`DMA::free`] after you've called this method.
    pub fn split(self) -> Parts {
        Parts {
            handle  : Handle::new(self.dma),
            channels: Channels::new(),
        }
    }

    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/braun-robotics/rust-lpc82x-hal/issues
    pub fn free(self) -> raw::DMA {
        self.dma
    }
}

/// The main API for the DMA controller
///
/// Provides access to all types that make up the DMA API. Please refer to the
/// [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts {
    /// Handle to the DMA controller
    pub handle: Handle<init_state::Disabled>,

    /// The DMA channels
    pub channels: Channels,
}


/// Handle to the DMA controller
pub struct Handle<State = init_state::Enabled<&'static mut DescriptorTable>> {
    _state: State,
    dma   : raw::DMA,
}

impl Handle<init_state::Disabled> {
    pub(crate) fn new(dma: raw::DMA) -> Self {
        Handle {
            _state: init_state::Disabled,
            dma   : dma,
        }
    }
}

impl<'dma> Handle<init_state::Disabled> {
    /// Enable the DMA controller
    pub fn enable(self,
        descriptors: &'static mut DescriptorTable,
        syscon     : &mut syscon::Handle,
    )
        -> Handle<init_state::Enabled<&'static mut DescriptorTable>>
    {
        syscon.enable_clock(&self.dma);

        // Set descriptor table address
        //
        // See user manual, section 12.6.3.
        self.dma.srambase.write(|w|
            unsafe { w.bits(descriptors as *const _ as u32) }
        );

        // Enable the DMA controller
        //
        // See user manual, section 12.6.1.
        self.dma.ctrl.write(|w| w.enable().enabled());

        Handle {
            _state: init_state::Enabled(descriptors),
            dma   : self.dma,
        }
    }
}

impl Handle<init_state::Enabled> {
    /// Disable the DMA controller
    pub fn disable(self, syscon: &mut syscon::Handle)
        -> Handle<init_state::Disabled>
    {
        syscon.disable_clock(&self.dma);

        Handle {
            _state: init_state::Disabled,
            dma   : self.dma,
        }
    }
}


/// The channel descriptor table
///
/// Contains a descriptor for each DMA channel.
#[repr(C, align(512))]
pub struct DescriptorTable([ChannelDescriptor; 18]);

impl DescriptorTable {
    /// Create a new channel descriptor table
    pub const fn new() -> Self {
        DescriptorTable([
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
            ChannelDescriptor::new(),
        ])
    }
}


#[repr(C, align(16))]
struct ChannelDescriptor {
    config    : u32,
    source_end: *const u8,
    dest_end  : *mut u8,
    next_desc : *const ChannelDescriptor,
}

impl ChannelDescriptor {
    const fn new() -> Self {
        ChannelDescriptor {
            config    : 0,
            source_end: ptr::null(),
            dest_end  : ptr::null_mut(),
            next_desc : ptr::null(),
        }
    }
}

// `ChannelDescriptor` contains raw pointers, therefore `Send` is not derived
// automatically. I really see no reason why `ChannelDescriptor` shouldn't be
// `Send` though, and it needs to be `Send`, so one can put it into a
// `cortex_m::interrupt::Mutex`.
unsafe impl Send for ChannelDescriptor {}


/// A DMA channel
pub struct Channel<T> where T: ChannelTrait {
    _ty: T,

    // This channel's dedicated registers.
    cfg    : RegProxy<T::Cfg>,
    xfercfg: RegProxy<T::Xfercfg>,

    // Shared registers. We restrict our access to the one bit that is dedicated
    // to this channel, so sharing those with other channels should be safe.
    active0   : RegProxy<ACTIVE0>,
    enableset0: RegProxy<ENABLESET0>,
    settrig0  : RegProxy<SETTRIG0>,
}

impl<T> Channel<T> where T: ChannelTrait {
    /// Starts a DMA transfer
    ///
    /// # Limitations
    ///
    /// The length of `source` must be 1024 or less.
    pub fn start_transfer<D>(self,
            dma   : &mut Handle,
            source: &'static mut [u8],
        mut dest  : D,
    )
        -> Transfer<T, D>
        where D: Dest
    {
        compiler_fence(Ordering::SeqCst);

        // We need to substract 1 from the length below. If the source is empty,
        // return early to prevent underflow.
        if source.len() == 0 {
            return Transfer {
                channel: self,
                source : source,
                dest   : dest,
            }
        }

        // Configure channel 1 (has request input USART0_TX_DMA)
        // See user manual, section 12.6.16.
        self.cfg.write(|w| {
            let w = w
                .periphreqen().enabled()
                .hwtrigen().disabled()
                .trigburst().single_transfer();
            unsafe { w.chpriority().bits(0) }
        });

        // Set channel transfer configuration
        // See user manual, section 12.6.18.
        self.xfercfg.write(|w| {
            let w = w
                .cfgvalid().valid()
                .reload().disabled()
                .swtrig().notset()
                .clrtrig().cleared()
                .setinta().no_effect()
                .setintb().no_effect()
                .width()._8_bit_transfers_are()
                .srcinc()._1_x_width()
                .dstinc().no_increment();
            unsafe { w.xfercount().bits(source.len() as u16 - 1) }
        });

        let source_end = unsafe { source.as_ptr().add(source.len() - 1) };

        // Configure channel descriptor
        // See user manual, sections 12.5.2 and 12.5.3.
        (dma._state.0).0[T::INDEX].source_end = source_end;
        (dma._state.0).0[T::INDEX].dest_end   = dest.end_addr();

        // Enable channel 1
        // See user manual, section 12.6.4.
        self.enableset0.write(|w| unsafe { w.ena().bits(T::FLAG) });

        // Trigger transfer
        self.settrig0.write(|w| unsafe { w.trig().bits(T::FLAG) });

        Transfer {
            channel: self,
            source : source,
            dest   : dest,
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
    const FLAG : u32;

    /// The type that represents this channel's CFG register
    type Cfg: Reg<Target=CFG>;

    /// The type that represents this channel's XFERCFG register
    type Xfercfg: Reg<Target=XFERCFG>;
}

macro_rules! channels {
    ($($field:ident, $name:ident, $index:expr, $cfg:ty, $xfercfg:ty;)*) => {
        /// Provides access to all channels
        #[allow(missing_docs)]
        pub struct Channels {
            $(pub $field: Channel<$name>,)*
        }

        impl Channels {
            fn new() -> Self {
                Channels {
                    $(
                        $field: Channel {
                            _ty: $name(()),

                            cfg    : RegProxy::new(),
                            xfercfg: RegProxy::new(),

                            active0   : RegProxy::new(),
                            enableset0: RegProxy::new(),
                            settrig0  : RegProxy::new(),
                        },
                    )*
                }
            }
        }


        $(
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

channels!(
    channel_0 , Channel0 ,  0, CFG0 , XFERCFG0 ;
    channel_1 , Channel1 ,  1, CFG1 , XFERCFG1 ;
    channel_2 , Channel2 ,  2, CFG2 , XFERCFG2 ;
    channel_3 , Channel3 ,  3, CFG3 , XFERCFG3 ;
    channel_4 , Channel4 ,  4, CFG4 , XFERCFG4 ;
    channel_5 , Channel5 ,  5, CFG5 , XFERCFG5 ;
    channel_6 , Channel6 ,  6, CFG6 , XFERCFG6 ;
    channel_7 , Channel7 ,  7, CFG7 , XFERCFG7 ;
    channel_8 , Channel8 ,  8, CFG8 , XFERCFG8 ;
    channel_9 , Channel9 ,  9, CFG9 , XFERCFG9 ;
    channel_10, Channel10, 10, CFG10, XFERCFG10;
    channel_11, Channel11, 11, CFG11, XFERCFG11;
    channel_12, Channel12, 12, CFG12, XFERCFG12;
    channel_13, Channel13, 13, CFG13, XFERCFG13;
    channel_14, Channel14, 14, CFG14, XFERCFG14;
    channel_15, Channel15, 15, CFG15, XFERCFG15;
    channel_16, Channel16, 16, CFG16, XFERCFG16;
    channel_17, Channel17, 17, CFG17, XFERCFG17;
);


/// A destination for a DMA transfer
pub trait Dest {
    /// The error that can occur while waiting for the destination to be idle
    type Error;

    /// Wait for the destination to be idle
    fn wait(&mut self) -> nb::Result<(), Self::Error>;

    /// The last byte of the destination's memory range
    fn end_addr(&mut self) -> *mut u8;
}


/// A DMA transfer
pub struct Transfer<T, D> where T: ChannelTrait {
    channel: Channel<T>,
    source : &'static mut [u8],
    dest   : D,
}

impl<T, D> Transfer<T, D>
    where
        T: ChannelTrait,
        D: Dest,
{
    /// Waits for the transfer to finish
    pub fn wait(mut self)
        -> Result<(Channel<T>, &'static mut [u8], D), D::Error>
    {
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
                Ok(())                     => break,

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


/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG0;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG1;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG2;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG3;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG4;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG5;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG6;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG7;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG8;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG9;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG10;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG11;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG12;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG13;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG14;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG15;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG16;

/// This struct is an implementation detail that shouldn't be used by user
pub struct CFG17;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG0;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG1;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG2;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG3;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG4;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG5;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG6;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG7;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG8;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG9;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG10;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG11;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG12;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG13;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG14;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG15;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG16;

/// This struct is an implementation detail that shouldn't be used by user
pub struct XFERCFG17;


reg!(CFG0 , CFG, raw::DMA, cfg0 );
reg!(CFG1 , CFG, raw::DMA, cfg1 );
reg!(CFG2 , CFG, raw::DMA, cfg2 );
reg!(CFG3 , CFG, raw::DMA, cfg3 );
reg!(CFG4 , CFG, raw::DMA, cfg4 );
reg!(CFG5 , CFG, raw::DMA, cfg5 );
reg!(CFG6 , CFG, raw::DMA, cfg6 );
reg!(CFG7 , CFG, raw::DMA, cfg7 );
reg!(CFG8 , CFG, raw::DMA, cfg8 );
reg!(CFG9 , CFG, raw::DMA, cfg9 );
reg!(CFG10, CFG, raw::DMA, cfg10);
reg!(CFG11, CFG, raw::DMA, cfg11);
reg!(CFG12, CFG, raw::DMA, cfg12);
reg!(CFG13, CFG, raw::DMA, cfg13);
reg!(CFG14, CFG, raw::DMA, cfg14);
reg!(CFG15, CFG, raw::DMA, cfg15);
reg!(CFG16, CFG, raw::DMA, cfg16);
reg!(CFG17, CFG, raw::DMA, cfg17);

reg!(XFERCFG0 , XFERCFG, raw::DMA, xfercfg0 );
reg!(XFERCFG1 , XFERCFG, raw::DMA, xfercfg1 );
reg!(XFERCFG2 , XFERCFG, raw::DMA, xfercfg2 );
reg!(XFERCFG3 , XFERCFG, raw::DMA, xfercfg3 );
reg!(XFERCFG4 , XFERCFG, raw::DMA, xfercfg4 );
reg!(XFERCFG5 , XFERCFG, raw::DMA, xfercfg5 );
reg!(XFERCFG6 , XFERCFG, raw::DMA, xfercfg6 );
reg!(XFERCFG7 , XFERCFG, raw::DMA, xfercfg7 );
reg!(XFERCFG8 , XFERCFG, raw::DMA, xfercfg8 );
reg!(XFERCFG9 , XFERCFG, raw::DMA, xfercfg9 );
reg!(XFERCFG10, XFERCFG, raw::DMA, xfercfg10);
reg!(XFERCFG11, XFERCFG, raw::DMA, xfercfg11);
reg!(XFERCFG12, XFERCFG, raw::DMA, xfercfg12);
reg!(XFERCFG13, XFERCFG, raw::DMA, xfercfg13);
reg!(XFERCFG14, XFERCFG, raw::DMA, xfercfg14);
reg!(XFERCFG15, XFERCFG, raw::DMA, xfercfg15);
reg!(XFERCFG16, XFERCFG, raw::DMA, xfercfg16);
reg!(XFERCFG17, XFERCFG, raw::DMA, xfercfg17);

reg!(ACTIVE0   , ACTIVE0   , raw::DMA, active0   );
reg!(ENABLESET0, ENABLESET0, raw::DMA, enableset0);
reg!(SETTRIG0  , SETTRIG0  , raw::DMA, settrig0  );
