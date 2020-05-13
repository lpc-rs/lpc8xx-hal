use core::marker::PhantomData;

/// A struct containing the clock configuration for a peripheral
pub struct SpiClock<Clock> {
    pub(crate) divval: u16,
    // The fields in the DLY register are ignored, since SSEL & EOF aren't used
    pub(crate) _clock: PhantomData<Clock>,
}
