# Changelog

All notables changes between versions are documented in this file.

## v0.1.1 - TBC

### New Features

- Added support for ROM sets - groups of ROM images that can be served by a single SDRR, by connecting just the CS from other, empty sockets to pins X1/X2 (hardware revision F onwards).
- Added support for F446 STM32F4xxR C/E variants - max clock speed 180 MHz (in excess of the F405's 168 MHz).  Currently untested.
- Added [`sdrr-info`](/sdrr-info/README.md) tool to parse the firmware and extract information about the configuration, ROM images, and to extract ROM images from the firmware.  In particular this allows
  - listing which STM32F4 device the firmware was built for
  - extraction of ROM images from the firmware, for checksumming and/or comparing with the originals.

### Changes

- Updated VIC-20 PAL config to use VIC-20 dead-test v1.1.01.
- PCB rev E/F gerbers provided (as yet unverified).
- Moved to a runtime configuration model, instead of compile time, for many features.

### Fixed

- Moved to fast speed outputs for data lines, instead of high speed, to ensure VOL is 0.4V, within the 6502's 0.8V requirement.  With high speed outputs, the VOL can be as high as 1.3V, which is beyond the 6502's 0.8V requirement.

## v0.1.0 - 29-06-2025

First release of SDRR.

- Supports F401, F411, and F405 STM32F4xxR variants.
- Includes configurations and pre-built firmware for C64, VIC-20 PAL, PET, 1541 disk drive, and IEEE disk drives.
- PCB rev D design included.
- Release binaries
