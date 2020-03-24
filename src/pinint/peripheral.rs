use core::marker::PhantomData;

use crate::{
    init_state::{Disabled, Enabled},
    pac, syscon,
};

use super::gen::Interrupts;

/// Entry point to the PININT API
pub struct PININT<State> {
    /// Provides access to the pin interrupts
    pub interrupts: Interrupts<State>,

    pinint: pac::PINT,
    _state: PhantomData<State>,
}

impl PININT<Disabled> {
    pub(crate) fn new(pinint: pac::PINT) -> Self {
        Self {
            interrupts: Interrupts::new(),
            pinint,
            _state: PhantomData,
        }
    }

    /// Enable the PININT peripheral
    pub fn enable(self, syscon: &mut syscon::Handle) -> PININT<Enabled> {
        syscon.enable_clock(&self.pinint);

        PININT {
            interrupts: Interrupts::new(),
            pinint: self.pinint,
            _state: PhantomData,
        }
    }
}

impl<State> PININT<State> {
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
    pub fn free(self) -> pac::PINT {
        self.pinint
    }
}
