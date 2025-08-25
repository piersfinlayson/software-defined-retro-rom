# Verified PCB Revisions

This folder contains design and manufacturing files for verified revisions of the One ROM PCB.

These revisions have been tested and confirmed to work with specific MCU microcontroller variants and are supported by the stock One ROM firmware using the `HW_REV` field in the top-level [Makefile](/Makefile).

## RP2350

The current recommended hardware revision is A.  The RP2350 design is less mature than the STM32F4 design.

- [Rev A](./rp2350-24-pin-rev-a/README.md)
  - `HW_REV=p24-a`

## STM32F4

The current recommended hardware revision is G, although it has only been verified with STM32F411RET6.  It is highly likely to work with all of the other supports models.

- [Rev G](./stm32f4-24-pin-rev-g/README.md)
  - `HW_REV=24-g`
  - Partially verified, using STM32F411RET6.
  - New 2x3 programming header, including BOOT0.
  - Additional image select jumper.
  - Image select jumpers start from the other side.
  - Closing image select/X jumpers pulls to ground.
  - Additional 3V3/GND pins exposed.
  - MCO1 test point moved to underside of board.
  - Status LED moved next to programming header.
  - Moved to 0402 passives. 
- [Rev F2](./stm32f4-24-pin-rev-f2/README.md)
  - Silk screen differences
  - Includes all files required to have the PCB manufactured and assembled.
- [Rev F](./stm32f4-24-pin-rev-f/README.md) - Adds additional pins for multi-ROM support.
- [STM32F4 24 pin Rev E](./stm32f4-24-pin-rev-e/README.md) - Supports STM32F401, STM32F411, STM32F405 and STM32F415 variants.
  - Tested with STM32F401, STM32F405 and GD32F405 variants
  - Includes status LED.
- [STM32F4 24 pin Rev D](./stm32f4-24-pin-rev-d/README.md) - 24-pin ROM replacement:
  - Tested with STM32F401 and STM32F411 variants
  - Tested as 2364 and 2332 ROM replacements in PET, VIC-20 and C64 (50Hz/PAL)
