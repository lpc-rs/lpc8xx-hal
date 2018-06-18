<a name="v0.2.0"></a>
## v0.2.0 (2018-06-15)


- Add support for additional low-power modes ([#106](https://github.com/braun-robotics/rust-lpc82x-hal/pull/106))
- Add basic I2C API ([#97](https://github.com/braun-robotics/rust-lpc82x-hal/pull/97))
- Implement `fmt::Write` for `USART` ([dbf1ec6a](dbf1ec6a))
- Drastically simplify compile-time state management ([#95](https://github.com/braun-robotics/rust-lpc82x-hal/pull/95))
- Drastically simplify switch matrix API ([#86](https://github.com/braun-robotics/rust-lpc82x-hal/pull/86), [#87](https://github.com/braun-robotics/rust-lpc82x-hal/pull/87), [#90](https://github.com/braun-robotics/rust-lpc82x-hal/pull/90), [#91](https://github.com/braun-robotics/rust-lpc82x-hal/pull/91), [#92](https://github.com/braun-robotics/rust-lpc82x-hal/pull/92), [#93](https://github.com/braun-robotics/rust-lpc82x-hal/pull/93))
- Expect USART RX/TX to be assigned on enable ([010d8982](010d8982))
- Remove macro re-exports ([1faea72f](1faea72f))
- Rename pin state transition methods ([593cf199](593cf199))
- Clean up peripheral state management ([#79](https://github.com/braun-robotics/rust-lpc82x-hal/pull/79))
- Re-export `lpc82x` as `lpc82x_hal::raw` ([468ddba0](468ddba0))
- Many additional cleanups and updates


<a name="v0.1.0"></a>
### v0.1.0 (2018-03-12)

Initial release
