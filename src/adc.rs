//! Contains ADC-related type definitions
//!
//! Please note that there is no ADC API at this time, and that this module just
//! contains some code related to switching pins into ADC mode.
//!
//! If you need an ADC API, [please let us know](https://github.com/braun-robotics/rust-lpc82x-hal/issues/51).


use swm;


/// Marker trait for fixed functions representing ADC channels
///
/// This is an internal trait. Any changes made to it won't be considered
/// breaking changes.
pub trait AdcChannel {}

impl AdcChannel for swm::ADC_0  {}
impl AdcChannel for swm::ADC_1  {}
impl AdcChannel for swm::ADC_2  {}
impl AdcChannel for swm::ADC_3  {}
impl AdcChannel for swm::ADC_4  {}
impl AdcChannel for swm::ADC_5  {}
impl AdcChannel for swm::ADC_6  {}
impl AdcChannel for swm::ADC_7  {}
impl AdcChannel for swm::ADC_8  {}
impl AdcChannel for swm::ADC_9  {}
impl AdcChannel for swm::ADC_10 {}
impl AdcChannel for swm::ADC_11 {}
