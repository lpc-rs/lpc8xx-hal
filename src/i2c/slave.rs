//! API for the I2C slave mode

use core::marker::PhantomData;

use crate::{
    init_state,
    pac::i2c0::{SLVCTL, SLVDAT},
    reg_proxy::{Reg, RegProxy},
};

use super::{Error, Instance};

/// API for I2C slave mode
///
/// You can get access to this struct through the [`I2C`] struct.
///
/// This struct has two type parameters that track its state:
/// - `State` tracks whether the I2C instance is enabled.
/// - `ModeState` tracks whether the master mode is enabled.
///
/// [`I2C`]: ../struct.I2C.html
pub struct Slave<I: Instance, State, ModeState> {
    _state: PhantomData<State>,
    _mode_state: PhantomData<ModeState>,

    slvctl: RegProxy<SlvCtl<I>>,
    slvdat: RegProxy<SlvDat<I>>,
}

impl<I, State, ModeState> Slave<I, State, ModeState>
where
    I: Instance,
{
    pub(super) fn new() -> Self {
        Self {
            _state: PhantomData,
            _mode_state: PhantomData,

            slvctl: RegProxy::new(),
            slvdat: RegProxy::new(),
        }
    }
}

impl<I, C> Slave<I, init_state::Enabled<PhantomData<C>>, init_state::Enabled>
where
    I: Instance,
{
    /// Wait until software intervention is required
    ///
    /// The returned enum indicates the current state. Each variant provides an
    /// API to react to that state.
    pub fn wait(&mut self) -> nb::Result<State<I>, Error> {
        // Sound, as we're only reading from the STAT register.
        let i2c = unsafe { &*I::REGISTERS };

        Error::read::<I>()?;

        if i2c.stat.read().slvpending().is_in_progress() {
            return Err(nb::Error::WouldBlock);
        }

        let slave_state = i2c.stat.read().slvstate();

        if slave_state.is_slave_address() {
            return Ok(State::AddressMatched(AddressMatched {
                slvctl: &self.slvctl,
                slvdat: &self.slvdat,
            }));
        }
        if slave_state.is_slave_receive() {
            return Ok(State::RxReady(RxReady {
                slvctl: &self.slvctl,
                slvdat: &self.slvdat,
            }));
        }
        if slave_state.is_slave_transmit() {
            return Ok(State::TxReady(TxReady {
                slvctl: &self.slvctl,
                slvdat: &self.slvdat,
            }));
        }

        Err(nb::Error::Other(Error::UnknownSlaveState(
            slave_state.bits(),
        )))
    }
}

/// The current state of the slave
///
/// Each variant provides an API to react to that state. Call [`I2C::wait`] to
/// get an instance of this struct.
///
/// [`I2c::wait`]: ../struct.I2C.html#method.wait
pub enum State<'r, I: Instance> {
    /// Address sent by master has been matched
    AddressMatched(AddressMatched<'r, I>),

    /// Data has been received from master
    RxReady(RxReady<'r, I>),

    /// Ready to transmit data to master
    TxReady(TxReady<'r, I>),
}

/// API for handling the "address matched" state
///
/// You can gain access to this API through [`State`].
///
/// [`State`]: enum.State.html
pub struct AddressMatched<'r, I: Instance> {
    slvctl: &'r RegProxy<SlvCtl<I>>,
    slvdat: &'r RegProxy<SlvDat<I>>,
}

impl<'r, I> AddressMatched<'r, I>
where
    I: Instance,
{
    /// Return the received address
    pub fn address(&self) -> Result<u8, Error> {
        Error::read::<I>()?;

        let address = self.slvdat.read().data().bits() >> 1;
        Ok(address)
    }

    /// Acknowledge the matched address
    pub fn ack(self) -> Result<(), Error> {
        Error::read::<I>()?;

        self.slvctl.write(|w| w.slvcontinue().continue_());

        Ok(())
    }

    /// Reject the matched address
    pub fn nack(self) -> Result<(), Error> {
        Error::read::<I>()?;

        self.slvctl.write(|w| w.slvnack().nack());

        Ok(())
    }
}

/// API for handling the "data received" state
///
/// You can gain access to this API through [`State`].
///
/// [`State`]: enum.State.html
pub struct RxReady<'r, I: Instance> {
    slvctl: &'r RegProxy<SlvCtl<I>>,
    slvdat: &'r RegProxy<SlvDat<I>>,
}

impl<'r, I> RxReady<'r, I>
where
    I: Instance,
{
    /// Read the available data
    ///
    /// If you call this method multiple times, the same data will be returned
    /// each time. To receive the next byte, acknowledge the current one using
    /// [`ack`], then call [`I2C::wait`] again.
    ///
    /// [`ack`]: #method.ack
    /// [`I2C::wait`]: ../struct.I2C.html#method.wait
    pub fn read(&self) -> Result<u8, Error> {
        Error::read::<I>()?;

        let data = self.slvdat.read().data().bits();
        Ok(data)
    }

    /// Acknowledge the received data
    pub fn ack(self) -> Result<(), Error> {
        Error::read::<I>()?;

        self.slvctl.write(|w| w.slvcontinue().continue_());

        Ok(())
    }

    /// Reject the received data
    pub fn nack(self) -> Result<(), Error> {
        Error::read::<I>()?;

        self.slvctl.write(|w| w.slvnack().nack());

        Ok(())
    }
}

/// API for handling the "ready to transmit" state
///
/// You can gain access to this API through [`State`].
///
/// [`State`]: enum.State.html
pub struct TxReady<'r, I: Instance> {
    slvctl: &'r RegProxy<SlvCtl<I>>,
    slvdat: &'r RegProxy<SlvDat<I>>,
}

impl<'r, I> TxReady<'r, I>
where
    I: Instance,
{
    /// Transmit data
    pub fn transmit(self, data: u8) -> Result<(), Error> {
        Error::read::<I>()?;

        // Sound, as all possible values of `u8` are accepted by the DATA field.
        unsafe {
            self.slvdat.write(|w| w.data().bits(data));
        }

        self.slvctl.write(|w| w.slvcontinue().continue_());

        Ok(())
    }
}

struct SlvCtl<I>(PhantomData<I>);

// Sound, as the pointer returned is valid for the duration of the program.
unsafe impl<I> Reg for SlvCtl<I>
where
    I: Instance,
{
    type Target = SLVCTL;

    fn get() -> *const Self::Target {
        // Sound, as SLVCTL is exclusively used by `Slave`, and only one
        // `RegProxy` instance for it exists.
        unsafe { &(*I::REGISTERS).slvctl as *const _ }
    }
}

struct SlvDat<I>(PhantomData<I>);

// Sound, as the pointer returned is valid for the duration of the program.
unsafe impl<I> Reg for SlvDat<I>
where
    I: Instance,
{
    type Target = SLVDAT;

    fn get() -> *const Self::Target {
        // Sound, as SLVDAT is exclusively used by `Slave`, and only one
        // `RegProxy` instance for it exists.
        unsafe { &(*I::REGISTERS).slvdat as *const _ }
    }
}
