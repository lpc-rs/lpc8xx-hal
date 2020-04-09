//! APIs for the switch matrix (SWM)
//!
//! The entry point to this API is [`SWM`]. Please refer to [`SWM`]'s
//! documentation for additional information.
//!
//! The switch matrix is described in the user manual, chapter 7.

pub mod state;

mod assignment;
mod fixed_functions;
mod function_kind;
mod functions;
mod handle;
mod movable_functions;
mod peripheral;

pub use self::{
    fixed_functions::*,
    function_kind::{Analog, FunctionKind, Input, Output},
    functions::{Function, FunctionTrait},
    handle::Handle,
    movable_functions::*,
    peripheral::{Parts, SWM},
};
