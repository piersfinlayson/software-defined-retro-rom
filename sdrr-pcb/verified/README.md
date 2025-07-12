# Verified PCB Revisions

This folder contains design and manufacturing files for verified revisions of the SDRR PCB.  These revisions have been tested and confirmed to work with specific STM32 microcontroller variants and are supported by the stock SDRR firmware using the `HW_REV` field in the top-level [Makefile](/Makefile).

Revisions:

- [STM32F4-24-pin Rev D](./stm32f4-24-pin-rev-d/README.md) - 24-pin ROM replacement:
  - Tested with STM32F401 and STM32F411 variants
  - Tested as 2364 and 2332 ROM replacements in PET, VIC-20 and C64 (50Hz/PAL)
