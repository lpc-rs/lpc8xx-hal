/// Implemented by types that identify pin interrupts
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
pub trait Trait {
    /// The index of this interrupt
    ///
    /// Used to select the PINTSEL register for this interupt.
    const INDEX: usize;

    /// The interrupt's bit mask
    ///
    /// Used in various registers.
    const MASK: u8;
}
