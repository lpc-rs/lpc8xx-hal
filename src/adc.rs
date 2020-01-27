//! TODO

use embedded_hal::adc::{Channel, OneShot};

use crate::{
    init_state, pac, swm,
    syscon::{self, clocksource::AdcClock},
};

/// Interface to the ADC peripheral
///
/// Controls the ADC.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct ADC<State = init_state::Enabled> {
    adc: pac::ADC0,
    _state: State,
}

impl ADC<init_state::Disabled> {
    pub(crate) fn new(adc: pac::ADC0) -> Self {
        Self {
            adc,
            _state: init_state::Disabled,
        }
    }
    /// Enable the ADC
    ///
    /// This method is only available, if `ADC` is in the [`Disabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// enabled will not compile.
    ///
    /// Consumes this instance of `ADC` and returns another instance that has
    /// its `State` type parameter set to [`Enabled`].
    ///
    /// # Limitations
    ///
    /// This assumes that the clock is running at 500 kHz for the calibration
    ///
    /// # Examples
    ///
    /// Please refer to the [module documentation] for a full example.
    ///
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [module documentation]: index.html
    pub fn enable(self, clock: &AdcClock, syscon: &mut syscon::Handle) -> ADC {
        syscon.enable_clock(&self.adc);
        syscon.power_up(&self.adc);

        // Start calibration
        // The clock needs to be at 500 kHz for this task
        self.adc.ctrl.write(|w| {
            unsafe { w.clkdiv().bits(clock.caldiv) };
            w.calmode().set_bit()
        });

        // Wait until the calibration is done
        while self.adc.ctrl.read().calmode().bit_is_set() {}

        self.adc
            .ctrl
            .write(|w| unsafe { w.clkdiv().bits(clock.div) });

        ADC {
            adc: self.adc,
            _state: init_state::Enabled(()),
        }
    }
}

impl ADC<init_state::Enabled> {
    /// Disable the ADC
    ///
    /// This method is only available, if `ADC` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `ADC` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> ADC<init_state::Disabled> {
        syscon.disable_clock(&self.adc);

        ADC {
            adc: self.adc,
            _state: init_state::Disabled,
        }
    }
}

impl<State> ADC<State> {
    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> pac::ADC0 {
        self.adc
    }
}

impl<PIN> OneShot<ADC, u16, PIN> for ADC
where
    PIN: Channel<ADC, ID = u8>,
{
    type Error = ();

    // TODO Nonblocking
    fn read(&mut self, _: &mut PIN) -> nb::Result<u16, Self::Error> {
        // Start reading ofthe channel
        // Follows the description in the um
        self.adc.seq_ctrla.write(|w| {
            unsafe { w.channels().bits(1 << PIN::channel()) };
            w.start().set_bit();
            w.trigpol().set_bit();
            w.seq_ena().enabled();
            w.mode().end_of_conversion()
        });
        let mut read = self.adc.seq_gdata.read();
        // Wait until the conversion is done
        while read.datavalid().bit_is_clear() {
            read = self.adc.seq_gdata.read();
        }
        // Return the results as 16 bit values
        Ok(read.result().bits() << 4)
    }
}

macro_rules! adc_channel {
    ($pin:ident, $num:expr) => {
        impl<PIN> Channel<ADC>
            for swm::Function<swm::$pin, swm::state::Assigned<PIN>>
        {
            type ID = u8;

            fn channel() -> Self::ID {
                $num
            }
        }
    };
}

adc_channel!(ADC_0, 0);
adc_channel!(ADC_1, 1);
adc_channel!(ADC_2, 2);
adc_channel!(ADC_3, 3);
adc_channel!(ADC_4, 4);
adc_channel!(ADC_5, 5);
adc_channel!(ADC_6, 6);
adc_channel!(ADC_7, 7);
adc_channel!(ADC_8, 8);
adc_channel!(ADC_9, 9);
adc_channel!(ADC_10, 10);
adc_channel!(ADC_11, 11);
