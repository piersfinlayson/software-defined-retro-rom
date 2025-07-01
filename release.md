# Software Defined Retro ROM - Release Notes

Provides a highly configurable and low-cost ROM replacement solution for vintage computers using STM32F4 microcontrollers.

## Quick Start

**Most users:** Depending on your flashing method, either download the **bin-*.zip** files or the **elf-*.zip** files.

**For developers:** Download **full-*.zip** files for complete build outputs including disassembly and map # Software Defined Retro ROM - Release Notes

## File Types

- **bin-stm32_variant.zip** - Binary files for STM32 variant (ready to flash)
- **elf-stm32_variant.zip** - ELF firmware files for STM32 variant (ready to flash)
- **full-stm32_variant.zip** - Complete build output (bin/elf/dis/map files)

## Supported Configurations

Each zip firmware with a selection of ROM images for multiple system configurations, including:

- C64
- VIC-20 PAL  
- PET
- Commodore 1541 disk drive
- IEEE disk drives
- And more...

Firmware for ROM image configurations that require explicit licence acceptance (for example ['c64'](/config/c64.mk)) are not included in the release.  You can build them yourself, by following instructions in the [README](/README.md) and [INSTALL](/INSTALL.md).

## Installation

1. Download the appropriate **bin-*.zip** for your STM32 variant
2. Extract the zip file  
3. Flash the desired configuration file (e.g., **vic20-pal-f411re.bin**) to your STM32
4. Connect according to your hardware setup

## Changes In This Release

[CHANGELOG_PLACEHOLDER]

## Support

For documentation, hardware setup guides, and support:

- GitHub: [Project](https://github.com/piersfinlayson/software-defined-retro-rom)
- YouTube: [@piers.rocks](https://www.youtube.com/@piers_rocks)
