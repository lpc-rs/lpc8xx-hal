//! API for the I2C master mode

use core::{
    convert::{TryFrom, TryInto as _},
    fmt,
    marker::PhantomData,
};

use embedded_hal::blocking::i2c;

use crate::{
    dma::{self, transfer::state::Ready},
    init_state::Enabled,
    pac::{
        dma0::channel::xfercfg::{DSTINC_A, SRCINC_A},
        i2c0::{stat::MSTSTATE_A, MSTCTL, MSTDAT},
    },
    reg_proxy::{Reg, RegProxy},
};

use super::{Error, Instance};

/// API for the I2C master mode
///
/// You can get access to this struct through the [`I2C`] struct.
///
/// This struct has two type parameters that track its state:
/// - `State` tracks whether the I2C instance is enabled.
/// - `ModeState` tracks whether the master mode is enabled.
///
/// # `embedded-hal` traits
/// - [`embedded_hal::blocking::i2c::Read`] for blocking reads
/// - [`embedded_hal::blocking::i2c::Write`] for blocking writes
///
/// [`I2C`]: ../struct.I2C.html
/// [`embedded_hal::blocking::i2c::Read`]: #impl-Read
/// [`embedded_hal::blocking::i2c::Write`]: #impl-Write
pub struct Master<I: Instance, State, ModeState> {
    _state: PhantomData<State>,
    _mode_state: PhantomData<ModeState>,

    mstctl: RegProxy<MstCtl<I>>,
    mstdat: RegProxy<MstDat<I>>,
}

impl<I, State, ModeState> Master<I, State, ModeState>
where
    I: Instance,
{
    pub(super) fn new() -> Self {
        Self {
            _state: PhantomData,
            _mode_state: PhantomData,

            mstctl: RegProxy::new(),
            mstdat: RegProxy::new(),
        }
    }
}

impl<I, C> Master<I, Enabled<PhantomData<C>>, Enabled>
where
    I: Instance,
{
    /// Writes the provided buffer using DMA
    ///
    /// # Panics
    ///
    /// Panics, if the length of `buffer` is 0 or larger than 1024.
    pub fn write_all(
        mut self,
        address: u8,
        buffer: &'static [u8],
        channel: dma::Channel<I::MstChannel, Enabled>,
    ) -> Result<dma::Transfer<Ready, I::MstChannel, &'static [u8], Self>, Error>
    {
        self.start_operation(address, Rw::Write)?;
        self.wait_for_state(State::TxReady)?;
        self.mstctl.modify(|_, w| w.mstdma().enabled());
        Ok(dma::Transfer::new(channel, buffer, self))
    }

    /// Reads until the provided buffer is full, using DMA
    ///
    /// # Panics
    ///
    /// Panics, if the length of `buffer` is 0 or larger than 1024.
    pub fn read_all(
        mut self,
        address: u8,
        buffer: &'static mut [u8],
        channel: dma::Channel<I::MstChannel, Enabled>,
    ) -> Result<
        dma::Transfer<Ready, I::MstChannel, Self, &'static mut [u8]>,
        Error,
    > {
        self.start_operation(address, Rw::Read)?;
        self.mstctl.modify(|_, w| w.mstdma().enabled());
        Ok(dma::Transfer::new(channel, self, buffer))
    }

    /// Wait while the peripheral is busy
    ///
    /// Once this method returns, the peripheral should either be idle or in a
    /// state that requires software interaction.
    fn wait_for_state(&self, expected: State) -> Result<(), Error> {
        // Sound, as we're only reading from the STAT register.
        let i2c = unsafe { &*I::REGISTERS };

        while i2c.stat.read().mstpending().is_in_progress() {
            Error::read::<I>()?;
        }

        let mststate = i2c.stat.read().mststate();
        let actual =
            mststate.variant().try_into().map_err(|()| mststate.bits());
        if Ok(&expected) != actual.as_ref() {
            return Err(Error::UnexpectedState { expected, actual });
        }

        Ok(())
    }

    fn start_operation(&mut self, address: u8, rw: Rw) -> Result<(), Error> {
        Error::check_address(address)?;
        self.wait_for_state(State::Idle)?;

        // Write address
        let address_rw = (address << 1) | rw as u8;
        self.mstdat.write(|w| unsafe {
            // Sound, as all 8-bit values are accepted here.
            w.data().bits(address_rw)
        });

        // Start operation
        self.mstctl.write(|w| w.mststart().start());

        Ok(())
    }

    fn finish_write(&mut self) -> Result<(), Error> {
        self.wait_for_state(State::TxReady)?;

        // Stop operation
        self.mstctl.write(|w| w.mststop().stop());

        Ok(())
    }

    fn finish_read(&mut self) -> Result<(), Error> {
        self.wait_for_state(State::RxReady)?;

        // Stop operation
        self.mstctl.write(|w| w.mststop().stop());

        Ok(())
    }
}

impl<I, C> i2c::Write for Master<I, Enabled<PhantomData<C>>, Enabled>
where
    I: Instance,
{
    type Error = Error;

    /// Write to the I2C bus
    ///
    /// Please refer to the [embedded-hal documentation] for details.
    ///
    /// [embedded-hal documentation]: https://docs.rs/embedded-hal/0.2.1/embedded_hal/blocking/i2c/trait.Write.html#tymethod.write
    fn write(&mut self, address: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.start_operation(address, Rw::Write)?;

        for &b in data {
            self.wait_for_state(State::TxReady)?;

            // Write byte
            self.mstdat.write(|w| unsafe { w.data().bits(b) });

            // Continue transmission
            self.mstctl.write(|w| w.mstcontinue().continue_());
        }

        self.finish_write()?;

        Ok(())
    }
}

impl<I, C> i2c::Read for Master<I, Enabled<PhantomData<C>>, Enabled>
where
    I: Instance,
{
    type Error = Error;

    /// Read from the I2C bus
    ///
    /// Please refer to the [embedded-hal documentation] for details.
    ///
    /// [embedded-hal documentation]: https://docs.rs/embedded-hal/0.2.1/embedded_hal/blocking/i2c/trait.Read.html#tymethod.read
    fn read(
        &mut self,
        address: u8,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.start_operation(address, Rw::Read)?;

        for (i, b) in buffer.iter_mut().enumerate() {
            if i != 0 {
                // Continue transmission
                self.mstctl.write(|w| w.mstcontinue().continue_());
            }

            self.wait_for_state(State::RxReady)?;

            // Read received byte
            *b = self.mstdat.read().data().bits();
        }

        self.finish_read()?;

        Ok(())
    }
}

impl<I, State, ModeState> crate::private::Sealed for Master<I, State, ModeState> where
    I: Instance
{
}

impl<I, C> dma::Dest for Master<I, Enabled<PhantomData<C>>, Enabled>
where
    I: Instance,
{
    type Error = Error;

    fn is_valid(&self) -> bool {
        true
    }

    fn is_full(&self) -> bool {
        false
    }

    fn increment(&self) -> DSTINC_A {
        DSTINC_A::NO_INCREMENT
    }

    fn transfer_count(&self) -> Option<u16> {
        None
    }

    fn end_addr(&mut self) -> *mut u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).mstdat }) as *const _ as *mut u8
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        self.mstctl.modify(|_, w| w.mstdma().disabled());
        self.finish_write()?;
        Ok(())
    }
}

impl<I, C> dma::Source for Master<I, Enabled<PhantomData<C>>, Enabled>
where
    I: Instance,
{
    type Error = Error;

    fn is_valid(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn increment(&self) -> SRCINC_A {
        SRCINC_A::NO_INCREMENT
    }

    fn transfer_count(&self) -> Option<u16> {
        None
    }

    fn end_addr(&self) -> *const u8 {
        // Sound, because we're dereferencing a register address that is always
        // valid on the target hardware.
        (unsafe { &(*I::REGISTERS).mstdat }) as *const _ as *mut u8
    }

    fn finish(&mut self) -> nb::Result<(), Self::Error> {
        self.mstctl.modify(|_, w| w.mstdma().disabled());
        self.finish_read()?;
        Ok(())
    }
}

// Can't derive, because peripheral structs from the PAC don't implement
// `Debug`. See https://github.com/rust-embedded/svd2rust/issues/48.
impl<I, State, ModeState> fmt::Debug for Master<I, State, ModeState>
where
    I: Instance,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Master")
            .field("_state", &self._state)
            .field("_mode_state", &self._mode_state)
            .field("mstctl", &self.mstctl)
            .field("mstdat", &self.mstdat)
            .finish()
    }
}

/// Private helper struct to model the R/W bit
#[repr(u8)]
enum Rw {
    Write = 0,
    Read = 1,
}

/// The state of an I2C instance set to master mode
#[derive(Debug, Eq, PartialEq)]
pub enum State {
    /// The peripheral is currently idle
    ///
    /// A new transaction can be started.
    Idle,

    /// Data has been received and is available to be read
    ///
    /// A read transaction has previously been initiated, and has been
    /// acknowledged by the slave.
    RxReady,

    /// Data can be transmitted
    ///
    /// A write transaction has previously been initiated, and has been
    /// acknowledged by the slave.
    TxReady,

    /// Slave has sent NACK in response to an address
    NackAddress,

    /// Slave has sent NACK in response to data
    NackData,
}

impl TryFrom<Option<MSTSTATE_A>> for State {
    type Error = ();

    fn try_from(state: Option<MSTSTATE_A>) -> Result<Self, Self::Error> {
        match state {
            Some(MSTSTATE_A::IDLE) => Ok(Self::Idle),
            Some(MSTSTATE_A::RECEIVE_READY) => Ok(Self::RxReady),
            Some(MSTSTATE_A::TRANSMIT_READY) => Ok(Self::TxReady),
            Some(MSTSTATE_A::NACK_ADDRESS) => Ok(Self::NackAddress),
            Some(MSTSTATE_A::NACK_DATA) => Ok(Self::NackData),
            None => Err(()),
        }
    }
}

struct MstCtl<I>(PhantomData<I>);

// Sound, as the pointer returned is valid for the duration of the program.
unsafe impl<I> Reg for MstCtl<I>
where
    I: Instance,
{
    type Target = MSTCTL;

    fn get() -> *const Self::Target {
        // Sound, as MSTCTL is exclusively used by `Master`, and only one
        // `RegProxy` instance for it exists.
        unsafe { &(*I::REGISTERS).mstctl as *const _ }
    }
}

// Can't derive, because peripheral structs from the PAC don't implement
// `Debug`. See https://github.com/rust-embedded/svd2rust/issues/48.
impl<I> fmt::Debug for MstCtl<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MstCtl(...)")
    }
}

struct MstDat<I>(PhantomData<I>);

// Sound, as the pointer returned is valid for the duration of the program.
unsafe impl<I> Reg for MstDat<I>
where
    I: Instance,
{
    type Target = MSTDAT;

    fn get() -> *const Self::Target {
        // Sound, as MSTDAT is exclusively used by `Master`, and only one
        // `RegProxy` instance for it exists.
        unsafe { &(*I::REGISTERS).mstdat as *const _ }
    }
}

// Can't derive, because peripheral structs from the PAC don't implement
// `Debug`. See https://github.com/rust-embedded/svd2rust/issues/48.
impl<I> fmt::Debug for MstDat<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MstDat(...)")
    }
}
