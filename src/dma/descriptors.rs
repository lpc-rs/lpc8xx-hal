use core::ptr;

/// The channel descriptor table
///
/// Contains a descriptor for each DMA channel.
#[repr(C, align(512))]
pub struct DescriptorTable(pub(super) [ChannelDescriptor; 18]);

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

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub(super) struct ChannelDescriptor {
    config: u32,
    pub(super) source_end: *const u8,
    pub(super) dest_end: *mut u8,
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
