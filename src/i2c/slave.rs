//! API for the I2C slave mode

use core::marker::PhantomData;

/// API for I2C slave mode
///
/// This is currently not implemented.
pub struct Slave<I, State, ModeState> {
    _instance: PhantomData<I>,
    _state: PhantomData<State>,
    _mode_state: PhantomData<ModeState>,
}

impl<I, State, ModeState> Slave<I, State, ModeState> {
    pub(super) fn new() -> Self {
        Self {
            _instance: PhantomData,
            _state: PhantomData,
            _mode_state: PhantomData,
        }
    }
}
