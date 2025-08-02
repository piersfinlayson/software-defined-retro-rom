# Changelog

All notables changes between versions are documented in this file.

## v0.3.0 - ????-??-??

The main user facing change in this release is the addition of support for remote analysis and co-processing alongside the SDRR device via plug-ins, such as [Airfrog](https://piers.rocks/u/airfrog) - **a tiny $3 probe for ARM devices**.  This allows you to inspect the firmware and runtime state of the SDRR device, and change its configuration and ROM data - **while it is serving ROMs**.

There is also new ROM access counting feature, which causes SDRR to count how times the CS line(s) go active.  This can be extracted and visualised using [Airfrog](https://piers.rocks/u/airfrog) and other SWD probes, to determine how often the ROMs are accessed based on host activity.

![ROM Access Graph](docs/images/access-rate.png)

The Rust tooling has been substantially refactored to easier to integrate SDRR support in third-party tooling, such as [Airfrog](https://piers.rocks/u/airfrog).  In particular there is [Firmware Parser](rust/sdrr-fw-parser/README.md) crate, which can be used to parse the firmware from a file or running SDRR, and extract information about the configuration, ROM images, and to extract ROM images from the firmware.

### Changes

- TI-99/4A and CoCo2 configurations have been added to the [third-party configs](config/third-party/README.md) directory.  Thanks to [@keronian](https://github.com/keronian) for contributing these.
- Added a C main loop implementation for which GCC produces the assembly/machine code.  This requires a roughly 25% faster clock speed.  Use `EXTRA_C_FLAGS=-DC_MAIN_LOOP` when running `make` to use this version.
- Stored off image files used to create the firmware in `output/images/`.  This allows post build inspection of the images used.  It also enables additional tests - `sdrr-info` can be used as an additional check (along with `test`), to ensure the images in the firmware are correct, and validate the behaviour of `sdrr-info` and `test` to be compared.
- Substantial refactoring of `sdrr-gen`, to make it more maintainable.
- Substantial refactoring of `sdrr-fw-parser` in order to make it suitable for airfrog integration.
- Added ROM access counting behind COUNT_ROM_ACCESS feature flag.  When enabled, the firmware updates a u32 counter at RAM address 0x20000008 every time the chip select line(s) go active - i.e. the ROM is selected.  This can be read by an SWD probe, such as [Airfrog](https://piers.rocks/u/sdrr). 
- Changed default Makefile configuration to HW_REV=24-f COUNT_ROM_ACCESS=1 STATUS_LED=1.

### Fixes


## v0.2.1 - 2025-07-22

The main new feature in this version of SDRR is the addition of dynamic [bank switching](docs/MULTI-ROM-SETS.md#dynamic-bank-switching) of ROM images.  This allows SDRR to hold up to 4 different ROM images in RAM, and to switch between them **while the host is running** by using the X1/X2 pins (hardware revision F and later) to switch between them.  Some fun [C64](config/bank-c64-char-fun.mk) and [VIC-20](config/bank-vic20-char-fun.mk) character ROM configurations that support bank switching are included.

In other news:
- The default ROM serving algorithm has been improved, leading to better performance, and hence the ability to support more systems with lower powered STM32F4 devices than before.  Check out [STM32 Selection](docs/STM32-SELECTION.md). The current price/performance sweet spot is the F411.
- [Hardware revision E](sdrr-pcb/verified/stm32f4-24-pin-rev-e/README.md) is now fully verified, so manufacture these with confidence.
- If you'd rather use [revision F](sdrr-pcb/unverified/stm32f4-24-pin-rev-f/README.md) (required for multi-ROM and bank switching support), at least once user has reported getting these manufactured and working with his NTSC VIC-20 - although they did not testing either multi-ROM or bank switching support.

### Changes

- Added pull-up/downs to X1/X2 in multi-ROM cases, so that when a multi-set ROM is configured, but X1/X2 are not connected, the other ROMs in the set still serve properly.
- Improved serving algorithm `B` in the CS active low case.
- Moved to algorithm `B` by default.
- Measured performance of both algorithm on all targets.
- Refactor `rom_impl.c`, breaking out assembly code to `rom_asm.h` to make the main_loop easier to read, and commonalising a bunch of the code for greater maintainability.
- Added detection of hardware reported STM32F4 device and flash size at runtime, and comparison to firmware values - warning logs are produced in event of a mismatch.
- Verified [hw revision e](/sdrr-pcb/verified/stm32f4-24-pin-rev-e/) - supports STM32F4x5 variants in addition to F401/F411, all passives are now 0603, contains a status LED and a 4th image select jumper.
- Added [documentation](/docs/STM32-CLONES.md) on STM32 clones.
- Moved firmware parsing to [`rust/sdrr-fw-parser`](/rust/sdrr-fw-parser/README.md) crate, which can be used to parse the firmware and extract information about the configuration, ROM images, and to extract ROM images from the firmware.  Done in preparation for using the same code from a separate MCU.
- Moved Rust code to [`rust/`](/rust/) directory to declutter the repo a bit.
- Added experimenta; [build containers](/ci/docker/README.md) to assist with building SDRR, and doing so with the recommended build environment.
- Added dynamic [bank switchable](docs/MULTI-ROM-SETS.md#dynamic-bank-switching) ROM image support, using X1/X2 (you can use __either__ multi-ROM __or__ bank switching in a particular set).
- Added fun banked character ROM configs.
- Added VIC-20 NTSC config.
- Added retry in [ci/build.sh](ci/build.sh) to allow for intermittent network issues when downloading dependencies.
- Added [demo programs](demo/README.md) for C64 and VIC-20 to list SDRR features and other information/

### Fixes

- Fixed status LED behaviour, by placing outside of MAIN_LOOP_ONE_SHOT, and using the configured pin.
- Got `sdrr` firmware working on STM32F401RB/RC variants.  These have 64KB RAM, so can only support individual ROM images (quantity limited by flash) and do not support banked or multi-set ROMs.

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
