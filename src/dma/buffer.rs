use crate::{
    pac::dma0::channel::xfercfg::{DSTINC_A, SRCINC_A},
    void::Void,
};

use super::{Dest, Source};

impl crate::private::Sealed for &'static [u8] {}

impl Source for &'static [u8] {
    type Error = Void;

    fn is_valid(&self) -> bool {
        self.len() <= 1024
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn increment(&self) -> SRCINC_A {
        SRCINC_A::WIDTH_X_1
    }

    fn transfer_count(&self) -> Option<u16> {
        if self.is_empty() {
            None
        } else {
            // The cast should be fine, as DMA buffers are restricted to a
            // length of 1024.
            Some(self.len() as u16 - 1)
        }
    }

    fn end_addr(&self) -> *const u8 {
        // Sound, as we stay within the bounds of the slice.
        unsafe { self.as_ptr().add(self.len() - 1) }
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

impl crate::private::Sealed for &'static mut [u8] {}

impl Dest for &'static mut [u8] {
    /// The error that can occur while waiting for the destination to be idle
    type Error = Void;

    fn is_valid(&self) -> bool {
        self.len() <= 1024
    }

    fn is_full(&self) -> bool {
        self.len() == 0
    }

    fn increment(&self) -> DSTINC_A {
        DSTINC_A::WIDTH_X_1
    }

    fn transfer_count(&self) -> Option<u16> {
        if self.is_full() {
            None
        } else {
            // The cast should be fine, as DMA buffers are restricted to a
            // length of 1024.
            Some(self.len() as u16 - 1)
        }
    }

    fn end_addr(&mut self) -> *mut u8 {
        // Sound, as we stay within the bounds of the slice.
        unsafe { self.as_mut_ptr().add(self.len() - 1) }
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

pub(crate) struct Buffer {
    ptr: *mut u8,
    len: usize,
}

impl Buffer {
    /// Create a `Buffer` from a static slice
    ///
    /// # Unsafety
    ///
    /// The caller must make sure that the create `Buffer` instance is not used
    /// in a way that would interfere with the nature or usage of the slice. For
    /// example:
    ///
    /// - If the `Buffer` instance is used as a DMA destination, the caller must
    ///   prevent race conditions by making sure no one else writes to the
    ///   slice.
    /// - If the `Buffer` instance is used as a DMA destination, it is the
    ///   caller's responsibility to only pass a reference to a mutable slice,
    ///   even though this method accepts references to immutable slices.
    pub(crate) unsafe fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }
}

impl crate::private::Sealed for Buffer {}

impl Source for Buffer {
    type Error = Void;

    fn is_valid(&self) -> bool {
        self.len <= 1024
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn increment(&self) -> SRCINC_A {
        SRCINC_A::WIDTH_X_1
    }

    fn transfer_count(&self) -> Option<u16> {
        if self.is_empty() {
            None
        } else {
            // The cast should be fine, as DMA buffers are restricted to a
            // length of 1024.
            Some(self.len as u16 - 1)
        }
    }

    fn end_addr(&self) -> *const u8 {
        // Sound, as we stay within the bounds of the slice.
        unsafe { self.ptr.add(self.len - 1) }
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

impl Dest for Buffer {
    /// The error that can occur while waiting for the destination to be idle
    type Error = Void;

    fn is_valid(&self) -> bool {
        self.len <= 1024
    }

    fn is_full(&self) -> bool {
        self.len == 0
    }

    fn increment(&self) -> DSTINC_A {
        DSTINC_A::WIDTH_X_1
    }

    fn transfer_count(&self) -> Option<u16> {
        if self.is_full() {
            None
        } else {
            // The cast should be fine, as DMA buffers are restricted to a
            // length of 1024.
            Some(self.len as u16 - 1)
        }
    }

    fn end_addr(&mut self) -> *mut u8 {
        // Sound, as we stay within the bounds of the slice.
        unsafe { self.ptr.add(self.len - 1) }
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}
