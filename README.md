# Software Defined Retro ROM (SDRR)

A highly configurable and low-cost ROM replacement solution for vintage computers.  Replace failed ROMs in Commodore 64s, VIC-20s, PETs, and other retro systems with a $2 microcontroller.

Emulates 2364, 2332 and 2316 ROMs using the STM32F4 family of microcontrollers.  Replaces kernel, BASIC, character set, diagnostic, and application ROMs.

## Key Features

- 💰 Based on **sub $2 microcontroller**.
- ⚡ **Fast** enough for PETs, VIC-20s, C64s, 1541s, IEEE drives, etc. Support faster systems with STM32F405R.
- 📐 **Same footprint** as original ROMs - doesn't overhang the socket like other solutions.
- 🚀 **Quick programming** - just connect 4 wires and `make run`. No programming jigs necessary.
- 🔌 **Use a $5 programmer** - no expensive EEPROM programmer required.
- 🛠️ **Reflash in situ** - no need to remove the ROM from the host when reprogramming.
- ⚙️ **Software configurable** chip select lines - no hardware jumpers required.
- 💾 Stores up to **16 ROM images** of different sizes and chip select configurations.  Image selectable via jumpers.
- 📦 **Replace multiple ROMs with one ROM** one SDRR can replace up to 3 original ROMs e.g. all of C64 kernel, BASIC, character set.
- 🔀 **Dynamic bank switching** - switch between ROM images on the fly, e.g. different char ROMs.
- 🧩 **Images combined automatically** - no need to manually build up your own larger PROM image containing multiple retro ROMs.
- 🏭 **Two layer PCB**, component on single-side, limited BOM for low manufacturing cost/complexity.
- 🎯 Supports multiple **STM32F4xxR** variants: F401, F411, F405, F446 (others can be added).
- 🔓 **Open source** software and hardware - see [License](LICENSE.md)

## Introduction

The video below provides an introduction to the Software Defined Retro ROM:

[![Video Title](https://img.youtube.com/vi/Jhe4LF5LrZ8/maxresdefault.jpg)](https://youtu.be/Jhe4LF5LrZ8)

## Hardware

This is the STM32F4 24-pin version, hardware revison F.  See [sdrr-pcb](sdrr-pcb/README.md) for the hardware designs and documentation.

<div style="display: flex; justify-content: center; gap: 20px;">
  <a href="docs/images/sdrr-24-pin-side.png">
    <img src="docs/images/sdrr-24-pin-side.png" alt="SDRR STM32F4 24 pin side on" width="400">
  </a>
</div>

## Quick Start

You have two options

- [use the pre-built binaries](#pre-built-binaries)
- [build the firmware yourself.](#build-yourself)

### Pre-built Binaries

Use your favourite STM32 programmer to flash the pre-built binaries from the project's [releases](https://github.com/piersfinlayson/software-defined-retro-rom/releases/) page.  Scroll down to the "Assets" section of the latest release, and download the `bin-*.zip` or `elf-*.zip` file for your STM32 variant.  These zip files contain pre-built binaries for the various SDRR image collections, including the C64, VIC-20, PET, and 1541 disk drive.

### Build Yourself

Once you have the project cloned, and the required [dependencies](INSTALL.md) installed, you can build and flash an SDRR image using the following commands - replace `f411re` with your [target STM32 variant](#supported-stm32-microcontrollers), and [`config/c64.mk`](/config/c64.mk) with the [configuration](config/README.md#available-configurations) you want to use.

```bash
# C64
STM=f411re CONFIG=config/c64.mk make run
```

```bash
# VIC-20 (PAL)
STM=f411re CONFIG=config/vic20-pal.mk make run
```

```bash
# PET 40 column 50Hz
STM=f411re CONFIG=config/pet-4-40-50.mk make run
```

This will download the desired ROM images automatically, generate the required firmware, and flash it to the SDRR device.

### After Flashing

Set the [SDRR jumpers](docs/IMAGE-SELECTION.md) to select the desired ROM image (see the [config file](/config/), or logs from your SDRR device, for which # corresponds to which image) and you're now ready to install the SDRR device in your retro system.

For configuration options, see [Configuration](docs/CONFIGURATION.md) and the [Makefile](Makefile).

## Documentation

- [Frequently Asked Questions](docs/FAQ.md) - Answers to common questions about SDRR.
- [Installation](INSTALL.md) - Installation of dependencies.
- [Programming](docs/PROGRAMMING.md) - How to program the SDRR device.
- [Available Configurations](config/README.md#available-configurations) - Various pre-collated ROM collection configurations.
- [STM32 Selection](docs/STM32-SELECTION.md) - How to select the appropriate STM32 variant for your application.
- [Image Selection](docs/IMAGE-SELECTION.md) - How to tell SDRR which of the installed ROM images to serve.
- [Multi-ROM Sets](docs/MULTI-ROM-SETS.md) - How to use a single SDRR to support multiple ROMs simultaneously or to dynamically switch between images.
- [Configuration](docs/CONFIGURATION.md) - SDRR configuration options.
- [Logging](docs/LOGGING.md) - How to enable and use logging in SDRR.
- [Technical Summary](docs/TECHNICAL-SUMMARY.md) - Overview of the SDRR architecture, operation and design decisions.
- [Technical Details](docs/TECHNICAL-DETAILS.md) - Technical details of the SDRR firmware and hardware.
- [Custom Hardware](docs/CUSTOM-HARDWARE.md) - Guide on designing custom hardware for SDRR.
- [Build System](docs/BUILD-SYSTEM.md) - How the SDRR build system works.
- [Voltage Levels](docs/VOLTAGE-LEVELS.md) - How the SDRR supports the required logic voltage levels.
- [Pi Pico Programmer](docs/PI-PICO-PROGRAMMER.md) - How to use a $5 Raspberry Pi Pico as a programmer for SDRR.
- [Future Enhancements](docs/FUTURE-ENHANCEMENTS.md) - Possible future enhancements under consideration.
- [License](LICENSE.md) - SDRR software and hardware licenses.

## Supported STM32 Microcontrollers

Most STM32F4xxR (LQFP-64) variants will work, but the following are supported out of the box by the build system:

| Model | Max single ROM Images | Max multi-ROM Sets |Build variant |
|-------|-----------------------|--------------------|--------------|
| STM32F401RBT6 | 6  | 1 | f401rb |
| STM32F401RCT6 | 14 | 3 | f401rc |
| STM32F401RET6 | 16 | 7 | f401re |
| STM32F411RCT6 | 14 | 3 | f411rc |
| STM32F411RET6 | 16 | 7 | f411re |
| STM32F405RGT6 | 16 | 15 | f405rg |
| STM32F446RCT6 | 16 | 3 | f446rc |
| STM32F446RET6 | 16 | 7 | f446re |

Selecting between more than 8 ROM images or sets requires hardware revision E onwards, as the PCB has more jumpers to select the images.

The vast majority of ROMs can be emulated by the cheapest variant, the F401.  The best bang for buck are likely to be either:

- STM32F401RET6 - [currently $1.93 on LCSC](https://lcsc.com/product-detail/Microcontrollers-MCU-MPU-SOC_ST-STM32F401RET6_C116978.html)
- STM32F411RET6 - [currently $2.69 on LCSC](https://lcsc.com/product-detail/Microcontrollers-MCU-MPU-SOC_ST-STM32F411RET6_C94355.html)

The F401 may require a small overclock for some systems, such as running the C64 multi-ROM set (~90MHz vs 84MHz rated maximum).

There are also cheap [clones available](docs/STM32-CLONES.md), with a clone F405 coming in at around the same price as a genuine F411 (from LCSC).

For more details about selecting the appropriate STM32 variant for your application, see [docs/STM32-SELECTION.md](docs/STM32-SELECTION.md).

If you'd like another variant supported, raise an issue via github or [add it yourself](docs/STM32-SELECTION.md#supporting-other-variants) and submit a PR.

## Debugging

The best place to start with debugging is [`Logging`](docs/LOGGING.md).  This will help you see what the SDRR is doing, and why it may not be working as expected.

## License

See [LICENSE](LICENSE.md) for software and hardware licensing information.
