# Changelog

All notables changes between versions are documented in this file.

## v0.1.1 - TBC

This version brings substantial improvements to the SDRR project, including:

- A single SDRR can be used to replace multiple ROM chips simultaneously.
- New tool to extract details and ROM images from firmware.
- Add your own hardware configurations, by adding a simple JSON file.
- New STM32F4 variants supported.

### New Features

- Added support for ROM sets, allowing SDRR to serve multiple ROM images simultaneously, for certain combinations of ROM types.  This is done by connecting just the chip selects from other, empty sockets to be served, to pins X1/X2 (hardware revision 24-f onwards).  Currently tested only on VIC-20 (PAL), serving kernal and BASIC ROMs simultaneously.
- Added [`sdrr-info`](/sdrr-info/README.md) tool to parse the firmware and extract information about the configuration, ROM images, and to extract ROM images from the firmware.  In particular this allows
  - listing which STM32F4 device the firmware was built for
  - extraction of ROM images from the firmware, for checksumming and/or comparing with the originals.
- Moved hardware configuration to a dynamic model, where the supported hardware configurations are defined in configuration files, and the desired version is selected at build time.  Users can easily add configurations for their own PCB layouts, and either submit pull requests to include them in the main repository, or keep them locally.  For more details see [Custom Hardware](/docs/CUSTOM-HARDWARE.md).
- Added [`test`](/test/README.md) suite, to verify the images source code files which are built into the firmware image, output the correct bytes, given the mangling that has taken place.
- Added support F446 STM32F446R C/E variants - max clock speed 180 MHz (in excess of the F405's 168 MHz).  Currently untested.

### Changes

- Updated VIC-20 PAL config to use VIC-20 dead-test v1.1.01.
- 24-pin PCB rev E/F gerbers provided (as yet unverified).
- Most previously compile time features now moved to runtime.
- Makefile produces more consistent and less verbose output.
- Added `sddr_info` struct to main firmware, containing firmware properties, for use at runtime, and later extraction.

### Fixed

- Moved to fast speed outputs for data lines, instead of high speed, to ensure VOL is 0.4V, within the 6502's 0.8V requirement.  With high speed outputs, the VOL can be as high as 1.3V, which is beyond the 6502's 0.8V requirement.

## v0.1.0 - 29-06-2025

First release of SDRR.

- Supports F401, F411, and F405 STM32F4xxR variants.
- Includes configurations and pre-built firmware for C64, VIC-20 PAL, PET, 1541 disk drive, and IEEE disk drives.
- PCB rev D design included.
- Release binaries
