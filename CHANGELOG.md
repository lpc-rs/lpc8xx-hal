<a name="v0.8.1"></a>
## v0.8.1 (2020-08-21)

- Attempt to fix fix docs.rs build ([#277])

[#277]: https://github.com/lpc-rs/lpc8xx-hal/pull/277


<a name="v0.8.0"></a>
## v0.8.0 (2020-08-15)

- Improve I2C API ([#251], [#252])
- Add support for I2C slave mode ([#253], [#254])
- Improve SPI API ([#255], [#256])
- Seal `Instance` traits ([#257])
- Add support for SPI slave mode ([#259])
- Improve DMA API ([#260], [#262], [#263])
- Add DMA read support to USART API ([#261])
- Add DMA support for I2C master mode ([#264])
- Add DMA support for SPI master mode ([#265])
- Improve USART API ([#266], [#267], [#268], [#270])
- Add support for hardware flow control to USART API ([#269])
- Add support for synchronous mode to USART API ([#271])
- Add support for address matching to USART API ([#272])

[#251]: https://github.com/lpc-rs/lpc8xx-hal/pull/251
[#252]: https://github.com/lpc-rs/lpc8xx-hal/pull/252
[#253]: https://github.com/lpc-rs/lpc8xx-hal/pull/253
[#254]: https://github.com/lpc-rs/lpc8xx-hal/pull/254
[#255]: https://github.com/lpc-rs/lpc8xx-hal/pull/255
[#256]: https://github.com/lpc-rs/lpc8xx-hal/pull/256
[#257]: https://github.com/lpc-rs/lpc8xx-hal/pull/257
[#259]: https://github.com/lpc-rs/lpc8xx-hal/pull/259
[#260]: https://github.com/lpc-rs/lpc8xx-hal/pull/260
[#261]: https://github.com/lpc-rs/lpc8xx-hal/pull/261
[#262]: https://github.com/lpc-rs/lpc8xx-hal/pull/262
[#263]: https://github.com/lpc-rs/lpc8xx-hal/pull/263
[#264]: https://github.com/lpc-rs/lpc8xx-hal/pull/264
[#265]: https://github.com/lpc-rs/lpc8xx-hal/pull/265
[#266]: https://github.com/lpc-rs/lpc8xx-hal/pull/266
[#267]: https://github.com/lpc-rs/lpc8xx-hal/pull/267
[#268]: https://github.com/lpc-rs/lpc8xx-hal/pull/268
[#269]: https://github.com/lpc-rs/lpc8xx-hal/pull/269
[#270]: https://github.com/lpc-rs/lpc8xx-hal/pull/270
[#271]: https://github.com/lpc-rs/lpc8xx-hal/pull/271
[#272]: https://github.com/lpc-rs/lpc8xx-hal/pull/272

<a name="v0.7.1"></a>
## v0.7.1 (2020-06-26)

- Fix `embedded-hal` version of `set_low` ([#246])
- Add more infallible GPIO methods ([#247])

[#246]: https://github.com/lpc-rs/lpc8xx-hal/pull/246
[#247]: https://github.com/lpc-rs/lpc8xx-hal/pull/247


<a name="v0.7.0"></a>
## v0.7.0 (2020-06-22)

- Improve documentation ([#180], [#200], [#231], [#233], [#234])
- Improve `cargo run` configuration ([#181], [#235])
- Enforce consistent formatting with rustfmt ([#184], [#186])
- Fix and improve build script ([#185], [#187], [#193], [#194], [#197], [#213])
- Fix and improve USART API ([#188], [#199], [#206], [#208], [#210], [#221], [#223])
- Implement digital input ([#189])
- Fix wrong initial SWM state on LPC845 ([#190])
- Re-export clock source enum from `frg` module ([#195])
- Add PWM implementation based on CTIMER ([#196])
- Update prelude ([#198])
- Add MRT API ([#201], [#230])
- Add LPC845 support to I2C API ([#202])
- Add ADC API ([#203])
- Add SPI API ([#204])
- Opt into default `ToggleableOutputPin` impl ([#207])
- Remove core peripherals from `Peripherals` ([#209])
- Improve pin/GPIO API ([#214], [#216], [#217], [#219], [#220], [#224], [#225], [#226], [#228], [#242])
- Simplify `GPIO`/`SWM` constructors ([#218])
- Split some modules to improve readability ([#222])
- Add PININT API ([#227], [#229])
- Clean up and improve I2C API ([#236], [#240], [#241])
- Clean up peripheral clock source API ([#237], [#238], [#239])
- Migrate from RTFM to RTIC ([#243])

[#180]: https://github.com/lpc-rs/lpc8xx-hal/pull/180
[#181]: https://github.com/lpc-rs/lpc8xx-hal/pull/181
[#184]: https://github.com/lpc-rs/lpc8xx-hal/pull/184
[#185]: https://github.com/lpc-rs/lpc8xx-hal/pull/185
[#186]: https://github.com/lpc-rs/lpc8xx-hal/pull/186
[#187]: https://github.com/lpc-rs/lpc8xx-hal/pull/187
[#188]: https://github.com/lpc-rs/lpc8xx-hal/pull/188
[#189]: https://github.com/lpc-rs/lpc8xx-hal/pull/189
[#190]: https://github.com/lpc-rs/lpc8xx-hal/pull/190
[#193]: https://github.com/lpc-rs/lpc8xx-hal/pull/193
[#194]: https://github.com/lpc-rs/lpc8xx-hal/pull/194
[#195]: https://github.com/lpc-rs/lpc8xx-hal/pull/195
[#196]: https://github.com/lpc-rs/lpc8xx-hal/pull/196
[#197]: https://github.com/lpc-rs/lpc8xx-hal/pull/197
[#198]: https://github.com/lpc-rs/lpc8xx-hal/pull/198
[#199]: https://github.com/lpc-rs/lpc8xx-hal/pull/199
[#200]: https://github.com/lpc-rs/lpc8xx-hal/pull/200
[#201]: https://github.com/lpc-rs/lpc8xx-hal/pull/201
[#202]: https://github.com/lpc-rs/lpc8xx-hal/pull/202
[#203]: https://github.com/lpc-rs/lpc8xx-hal/pull/203
[#204]: https://github.com/lpc-rs/lpc8xx-hal/pull/204
[#206]: https://github.com/lpc-rs/lpc8xx-hal/pull/206
[#207]: https://github.com/lpc-rs/lpc8xx-hal/pull/207
[#208]: https://github.com/lpc-rs/lpc8xx-hal/pull/208
[#209]: https://github.com/lpc-rs/lpc8xx-hal/pull/209
[#210]: https://github.com/lpc-rs/lpc8xx-hal/pull/210
[#213]: https://github.com/lpc-rs/lpc8xx-hal/pull/213
[#214]: https://github.com/lpc-rs/lpc8xx-hal/pull/214
[#216]: https://github.com/lpc-rs/lpc8xx-hal/pull/216
[#217]: https://github.com/lpc-rs/lpc8xx-hal/pull/217
[#218]: https://github.com/lpc-rs/lpc8xx-hal/pull/218
[#219]: https://github.com/lpc-rs/lpc8xx-hal/pull/219
[#220]: https://github.com/lpc-rs/lpc8xx-hal/pull/220
[#221]: https://github.com/lpc-rs/lpc8xx-hal/pull/221
[#222]: https://github.com/lpc-rs/lpc8xx-hal/pull/222
[#223]: https://github.com/lpc-rs/lpc8xx-hal/pull/223
[#224]: https://github.com/lpc-rs/lpc8xx-hal/pull/224
[#225]: https://github.com/lpc-rs/lpc8xx-hal/pull/225
[#226]: https://github.com/lpc-rs/lpc8xx-hal/pull/226
[#227]: https://github.com/lpc-rs/lpc8xx-hal/pull/227
[#228]: https://github.com/lpc-rs/lpc8xx-hal/pull/228
[#229]: https://github.com/lpc-rs/lpc8xx-hal/pull/229
[#230]: https://github.com/lpc-rs/lpc8xx-hal/pull/230
[#231]: https://github.com/lpc-rs/lpc8xx-hal/pull/231
[#233]: https://github.com/lpc-rs/lpc8xx-hal/pull/233
[#234]: https://github.com/lpc-rs/lpc8xx-hal/pull/234
[#235]: https://github.com/lpc-rs/lpc8xx-hal/pull/235
[#236]: https://github.com/lpc-rs/lpc8xx-hal/pull/236
[#237]: https://github.com/lpc-rs/lpc8xx-hal/pull/237
[#238]: https://github.com/lpc-rs/lpc8xx-hal/pull/238
[#239]: https://github.com/lpc-rs/lpc8xx-hal/pull/239
[#240]: https://github.com/lpc-rs/lpc8xx-hal/pull/240
[#241]: https://github.com/lpc-rs/lpc8xx-hal/pull/241
[#242]: https://github.com/lpc-rs/lpc8xx-hal/pull/242
[#243]: https://github.com/lpc-rs/lpc8xx-hal/pull/243


<a name="v0.6.1"></a>
## v0.6.1 (2019-11-09)

- Fix build.rs ([#178])

[#178]: https://github.com/lpc-rs/lpc8xx-hal/pull/178


<a name="v0.6.0"></a>
## v0.6.0 (2019-11-09)

- Improve LPC845 support ([#163], [#165])
- Re-export dependencies ([#164])
- Implement delay based on SysTick ([#166], [#169])
- Re-export PAC as `pac` ([#168])
- Enable WKT for LPC845 ([#171])
- Merge IRC and FRO APIs into IOSC ([#172])
- Upgrade to latest `lpc845-pac` release ([#174])

[#163]: https://github.com/lpc-rs/lpc8xx-hal/pull/163
[#164]: https://github.com/lpc-rs/lpc8xx-hal/pull/164
[#165]: https://github.com/lpc-rs/lpc8xx-hal/pull/165
[#166]: https://github.com/lpc-rs/lpc8xx-hal/pull/166
[#168]: https://github.com/lpc-rs/lpc8xx-hal/pull/168
[#169]: https://github.com/lpc-rs/lpc8xx-hal/pull/169
[#171]: https://github.com/lpc-rs/lpc8xx-hal/pull/171
[#172]: https://github.com/lpc-rs/lpc8xx-hal/pull/172
[#174]: https://github.com/lpc-rs/lpc8xx-hal/pull/174


<a name="v0.5.0"></a>
## v0.5.0 (2019-10-12)

- Add initial support for LPC845; rename to `lpc8xx-hal` ([#150], [#151], [#152], [#155], [#161])
- Update dependencies ([#160])

[#150]: https://github.com/lpc-rs/lpc8xx-hal/pull/150
[#151]: https://github.com/lpc-rs/lpc8xx-hal/pull/151
[#152]: https://github.com/lpc-rs/lpc8xx-hal/pull/152
[#155]: https://github.com/lpc-rs/lpc8xx-hal/pull/155
[#160]: https://github.com/lpc-rs/lpc8xx-hal/pull/160
[#161]: https://github.com/lpc-rs/lpc8xx-hal/pull/161


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
