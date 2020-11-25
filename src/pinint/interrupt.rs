use core::marker::PhantomData;

use super::traits::Trait;

use crate::{init_state::Enabled, pac, pins, syscon};

/// API for controlling pin interrupts
pub struct Interrupt<I, P, State> {
    interrupt: PhantomData<I>,
    _pin: PhantomData<P>,
    state: PhantomData<State>,
}

impl<I, P, State> Interrupt<I, P, State> {
    pub(super) fn new() -> Self {
        Self {
            interrupt: PhantomData,
            _pin: PhantomData,
            state: PhantomData,
        }
    }
}

impl<I, OldPin, State> Interrupt<I, OldPin, State>
where
    I: Trait,
{
    /// Select a pin as the source of this interrupt
    ///
    /// Please be aware that this method allows you to do things that might or
    /// might not be valid things to do:
    ///
    /// - You can select any pin, regardless of its current state.
    /// - You can select the same pin for multiple interrupts.
    ///
    /// The documentation isn't totally clear about whether these are allowed,
    /// and no research has been done to verify it one way or the other. Please
    /// be careful, and make sure to [open an issue], if you discover that this
    /// API allows you to do something that is not correct.
    ///
    /// Please also be aware that the interrupt handler for various pin
    /// interrupts is reused for other purposes.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn select<P>(
        self,
        interrupt_pin: &P,
        _: &mut syscon::Handle,
    ) -> Interrupt<I, P, State>
    where
        P: pins::Trait,
    {
        // Sound, as this `Interrupt` instance is the only one accessing this
        // register, and the mutable reference to the SYSCON handle guarantees
        // that safe concurrent PAC-level access to the register is not
        // possible.
        let syscon = unsafe { &*pac::SYSCON::ptr() };

        syscon.pintsel[I::INDEX].write(|w|
            // Sound, as any value with `0 <= value <= 63` is valid to write to
            // the register.
            unsafe { w.intpin().bits(32 * interrupt_pin.port() as u8 + interrupt_pin.id())});

        Interrupt {
            interrupt: self.interrupt,
            _pin: PhantomData,
            state: self.state,
        }
    }
}

impl<I, P> Interrupt<I, P, Enabled>
where
    I: Trait,
    P: pins::Trait,
{
    /// Returns whether a rising edge has been detected and clears the flag
    ///
    /// This method will work regardless of whether rising edge interrupts have
    /// been enabled or not.
    ///
    /// You must call this handle while handling a rising edge interrupt.
    /// Otherwise, the interrupt will be fired again immediately, after the
    /// interrupt handler exits.
    pub fn clear_rising_edge_flag(&mut self) -> bool {
        // This is sound, as we're only doing an atomic read and write, both to // a single bit that no other `Interrupt` instance is writing to.
        let pint = unsafe { &*pac::PINT::ptr() };

        let is_set = pint.rise.read().rdet().bits() & I::MASK != 0;

        // Clear flag
        pint.rise.write(|w|
            // Sound, as long as `Trait` is only implemented for valid
            // interrupts.
            unsafe { w.rdet().bits(I::MASK) });

        is_set
    }

    /// Fire interrupt on rising edge
    pub fn enable_rising_edge(&mut self) {
        // This is sound, as we're only doing an atomic write to a single bit
        // that no other `Interrupt` instance is writing to.
        let pint = unsafe { &*pac::PINT::ptr() };

        pint.sienr.write(|w|
            // Sound, as long as `Trait` is only implemented for valid
            // interrupts.
            unsafe { w.setenrl().bits(I::MASK) });
    }

    /// Don't fire interrupt on rising edge
    pub fn disable_rising_edge(&mut self) {
        // This is sound, as we're only doing an atomic write to a single bit
        // that no other `Interrupt` instance is writing to.
        let pint = unsafe { &*pac::PINT::ptr() };

        pint.cienr.write(|w|
            // Sound, as long as `Trait` is only implemented for valid
            // interrupts.
            unsafe { w.cenrl().bits(I::MASK) });
    }

    /// Returns whether a falling edge has been detected and clears the flag
    ///
    /// This method will work regardless of whether falling edge interrupts have
    /// been enabled or not.
    ///
    /// You must call this handle while handling a falling edge interrupt.
    /// Otherwise, the interrupt will be fired again immediately, after the
    /// interrupt handler exits.
    pub fn clear_falling_edge_flag(&mut self) -> bool {
        // This is sound, as we're only doing an atomic read and write, both to // a single bit that no other `Interrupt` instance is writing to.
        let pint = unsafe { &*pac::PINT::ptr() };

        let is_set = pint.fall.read().fdet().bits() & I::MASK != 0;

        // Clear flag
        pint.fall.write(|w|
            // Sound, as long as `Trait` is only implemented for valid
            // interrupts.
            unsafe { w.fdet().bits(I::MASK) });

        is_set
    }

    /// Fire interrupt on falling edge
    pub fn enable_falling_edge(&mut self) {
        // This is sound, as we're only doing an atomic write to a single bit
        // that no other `Interrupt` instance is writing to.
        let pint = unsafe { &*pac::PINT::ptr() };

        pint.sienf.write(|w|
            // Sound, as long as `Trait` is only implemented for valid
            // interrupts.
            unsafe { w.setenaf().bits(I::MASK) });
    }

    /// Don't fire interrupt on falling edge
    pub fn disable_falling_edge(&mut self) {
        // This is sound, as we're only doing an atomic write to a single bit
        // that no other `Interrupt` instance is writing to.
        let pint = unsafe { &*pac::PINT::ptr() };

        pint.cienf.write(|w|
            // Sound, as long as `Trait` is only implemented for valid
            // interrupts.
            unsafe { w.cenaf().bits(I::MASK) });
    }
}
