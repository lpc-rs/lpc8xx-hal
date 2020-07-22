use crate::{init_state, pac, syscon};

use super::{Channels, DescriptorTable};

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
    pub fn enable(
        self,
        syscon: &mut syscon::Handle,
    ) -> Handle<init_state::Enabled> {
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
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> Handle<init_state::Disabled> {
        syscon.disable_clock(&self.dma);

        Handle {
            _state: init_state::Disabled,
            dma: self.dma,
            srambase: self.srambase,
        }
    }
}
