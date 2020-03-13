use crate::pins::{self, Pin, PinTrait};

use super::{
    function_kind::{Analog, Input, Output},
    FunctionTrait,
};

/// Internal trait used to assign functions to pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Function::assign`] for the public API that uses this
/// trait.
pub trait AssignFunction<Function, Kind> {
    /// The type of the pin after the function has been assigned
    type Assigned;

    /// Internal method for assigning a function to a pin
    fn assign(self) -> Self::Assigned;
}

/// Internal trait used to unassign functions from pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Function::unassign`] for the public API that uses this
/// trait.
pub trait UnassignFunction<Function, Kind> {
    /// The type of the pin after the function has been unassigned
    type Unassigned;

    /// Internal method for unassigning a function from a pin
    fn unassign(self) -> Self::Unassigned;
}

impl<T, F, O, Is> AssignFunction<F, Input> for Pin<T, pins::state::Swm<O, Is>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Input>,
{
    type Assigned = Pin<T, pins::state::Swm<O, (Is,)>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty: self.ty,
            _state: pins::state::Swm::new(),
        }
    }
}

impl<T, F, Is> AssignFunction<F, Output> for Pin<T, pins::state::Swm<(), Is>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Output>,
{
    type Assigned = Pin<T, pins::state::Swm<((),), Is>>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty: self.ty,
            _state: pins::state::Swm::new(),
        }
    }
}

impl<T, F, O, Is> UnassignFunction<F, Input>
    for Pin<T, pins::state::Swm<O, (Is,)>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Input>,
{
    type Unassigned = Pin<T, pins::state::Swm<O, Is>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty: self.ty,
            _state: pins::state::Swm::new(),
        }
    }
}

impl<T, F, Is> UnassignFunction<F, Output>
    for Pin<T, pins::state::Swm<((),), Is>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Output>,
{
    type Unassigned = Pin<T, pins::state::Swm<(), Is>>;

    fn unassign(self) -> Self::Unassigned {
        Pin {
            ty: self.ty,
            _state: pins::state::Swm::new(),
        }
    }
}

impl<T, F> AssignFunction<F, Analog> for Pin<T, pins::state::Swm<(), ()>>
where
    T: PinTrait,
    F: FunctionTrait<T, Kind = Analog>,
{
    type Assigned = Pin<T, pins::state::Analog>;

    fn assign(self) -> Self::Assigned {
        Pin {
            ty: self.ty,
            _state: pins::state::Analog,
        }
    }
}
