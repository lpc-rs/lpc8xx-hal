<a name="v0.4.0"></a>
## v0.4.0 (2019-03-14)

- Support stable Rust ([#145](https://github.com/lpc-rs/lpc8xx-hal/pull/145))
- Update documentation ([#148](https://github.com/lpc-rs/lpc8xx-hal/pull/148))

<a name="v0.3.0"></a>
## v0.3.0 (2019-03-12)

- Flush output after formatted UART write ([54ad833](https://github.com/lpc-rs/lpc8xx-hal/commit/54ad833ee80d1cd0307b432d8c5a7fec7160ca7d)).
- Make peripheral names consistent with the ones used in PAC and device documentation, i.e. `GPIO` instead of `gpio` ([4b36798](https://github.com/lpc-rs/lpc8xx-hal/commit/4b367988011e853d8c5c90449c9c1269f22009d1)).
- Add core peripherals to `Peripherals` ([42dddb1](https://github.com/lpc-rs/lpc8xx-hal/commit/42dddb148f8129261aac4a2a947d7726de7587e2)).
- Remove `InitState` ([39b9855](https://github.com/lpc-rs/lpc8xx-hal/commit/39b9855bc3d09e42ed2257362c636b128ca98499)).
- Remove re-exports from `raw` ([2c42e18](https://github.com/lpc-rs/lpc8xx-hal/commit/2c42e18a932db79486a17cf131e68903f4c42116)).
- Simplify and improve USART API ([827874e](https://github.com/lpc-rs/lpc8xx-hal/commit/827874eab0c37f9195497ed7f054c6220fbe0770), [#122](https://github.com/lpc-rs/lpc8xx-hal/pull/122), [#123](https://github.com/lpc-rs/lpc8xx-hal/pull/123))
- Simplify and improve SYSCON API ([28d57ba](https://github.com/lpc-rs/lpc8xx-hal/commit/28d57baaa7c76ca9234cfd78892b34715f669d5c), [d06829d](https://github.com/lpc-rs/lpc8xx-hal/commit/d06829db3053632f394f67066d2fe381ec54e7df)).
- Add basic support for DMA ([#117](https://github.com/lpc-rs/lpc8xx-hal/pull/117), [#120](https://github.com/lpc-rs/lpc8xx-hal/pull/120), [#121](https://github.com/lpc-rs/lpc8xx-hal/pull/121)).


<a name="v0.2.0"></a>
## v0.2.0 (2018-06-15)

- Add support for additional low-power modes ([#106](https://github.com/lpc-rs/lpc8xx-hal/pull/106))
- Add basic I2C API ([#97](https://github.com/lpc-rs/lpc8xx-hal/pull/97))
- Implement `fmt::Write` for `USART` ([dbf1ec6a](dbf1ec6a))
- Drastically simplify compile-time state management ([#95](https://github.com/lpc-rs/lpc8xx-hal/pull/95))
- Drastically simplify switch matrix API ([#86](https://github.com/lpc-rs/lpc8xx-hal/pull/86), [#87](https://github.com/lpc-rs/lpc8xx-hal/pull/87), [#90](https://github.com/lpc-rs/lpc8xx-hal/pull/90), [#91](https://github.com/lpc-rs/lpc8xx-hal/pull/91), [#92](https://github.com/lpc-rs/lpc8xx-hal/pull/92), [#93](https://github.com/lpc-rs/lpc8xx-hal/pull/93))
- Expect USART RX/TX to be assigned on enable ([010d8982](010d8982))
- Remove macro re-exports ([1faea72f](1faea72f))
- Rename pin state transition methods ([593cf199](593cf199))
- Clean up peripheral state management ([#79](https://github.com/lpc-rs/lpc8xx-hal/pull/79))
- Re-export `lpc82x` as `lpc82x_hal::raw` ([468ddba0](468ddba0))
- Many additional cleanups and updates


<a name="v0.1.0"></a>
### v0.1.0 (2018-03-12)

Initial release
