use core::marker::PhantomData;

use crate::{init_state, pac, syscon};

/// Handle to the SWM peripheral
///
/// Can be used to enable and disable the switch matrix. It is also required by
/// other parts of the HAL API to synchronize access the the underlying
/// registers, wherever this is required.
///
/// This struct is part of [`swm::Parts`].
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [`swm::Parts`]: struct.Parts.html
/// [module documentation]: index.html
pub struct Handle<State = init_state::Enabled> {
    pub(super) swm: pac::SWM0,
    _state: PhantomData<State>,
}

impl<STATE> Handle<STATE> {
    pub(crate) fn new(swm: pac::SWM0) -> Self {
        Handle {
            swm,
            _state: PhantomData,
        }
    }
}

impl Handle<init_state::Disabled> {
    /// Enable the switch matrix
    ///
    /// This method is only available, if `swm::Handle` is in the [`Disabled`]
    /// state. Code that attempts to call this method when the peripheral is
    /// already enabled will not compile.
    ///
    /// Consumes this instance of `swm::Handle` and returns another instance
    /// that has its `State` type parameter set to [`Enabled`].
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    pub fn enable(
        self,
        syscon: &mut syscon::Handle,
    ) -> Handle<init_state::Enabled> {
        syscon.enable_clock(&self.swm);

        Handle {
            swm: self.swm,
            _state: PhantomData,
        }
    }
}

impl Handle<init_state::Enabled> {
    /// Disable the switch matrix
    ///
    /// The switch matrix retains it's configuration while disabled, but
    /// doesn't allow modifications
    ///
    /// This method is only available, if `swm::Handle` is in the [`Enabled`]
    /// state. Code that attempts to call this method when the peripheral is
    /// already disabled will not compile.
    ///
    /// Consumes this instance of `swm::Handle` and returns another instance
    /// that has its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> Handle<init_state::Disabled> {
        syscon.disable_clock(&self.swm);

        Handle {
            swm: self.swm,
            _state: PhantomData,
        }
    }
}
