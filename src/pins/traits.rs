/// Implemented by types that identify pins
///
/// This trait is an internal implementation detail and should neither be
/// implemented nor used outside of LPC8xx HAL. Any changes to this trait won't
/// be considered breaking changes.
///
/// Please refer to [`Pin`] for the public API used to control pins.
///
/// [`Pin`]: struct.Pin.html
pub trait Trait {
    /// Get the number that indentifies the port
    ///
    /// This is `0` for PIO0 pins (e.g. [`PIO0_0`]) and `1` for PIO1 pins (e.g.
    /// [`PIO1_0`]).
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO1_0`]: struct.PIO1_0.html
    fn port(&self) -> usize; // TODO make u8

    /// A number that identifies the pin
    ///
    /// This is `0` for [`PIO0_0`], `1` for [`PIO0_1`] and so forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    fn id(&self) -> u8;

    /// The pin's bit mask
    ///
    /// This is `0x00000001` for [`PIO0_0`], `0x00000002` for [`PIO0_1`],
    /// `0x00000004` for [`PIO0_2`], and so forth.
    ///
    /// [`PIO0_0`]: struct.PIO0_0.html
    /// [`PIO0_1`]: struct.PIO0_1.html
    /// [`PIO0_2`]: struct.PIO0_2.html
    fn mask(&self) -> u32;
}
