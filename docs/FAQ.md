# SDRR Frequently Asked Questions

## What is SDRR?

**Q: What exactly is Software Defined Retro ROM (SDRR)?**

A: SDRR is a ROM replacement solution that uses a cheap STM32F4 microcontroller to emulate failed or missing ROMs in retro computers. It is easily reprogrammed, can store up to 16 different ROM images and switch between them using jumpers, eliminating the need for expensive EEPROM programmers or multiple physical ROM chips.

**Q: Why would I use SDRR instead of buying replacement ROMs or using an EEPROM?**

A: SDRR offers several advantages:

- Much cheaper to program than xEPROMs ($5 programmer vs $40+ for xEPROM programmers).
- Fits the exact footprint of original ROMs, no overhanging the socket.
- Can be reprogrammed in-situ without removal, allowing sub-minute times to recompile and reflash firmware.
- Stores multiple images.
- Can support all required chip select line behaviours without any hardware changes.
- Is faster to build and program (just `make run` vs complex xEPROM procedures).
- Makes it easy to download and combine ROM images from various sources - no need to build combination ROM images by hand.

**Q: What ROM types does SDRR support?**

A: SDRR emulates 2364 (8KB), 2332 (4KB), and 2316 (2KB) ROMs commonly used in Commodore systems, disk drives, and other retro computers.

The original ROMs had mask programmable chip select behaviour, which meant they could be either low or high when selected. SDRR can emulate this by configuring the chip select lines in software, allowing it to work with any system that uses these ROM types - and can support multiple images with different chip select configurations in a single firmware build.

## Hardware Compatibility

**Q: Which STM32 variants are supported?**

A: Currently supported variants include:

- STM32F401RBT6/RCT6/RET6 (84MHz max clock, 6-16 ROM images)
- STM32F411RCT6/RET6 (100MHz max clock, 14-16 ROM images)  
- STM32F405RGT6 (168MHz max clock, 16 ROM images)
- STM32F446RCT6/RETx (180MHz max clock, 16 ROM images)

The F405/F446 are recommended for maximum compatibility as they runs fastest and supports the maximum number of target systems.

See the [Supported STM32 Microcontrollers](/README.md#supported-stm32-microcontrollers) section for more details.

**Q: What retro systems work with SDRR?**

A: Tested systems include Commodore PETs, VIC-20s, C64s, 1541 drives, and IEEE-488 drives. The F405 variant running at 168MHz should support virtually any system using compatible ROM types, including faster 2MHz systems.

**Q: How do I know which STM32 variant I need?**

A: Choose based on your target system's timing requirements. PETs need 26MHz minimum, VIC-20s need 37MHz, C64 kernals need 75MHz, and C64 character ROMs need 98MHz.

Use F405 or F446 if unsure - they supports all target systems.

See the [STM32 Selection](/docs/STM32-SELECTION.md) section for more details.

**Q: Why didn't you use a Raspberry Pi Pico/RP2040 or other microcontroller?**

A: The one sentence answer to this is that no other microcontroller had all of the required performance, 5V tolerant GPIOs and low cost.

In the case of the Raspberry Pi Pico/RP2040, its GPIOs are not 5V tolerant, and it would have required level shifters.  It also does not have internal flash (although a subsequent RP2350 model does).  This would have significantly increased the cost and complexity of the design, and almost certainly led to either 4 layer PCBs, or a larger PCB footprint, or both.  This is sad, because otherwise the RP2040/RP2350 would be a great fit for this project - it has a high clock speed, dual cores, enough RAM and a sufficient number of GPIOs.  Its PIO capability could probably have been put to great use.

There is a more detailed explanation of the STM32 choice in [Hardware Selection](/docs/TECHNICAL-DETAILS.md#hardware-selection).

Update 29 July 2025.  Raspberry Pi announced 5V tolerance in the A4 stepping (version) of the RP2350.  This makes a RP2350 version of the SDRR more feasible, but there are still issues:
- The RP2350 A4 is tolerant up to 5.5V.  However, the original chips in retro machines, and the STM32F4 are tolerant to 7V.  Voltages about 5.5V in old machines with questionable voltage regulators re not unheard of, and could lead to a dead RP2350.
- An external flash chip is still required, as it seems that the RP2354 (which built-in flash) is not yet 5V tolerant.
- 5V tolerance is limited to GPIOs 0-25 (this is sufficient for SDRR).
- An external oscillator is still required, as are other components in addition to the STM32F4 - such as more substantial power circuitry.  This continues makes a single sided layout challenging.
- The RP2350 is not really hand solderable.

If someone builds an RP2350 version of the SDRR a software port should be relatively easy!

**Q: Why are you using the STM32's internal oscillator - doesn't that make it unstable?**

A: An internal oscillator was one of the microcontroller selection criteria, in order to simplify the design, and reduce the PCB footprint and BOM cost.  The STM32F4 runs stably off its internal oscillator, although its accuracy is not as good as from an external crystal.  A precisely accurate clock is not required for this application - it primarily requires raw speed.

**Q: Why can't I find the STM32F4 for$2?**

A: You need to look at Chinese sites including [aliexpress.com](https://www.aliexpress.com) and [LCSC](https://www.lcsc.com), JLC PCB's component supplier ARM.

As of July 2025:

- LCSC had STM32F401 for $1.93 in quantities of 1 - good enough for PET, VIC-20 and C64 single ROMs
- LCSC had STM32F411 for $2.69 in quantities of 1 - also good enough for C64 multi-ROM sets
- aliexpress had STM32F405 for $3.08 in quantities of 1 - faster than the F401/F411
- aliexpress had GD32F405 $2.27 in quantities of 1 - GigaDevices produces clones of the STM32 parts

The aliexpress seller referred to above was also offering free shipping when buying more than 4 or 5 parts.

SDRR is extremely flexible in the STM32F4xxR variants it supports, so you can choose the cheapest one that meets your requirements.

**Q: Are clones just as good as the real deal STM32?**

A: Mostly, but not entirely.  The GigaDevices GD32F405 may take longer to boot than the STM32F405, but generally appears more performantSee [STM32 Clones](/docs/STM32-CLONES.md) for more details.

## Installation and Setup

**Q: What do I need to get started?**

A: You need:

- SDRR PCB
- a supported STM32F4 microcontroller
- some other components (voltage regulator, LED, resistors and capacitors)
- ARM GNU toolchain, Rust, probe-rs
- and an SWD programmer (ST-Link, Pi Debug Probe, or programmed Pi Pico).

See the [Installation Guide](INSTALL.md) for detailed setup instructions.

**Q: How difficult is the PCB assembly?**

A: Medium difficulty - some experience with surface mount soldering is recommended.

The rev D PCB uses a combination of 0402 and 0603 components.

Rev E/F PCBs uses all 0603 components, so are easier to solder.

All variants use an STM32 LQFP64 package, which requires soldering as it is close to other components.

**Q: Can I use a Raspberry Pi Pico as a programmer?**

A: Yes! Flash the debug probe firmware to a Pi Pico and connect CLK (GP2), DIO (GP3), GND, and optionally 5V (VBUS). This provides a $5 programming solution.

See the [Pi Pico Programmer Guide](/docs/PI-PICO-PROGRAMMER.md) for detailed instructions.

## Programming and Flashing

**Q: How do I flash firmware to SDRR?**

A: Connect your programmer's CLK, DIO, and GND lines to SDRR, then run:

```bash
STM=f411re CONFIG=config/c64.mk make run
```

This automatically downloads ROMs, generates firmware, and flashes the device.

There are other pre-collated collections of ROMs available in the [`config/`](/config/) directory, such as `vic20-pal.mk`, `pet-4-40-50.mk`, and `1541.mk`. You can also create your own custom configurations.

**Q: Can I reprogram SDRR while it's installed in my retro system?**

A: Yes! You can flash in-situ with the system powered on. The system may crash during flashing but will work normally after completion.

**Never** externally power SDRR when installed in a retro system.

**Q: What if I have flashing problems?**

A: Install the BOOT0 jumper and power cycle to force bootloader mode, then reflash. Alternatively, enable the `BOOTLOADER` config option to enter bootloader mode when all image select jumpers are closed.

See the [Programming Guide](/docs/PROGRAMMING.md) for more details.

**Q: How can I see what platform in my firmware image is for?**

A: You can use [`sdrr-info`](/rust/sdrr-info/README.md) to inspect firmware `.elf` and `.bin` files.

**Q: How can I see what ROMs are in my firmware image?**

A: You can use [`sdrr-info`](/rust/sdrr-info/README.md) to inspect firmware `.elf` and `.bin` files.

## Configuration and Usage

**Q: How do I select which ROM image to use?**

A: Use the image select jumpers labeled 1, 2, 4, and 8 on the PCB bottom.  Close jumpers to form a binary number (0-15) corresponding to your desired ROM image.  A closed jumper corresponds to 1, an open jumper 0.

You must reboot SDRR after changing jumpers.

**Q: How many ROM images can I store?**

A: Up to 16 images, limited by either flash size (smaller variants) or the physical jumper count. Each image consumes 16KB of flash regardless of actual ROM size.

**Q: Can I mix different ROM types in one firmware?**

A: Yes! Each ROM image can be independently configured as 2364, 2332, or 2316 with different chip select configurations. See the [config files](/config/) for examples.

**Q: How do I create custom ROM configurations?**

A: Define `ROM_CONFIGS` with file paths, ROM types, and chip select settings:

```bash
ROM_CONFIGS=file=kernal.rom,type=2364,cs1=0 file=basic.rom,type=2364,cs1=0
```

Or create your own [config file](/config/).

## Performance and Technical

**Q: Can you add feature X to SDRR?**

A: Perhaps - raise an issue on the [GitHub issues page](https://github.com/piersfinlayson/software-defined-retro-rom/issues).

SDRR is very busy serving ROM images, so has few spare cycles for additional features.  However, it can be augmented by a co-processor such as [Airfrog](https://piers.rocks/u/airfrog) to provide additional functionality, such as remote access and control, or telemetry.

**Q: Why does SDRR store all ROM images as 16KB in flash?**

A: This optimization allows the main loop to use chip select states as direct address offsets, eliminating runtime calculations and achieving the required access speeds.

See [Technical Details](/docs/TECHNICAL-DETAILS.md) for more information.

**Q: What makes SDRR fast enough for retro systems?**

A: Multiple optimizations: maximum clock speeds, highly optimised assembly main loop, preloaded ROM data in RAM, no interrupts, flash prefetch/caching, and "mangled" ROM storage that matches the PCB pin mapping.

See [Technical Details](/docs/TECHNICAL-DETAILS.md) for more information.

**Q: Why not use interrupts?**

A: Interrupts would add latency that could cause the system to miss critical timing windows. The polling-based approach ensures consistent, predictable response times.  As SDRR does nothing other than serve flash, a tight main loop with no interrupts is sufficient **and** optimal.

**Q: Isn't the STM32 a 3.3V device? How does it work in 5V systems?**

A: Most of the GPIO pins on the STM32F4xx are 5V tolerant, and the maximum and minimum input and output voltage levels are compatible with the main target retro driving ICs (such as the 6502, VIC and VIC-II) chips.  Care is taken to ensure that only 5V tolerant GPIOs are used, including setting the drive strength on the STM32 to stay within the required output levels.

There's a detailed analysis in [Voltage Levels](/docs/VOLTAGE-LEVELS.md).

**Q: What languages are used in SDRR?**

A: The main [`sdrr`](/sdrr/) firmware is written in a combination of C and assembly.  Assembly is used for the main loop to achieve the required performance, while C is used for the startup-up logic.

C was chosen above other lanuages in order to stay as close as possible to the hardware, for performance reasons.  The C code is fully bare-metal - no hardware abstraction layer or other libraries are used, except [SEGGER-RTT](https://github.com/piersfinlayson/segger-rtt) for logging.

The [`sdrr-gen`](/rust/sdrr-gen/README.md) tool, which generates the ROM images and other code to embed in the main firmware is written in Rust.

**Q: What does the build pipeline look like.**

```ascii
User Input → Top-level Make → sdrr-gen → sdrr Make → sdrr-info → test (optional) → probe-rs → STM32
    ↓              ↓            ↓          ↓             ↓              ↓              ↓        ↓
Config Files  ROM Download    Code Gen    Compile      Query      Compile tests      Flash    Running
ROM Images    & Validation    & Mangle    & Link      Firmware        & Run         Firmware
```

See [Build System](/docs/BUILD-SYSTEM.md) for more details on how the build system works.

## Troubleshooting

**Q: How do I debug SDRR issues?**

A: Enable boot logging (default) to see startup information via SWD/RTT. This shows hardware info, ROM configurations, frequency settings, and which image is loaded. Boot logging adds ~1.5ms to startup time.

See [Logging](/docs/LOGGING.md) for more details.

**Q: My retro system doesn't boot with SDRR installed. What should I check?**

A: Verify the correct ROM type and chip select configuration, ensure SDRR is fully seated in the socket, check that the STM32 variant can run fast enough for your system, and review boot logs for errors.

It would be worth reprogramming SDRR with your firmware image in-situ, and viewing the logs to ensure the expected ROM image is being served.

See [`BOOT_LOGGING`](/docs/LOGGING.md#boot_logging) for more details on how to interpret boot logging.

**Q: Can I reduce power consumption?**

A: Lower the clock frequency using the `FREQ` option, disable the status LED with `STATUS_LED=0`, or choose a lower-power STM32 variant if it meets your timing requirements.

[Clock Speed Requirements](/docs/STM32-SELECTION.md#clock-speed-requirements) provides guidance on selecting the right frequency for your system, but you may need to experiment to find the best balance between performance and power consumption.

## Cost and Availability

**Q: How much does SDRR cost to build?**

A: In small quantities, expect around $5 per device.  The STM32F4 costs around $2 depending on variant, PCB manufacturing is inexpensive (2-layer, single-side components), and the BOM is minimal.

**Q: Where can I get the PCB?**

A: Order the latest revision from [OSH Park](https://oshpark.com) using the provided link for your chosen revision in the [`sdrr-pcb`](/sdrr-pcb/README.md), or manufacture using the open-source design files.

**Q: Is SDRR open source?**

A: Yes! Software/firmware uses MIT license, hardware uses CC BY-SA 4.0. You can modify, improve, and share the designs within the license terms.

**Q: Is there a warranty?**

A: In common with most open source projects, no warranty is provided.  See [License](/LICENSE.md) for details.  The project is provided "as is" without any guarantees of fitness for purpose or reliability.

However, if you find a bug or issue, please raise an issue via GitHub.
