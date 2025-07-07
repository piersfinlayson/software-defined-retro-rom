# Multiple ROM Sets

As of version v0.1.1, SDRR hardware revision F onwards supports multiple ROM image sets.  A set is a group of up to 3 ROM images, all of which SDRR will serve, so long as all of the following conditions are met:

- All the ROMs being replaced share the same address and data buses.
- SDRR is installed in the socket of the first ROM in the set to be replaced.
- The other ROM sockets to be replaced are empty.
- The chip select line (pin 20) of the other, empty, ROM sockets to be replaced are connected to SDRR pins X1 (2nd image of set) and X2 (optional 3rd image of set).

This feature is currently available only for 2364 ROMs, and 2332/2316 ROMs whose CS2/CS3 lines are permanently tied to the active state.  This is true, for example, for the C64 character ROM.  You can mix ROM types, so long as this condition is met - so both C64 2364 kernal/basic ROM images can share a set with the C64 2332 character ROM image.

You can find pre-built configurations in [`config`](/config/README.md), named `set-*.mk`.

It is recommended to have a (10%+) faster STM32F4 variant than standard speed test results documented in [STM32 Selection](/docs/STM32-SELECTION.md) suggest, as the serving algorithm for rom sets is slightly less efficient than for single ROM images.

## Set Configuration

Under the covers, the code now always uses the concepts of sets, even if they are not specified - in this case, each ROM image is in its own set.  Hence the existing configuration model remains supported.

To explicitly define sets, use a `ROM_CONFIGS` setting like this:

```Makefile
ROM_CONFIGS = \
	set=0,file=rom_2364_1.bin,type=2364,cs1=0 \
	set=0,file=rom_2332_1.bin,type=2332,cs1=0,cs2=ignore \
	set=0,file=rom_2364_1.bin,type=2364,cs1=0 \
	set=1,file=rom_2364_3.bin,type=2364,cs1=0
```

Each set can contain 1, 2 or 3 images, and are selected by the regular CS line, X1 and X2 respectively.  All CS1 values must be the same within a set - `sdrr-gen` attempts to detect and reject invalid configurations.

## Future Enhancements

### 2332s With Varying CS2

It should be possible to support a set of 2332 ROMs with CS2 varying, so long as all of the chips being replaced share the same CS2 line (i.e. are connected together).  The serving algorithm to implement this may be less efficient again than the stock mutli-rom set serving algorithm, but it is likely to be required by less powerful machines, so this is unlikely to be an issue.

It may be possible support multiple 2316 ROMs with varying CS2/CS3, so long as all CS2s are tied together and all CS3s are tied together.

It may also be possible to support combined 2332/2316 sets, so long as CS2 on the 2316 is permanantly active (and CS2 for all the 2332 and CS3 for the 2316s are all tied together).  (Somewhat confusingly, 2332 CS2 and 2316 CS3 are the same pin - 2316 CS2 is distinct.)

## Technical Details

In order to implemented this feature, all ROM images in the set are combined into a single 64KB image before storing into the firmware, and then the entirety of the STM32 port C GPIO state is read as an offset into that image. The ROM images are placed in the correct place for this lookup to work by the pre-processing step, using `sdrr-gen`.

If only 2 ROM images are in the set, the final bit of port C (PC15) is expected to be unconnected, and pulled low internally, just as when serving a single ROM image, PC14/PC15 are both pulled low, to ensure a 16KB offset into a 16KB image.  A 64Kb image is still used in the 2 ROM images in a set case, although a future enhancement could reduce the space required to 32KB.

A different ROM serving algorithm is used in the rom set case, to serve addresses only when a chip select line is active, and to detect if _any_ chip select line goes active.  The standard algorithm:

- is constantly reading the address lines and looking up the data byte even when CS isn't selected - this isn't possible here as the CS line going active determines which byte to read
- detects when _all_ chip selects (for that ROM type) go active, and only sets the data lines to output when this happens.

It is this different algorithm that is roughly 10% less efficient than the single ROM image serving algorithm.  The pre-loading of 64KB at startup instead of 16KB probably also adds around 200-300us at boot.  Given systems typically have a decent reset circuit, this is not expected to be an issue.

## Acknowledgements

Original suggestion of this enhancement was made by [Adrian Black](https://www.youtube.com/@adriansdigitalbasement).
