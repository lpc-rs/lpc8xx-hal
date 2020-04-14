use core::marker::PhantomData;

use crate::{init_state, pac};

use super::{
    fixed_functions::FixedFunctions, handle::Handle,
    movable_functions::MovableFunctions,
};

/// Entry point to the switch matrix (SWM) API
///
/// The SWM API is split into multiple parts, which are all available through
/// [`swm::Parts`]. You can use [`SWM::split`] to gain access to [`swm::Parts`].
///
/// You can also use this struct to gain access to the raw peripheral using
/// [`SWM::free`]. This is the main reason this struct exists, as it's no longer
/// possible to do this after the API has been split.
///
/// Use [`Peripherals`] to gain access to an instance of this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`swm::Parts`]: struct.Parts.html
/// [`SWM::split`]: #method.split
/// [`SWM::free`]: #method.free
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct SWM<State = init_state::Enabled> {
    swm: pac::SWM0,
    state: PhantomData<State>,
}

impl<STATE> SWM<STATE> {
    pub(crate) fn new(swm: pac::SWM0) -> Self {
        SWM {
            swm,
            state: PhantomData,
        }
    }

    /// Splits the SWM API into its component parts
    ///
    /// This is the regular way to access the SWM API. It exists as an explicit
    /// step, as it's no longer possible to gain access to the raw peripheral
    /// using [`SWM::free`] after you've called this method.
    ///
    /// [`SWM::free`]: #method.free
    pub fn split(self) -> Parts<STATE> {
        Parts {
            handle: Handle::new(self.swm),
            movable_functions: MovableFunctions::new(),
            fixed_functions: FixedFunctions::new(),
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
    pub fn free(self) -> pac::SWM0 {
        self.swm
    }
}

/// The main API for the switch matrix (SWM)
///
/// Provides access to all types that make up the SWM API. You gain access to
/// this struct by calling [`SWM::split`].
///
/// Please refer to the [module documentation] for more information.
///
/// [`SWM::split`]: struct.SWM.html#method.split
/// [module documentation]: index.html
pub struct Parts<STATE> {
    /// Handle to the switch matrix
    pub handle: Handle<STATE>,

    /// Movable functions
    pub movable_functions: MovableFunctions,

    /// Fixed functions
    pub fixed_functions: FixedFunctions,
}
