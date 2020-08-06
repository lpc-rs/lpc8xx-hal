//! Type state for the USART peripheral

use core::marker::PhantomData;

/// Indicates that the USART peripheral is enabled
///
/// Used as a type argument on `USART`.
pub struct Enabled<W: Word, Mode>(PhantomData<W>, PhantomData<Mode>);

/// Implemented for words that are supported by the USART peripheral
pub trait Word: Into<u16> {
    /// Converts a `u16` into `Self`
    ///
    /// We can't require `From<u16>` as a trait bound, as that is not going to
    /// be implemented for `u8`, but we know that this conversion will never
    /// fail, as long as there's no bug that causes a mismatch between
    /// peripheral type state and configuration.
    ///
    /// Intended for internal use only.
    fn from_u16(w: u16) -> Self;
}

impl Word for u8 {
    fn from_u16(w: u16) -> Self {
        w as u8
    }
}

impl Word for u16 {
    fn from_u16(w: u16) -> Self {
        w
    }
}

/// Indicates that a USART instance is operation in asynchronous mode
///
/// Used as a type parameter by `USART`.
pub struct AsyncMode;

/// Indicates that transmitter is not throttled
///
/// Used as a type parameter by `usart::Tx`.
pub struct NoThrottle;

/// Indicates that the transmitter is throttled via the CTS signal
///
/// Used as a type parameter by `usart::Tx`.
pub struct CtsThrottle<F>(pub(super) F);
