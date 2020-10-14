//! Type state for the USART peripheral

use core::marker::PhantomData;

/// Indicates that the USART peripheral is enabled
///
/// Used as a type parameter on [`USART`], [`usart::Rx`], and [`usart::Tx`].
///
/// [`USART`]: ../struct.USART.html
/// [`usart::Rx`]: ../struct.Rx.html
/// [`usart::Tx`]: ../struct.Tx.html
#[derive(Debug)]
pub struct Enabled<W: Word, Mode>(PhantomData<W>, PhantomData<Mode>);

/// Implemented for types that represent a supported word size
///
/// The USART peripheral supports word sizes of 7 bits, 8 bits (both represented
/// by `u8`), and 9 bits (represented by `u16`).
pub trait Word: Into<u16> {
    /// Converts a `u16` to `Self`
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

/// Indicates that a USART instance is operating in asynchronous mode
///
/// Used as a type parameter on [`Enabled`].
///
/// [`Enabled`]: struct.Enabled.html
#[derive(Debug)]
pub struct AsyncMode;

/// Indicates that a USART instance is operating in synchronous mode
///
/// Used as a type parameter on [`Enabled`].
///
/// [`Enabled`]: struct.Enabled.html
#[derive(Debug)]
pub struct SyncMode;

/// Indicates that transmitter is not throttled
///
/// Used as a type parameter on [`usart::Tx`].
///
/// [`usart::Tx`]: ../struct.Tx.html
#[derive(Debug)]
pub struct NoThrottle;

/// Indicates that the transmitter is throttled via the CTS signal
///
/// Used as a type parameter on [`usart::Tx`].
///
/// [`usart::Tx`]: ../struct.Tx.html
#[derive(Debug)]
pub struct CtsThrottle<F>(pub(super) F);
