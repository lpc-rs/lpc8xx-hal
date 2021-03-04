use super::Channels;

use crate::{pac, syscon};

/// Represents the MRT instance
pub struct MRT {
    mrt: pac::MRT0,
}

impl MRT {
    /// Assumes peripheral is in reset state
    ///
    /// This means:
    /// - Each channel is in repeat mode
    /// - All channel interrupts are disabled
    pub(crate) fn new(mrt: pac::MRT0) -> Self {
        Self { mrt }
    }

    /// Enables the MRT and splits it into it's four channels
    pub fn split(self, syscon: &mut syscon::Handle) -> Channels {
        syscon.enable_clock(&self.mrt);

        Channels::new()
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
    pub fn free(self) -> pac::MRT0 {
        self.mrt
    }
}
