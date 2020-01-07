//! API for Direct Memory Access (DMA)
//!
//! The DMA controller is described in the user manual, chapter 12.

use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

use nb;

use crate::{
    init_state,
    pac::{
        self,
        dma0::{
            channel::{CFG, XFERCFG},
            ACTIVE0, ENABLESET0, SETTRIG0,
        },
    },
    reg_proxy::{Reg, RegProxy},
    syscon,
};

/// Entry point to the DMA API
pub struct DMA {
    dma: pac::DMA0,
}

impl DMA {
    pub(crate) fn new(dma: pac::DMA0) -> Self {
        DMA { dma }
    }

    /// Splits the DMA API into its component parts
    ///
    /// This is the regular way to access the DMA API. It exists as an explicit
    /// step, as it's no longer possible to gain access to the raw peripheral
    /// using [`DMA::free`] after you've called this method.
    pub fn split(self, descriptors: &'static mut DescriptorTable) -> Parts {
        let srambase = descriptors as *mut _ as u32;

        Parts {
            handle: Handle::new(self.dma, srambase),
            channels: Channels::new(descriptors),
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
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> pac::DMA0 {
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
pub struct Handle<State = init_state::Enabled> {
    _state: State,
    dma: pac::DMA0,
    srambase: u32,
}

impl Handle<init_state::Disabled> {
    pub(crate) fn new(dma: pac::DMA0, srambase: u32) -> Self {
        Handle {
            _state: init_state::Disabled,
            dma,
            srambase,
        }
    }
}

impl<'dma> Handle<init_state::Disabled> {
    /// Enable the DMA controller
    pub fn enable(self, syscon: &mut syscon::Handle) -> Handle<init_state::Enabled> {
        syscon.enable_clock(&self.dma);

        // Set descriptor table address
        //
        // See user manual, section 12.6.3.
        self.dma
            .srambase
            .write(|w| unsafe { w.bits(self.srambase) });

        // Enable the DMA controller
        //
        // See user manual, section 12.6.1.
        self.dma.ctrl.write(|w| w.enable().enabled());

        Handle {
            _state: init_state::Enabled(()),
            dma: self.dma,
            srambase: self.srambase,
        }
    }
}

impl Handle<init_state::Enabled> {
    /// Disable the DMA controller
    pub fn disable(self, syscon: &mut syscon::Handle) -> Handle<init_state::Disabled> {
        syscon.disable_clock(&self.dma);

        Handle {
            _state: init_state::Disabled,
            dma: self.dma,
            srambase: self.srambase,
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
    config: u32,
    source_end: *const u8,
    dest_end: *mut u8,
    next_desc: *const ChannelDescriptor,
}

impl ChannelDescriptor {
    const fn new() -> Self {
        ChannelDescriptor {
            config: 0,
            source_end: ptr::null(),
            dest_end: ptr::null_mut(),
            next_desc: ptr::null(),
        }
    }
}

// `ChannelDescriptor` contains raw pointers, therefore `Send` is not derived
// automatically. I really see no reason why `ChannelDescriptor` shouldn't be
// `Send` though, and it needs to be `Send`, so one can put it into a
// `cortex_m::interrupt::Mutex`.
unsafe impl Send for ChannelDescriptor {}

/// A DMA channel
pub struct Channel<T, S>
where
    T: ChannelTrait,
{
    ty: T,
    _state: S,
    descriptor: &'static mut ChannelDescriptor,

    // This channel's dedicated registers.
    cfg: RegProxy<T::Cfg>,
    xfercfg: RegProxy<T::Xfercfg>,

    // Shared registers. We restrict our access to the one bit that is dedicated
    // to this channel, so sharing those with other channels should be safe.
    active0: RegProxy<ACTIVE0>,
    enableset0: RegProxy<ENABLESET0>,
    settrig0: RegProxy<SETTRIG0>,
}

impl<T> Channel<T, init_state::Disabled>
where
    T: ChannelTrait,
{
    /// Enable the channel
    pub fn enable<'dma>(self, dma: &'dma Handle) -> Channel<T, init_state::Enabled<&'dma Handle>> {
        Channel {
            ty: self.ty,
            _state: init_state::Enabled(dma),
            descriptor: self.descriptor,

            cfg: self.cfg,
            xfercfg: self.xfercfg,

            active0: self.active0,
            enableset0: self.enableset0,
            settrig0: self.settrig0,
        }
    }
}

impl<'dma, T> Channel<T, init_state::Enabled<&'dma Handle>>
where
    T: ChannelTrait,
{
    /// Starts a DMA transfer
    ///
    /// # Limitations
    ///
    /// The length of `source` must be 1024 or less.
    pub fn start_transfer<D>(self, source: &'static mut [u8], mut dest: D) -> Transfer<'dma, T, D>
    where
        D: Dest,
    {
        compiler_fence(Ordering::SeqCst);

        // We need to substract 1 from the length below. If the source is empty,
        // return early to prevent underflow.
        if source.is_empty() {
            return Transfer {
                channel: self,
                source,
                dest,
            };
        }

        // Configure channel 1 (has request input USART0_TX_DMA)
        // See user manual, section 12.6.16.
        self.cfg.write(|w| {
            w.periphreqen().enabled();
            w.hwtrigen().disabled();
            w.trigburst().single();
            unsafe { w.chpriority().bits(0) }
        });

        // Set channel transfer configuration
        // See user manual, section 12.6.18.
        self.xfercfg.write(|w| {
            w.cfgvalid().valid();
            w.reload().disabled();
            w.swtrig().not_set();
            w.clrtrig().cleared();
            w.setinta().no_effect();
            w.setintb().no_effect();
            w.width().bit_8();
            w.srcinc().width_x_1();
            w.dstinc().no_increment();
            unsafe { w.xfercount().bits(source.len() as u16 - 1) }
        });

        let source_end = unsafe { source.as_ptr().add(source.len() - 1) };

        // Configure channel descriptor
        // See user manual, sections 12.5.2 and 12.5.3.
        self.descriptor.source_end = source_end;
        self.descriptor.dest_end = dest.end_addr();

        // Enable channel 1
        // See user manual, section 12.6.4.
        self.enableset0.write(|w| unsafe { w.ena().bits(T::FLAG) });

        // Trigger transfer
        self.settrig0.write(|w| unsafe { w.trig().bits(T::FLAG) });

        Transfer {
            channel: self,
            source,
            dest,
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
        pub struct Channels {
            $(pub $field: Channel<$name, init_state::Disabled>,)*
        }

        impl Channels {
            fn new(descriptors: &'static mut DescriptorTable) -> Self {
                let mut descriptors = (&mut descriptors.0).into_iter();

                Channels {
                    $(
                        $field: Channel {
                            ty        : $name(()),
                            _state    : init_state::Disabled,
                            descriptor: descriptors.next().unwrap(),

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
pub struct Transfer<'dma, T, D>
where
    T: ChannelTrait,
{
    channel: Channel<T, init_state::Enabled<&'dma Handle>>,
    source: &'static mut [u8],
    dest: D,
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
        (
            Channel<T, init_state::Enabled<&'dma Handle>>,
            &'static mut [u8],
            D,
        ),
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

reg!(ACTIVE0, ACTIVE0, pac::DMA0, active0);
reg!(ENABLESET0, ENABLESET0, pac::DMA0, enableset0);
reg!(SETTRIG0, SETTRIG0, pac::DMA0, settrig0);
