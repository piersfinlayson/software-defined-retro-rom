# Changelog

All notables changes between versions are documented in this file.

## v0.2.1 - ???

### Changes

- Added pull-up/downs to X1/X2 in multi-ROM cases, so that when a multi-set ROM is configured, but X1/X2 are not connected, the other ROMs in the set still serve properly.
- Improved serving algorithm `B` in the CS active low case.
- Moved to algorithm `B` by default.
- Measured performance of both algorithm on all targets.
- Refactor `rom_impl.c`, breaking out assembly code to `rom_asm.h` to make the main_loop easier to read, and commonalising some code.
- Added detection of hardware reported STM32F4 device and flash size at runtime, and comparison to firmware values.
- Verified [hw revision e](/sdrr-pcb/verified/stm32f4-24-pin-rev-e/) - supports STM32F4x5 variants in addition to F401/F411, all passives are now 0603, a status LED and a 4th image select jumper.
- Added [documentation](/docs/STM32-CLONES.md) on STM32 clones.
- Moved firmware parsing to [`rust/sdrr-fw-parser`](/rust/sdrr-fw-parser/README.md) crate, which can be used to parse the firmware and extract information about the configuration, ROM images, and to extract ROM images from the firmware.  Done in preparation for using from a separate WiFi Programmer.
- Moved Rust code to [`rust/`](/rust/) directory.
- Added [build containers](/ci/docker/README.md) to assist with building SDRR, and doing so with the recommended build environment.

### Fixes

- Fixed status LED behaviour, by placing outside of MAIN_LOOP_ONE_SHOT, and using the configured pin.

## v0.2.0 - 2025-07-13

This version brings substantial improvements to the SDRR project, including:

- A single SDRR can be used to replace multiple ROM chips simultaneously.
- New [`sdrr-info`] tool to extract details and ROM images from firmware.
- Add your own hardware configurations, by adding a simple JSON file.
- New STM32F4 variants supported.
- Comprehensive testing of compiled in images to ensure veracity.

Care has been taken to avoid non-backwards compatible interface (such as CLI) changes, but some may may have been missed.  If you find any, please report them as issues.

### New Features

- Added support for ROM sets, allowing SDRR to serve multiple ROM images simultaneously, for certain combinations of ROM types.  This is done by connecting just the chip selects from other, empty sockets to be served, to pins X1/X2 (hardware revision 24-f onwards).  Currently tested only on VIC-20 (PAL) and C64 (PAL), serving kernal and BASIC ROMs simultaneously on VIC-20 and kernal/BASIC/character ROMs simultaneously on the C64.  See [Multi-ROM Sets](/docs/MULTI-ROM-SETS.md) for more details.
- Added [`sdrr-info`](/rust/sdrr-info/README.md) tool to parse the firmware and extract information about the configuration, ROM images, and to extract ROM images from the firmware.  In particular this allows
  - listing which STM32F4 device the firmware was built for
  - extraction of ROM images from the firmware, for checksumming and/or comparing with the originals.
- Moved hardware configuration to a dynamic model, where the supported hardware configurations are defined in configuration files, and the desired version is selected at build time.  Users can easily add configurations for their own PCB layouts, and either submit pull requests to include them in the main repository, or keep them locally.  For more details see [Custom Hardware](/docs/CUSTOM-HARDWARE.md).
- Added support F446 STM32F446R C/E variants - max clock speed 180 MHz (in excess of the F405's 168 MHz).  Currently untested.
- Added [`test`](/test/README.md), to verify the images source code files which are built into the firmware image, output the correct bytes, given the mangling that has taken place.

### Changes

- Updated VIC-20 PAL config to use VIC-20 dead-test v1.1.01.
- 24-pin PCB rev E/F gerbers provided (as yet unverified).
- Many previously compile time features now moved to runtime.
- Makefile produces more consistent and less verbose output.
- Added `sddr_info` struct to main firmware, containing firmware properties, for use at runtime, and later extraction.  This should also allow querying the firmware of a running system via SWD in future.

### Fixed

- Moved to fast speed outputs for data lines, instead of high speed, to ensure VOL is 0.4V, within the 6502's 0.8V requirement.  With high speed outputs, the VOL can be as high as 1.3V, which is beyond the 6502's 0.8V requirement.

## v0.1.0 - 2025-06-29

First release of SDRR.

- Supports F401, F411, and F405 STM32F4xxR variants.
- Includes configurations and pre-built firmware for C64, VIC-20 PAL, PET, 1541 disk drive, and IEEE disk drives.
- PCB rev D design included.
- Release binaries
