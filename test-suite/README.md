# LPC8xx HAL Test Suite

## About

This is a unit testing suite for LPC8xx HAL, based on [defmt-test]. It is far from complete, but future contributors are invited to add tests here to test new functionality.


## Running

All of the following commands need to be executed from the repository root.

To run tests for all supported targets (requires an [LPCXpresso824-MAX] and an [LPC845-BRK] to be connected via USB):
```
./scripts/test.sh
```

To run tests for LPC82x (requires an [LPCXpresso824-MAX] to be connected via USB):
```
./scripts/test.sh 82x
```

To run tests for LPC845 (requires an [LPC845-BRK] to be connected via USB):
```
./scripts/test.sh 845
```


## Troubleshooting

The LPCXpresso824-MAX has been very finicky on my machine (probe not found), but it usually works when re-running it a few times. I'm not sure, if this is a local problem, or something that could be fixed in probe-rs.


[defmt-test]: https://github.com/knurling-rs/defmt/tree/main/firmware/defmt-test
[LPCXpresso824-MAX]: https://www.nxp.com/design/microcontrollers-developer-resources/lpcopen-libraries-and-examples/lpcxpresso824-max-board-for-lpc82x-family-mcus:OM13071
[LPC845-BRK]: https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK
