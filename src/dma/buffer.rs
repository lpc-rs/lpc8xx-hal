use crate::pac::dma0::channel::xfercfg::SRCINC_A;

use super::Source;

impl crate::private::Sealed for &'static [u8] {}

impl Source for &'static [u8] {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn increment(&self) -> SRCINC_A {
        SRCINC_A::WIDTH_X_1
    }

    fn transfer_count(&self) -> Option<u16> {
        // The cast should be fine, as DMA buffers are restricted to a length of
        // 1024.
        Some(self.len() as u16 - 1)
    }

    fn end_addr(&self) -> *const u8 {
        // Sound, as we stay within the bounds of the slice.
        unsafe { self.as_ptr().add(self.len() - 1) }
    }
}
