# MCU Selection

Your most important choice is between a Raspbery Pi RP2350-based One ROM, or an STM32F4-based One ROM.  Both have roughly equivalent support in the firmware.

| Feature | RP2350 | STM32F4 |
|---------|--------|---------|
| **Firmware** | Support just added but already mostly complete | More mature firmware |
| **Price Each (5 units)** | $9 | $6.50 |
| **Price Each (20 units)** | $4.50 | $3.25 |
| **Performance** | Roughly per STM32, shows more overclocking headroom | Roughly per RP2350, but has slight edge |
| **Image Select Jumpers** | 3 | 5 |
| **Bank/Set Pins** | 2 | 2 |
| **Flash Size** | 2MB | 128KB-1MB |
| **Physical Size** | 2316/2332/2364 footprint | Same |
| **Future Enhancements** | May utilize dual cores for additional features in future updates | Expected to be supported for foreseeable future |

Notes:
- **Price**: Approximate Assembled price, each, from JLCPCB, August 2025, not including tax and shipping.
- **Image Select Jumpers**: More jumpers allow more images to be selected between at boot time
- **Bank/Set Pins**: Allow dynamic switching between images at runtime, or serving multiple ROM sockets at once
- **Storage Impact**: Flash size impacts ROM image storage capacity and multi-ROM set usage
- **Support**: Both boards are expected to be supported long-term
- **Price Trend**: RP2350 is more expensive in small quantities but prices converge at larger volumes

## Which One Should **You** Choose?

If you don't have strong reasons to go with the STM32, go with the RP2350.  Get a few and expect you might fry one one day with a dodgy voltage regular/power supply.

One ROM's support is already essentially equivalent and will only get better.

## RP2350

The One ROM Pico uses the RP2350 MCU, A4 stepping or later.  This is now (August 2025) used by default by JLC's PCB assembly service.

See [RP2350](docs/RP2350.md) for more details about the RP2350 support, including some earlier performance results.

## STM32

The One ROM STM uses the STM32F4 family of MCUs.  This section aims to help you choose the right one for your needs.

All performance figures in this document are based on testing genuine STM32 parts.  For a discussion of clones, see [STM32-Clones](/docs/STM32-CLONES.md).

If you'd like an unsupported STM32 variant supported, please raise an issue via GitHub or [add it yourself](docs/STM32-SELECTION.md#supporting-other-variants) and submit a PR.

### Supported Microcontrollers

The following table provides details about the supported STM32 and clone microcontrollers:

| Model | Flash | RAM | Max Clock Speed | Max single ROM Images | Max mult-ROM sets | Build variant |
|-------|-------|-----|-----------------|-----------------------|-------------|---------------|
| STM32F401RBT6 | 128KB | 64KB | 84MHz | 6 | - | f401rb |
| STM32F401RCT6 | 256KB | 64KB | 84MHz | 14 | - | f401rc |
| STM32F401RET6 | 512KB | 96KB | 84MHz | 16 | 7 | f401re |
| STM32F411RCT6 | 256KB | 128KB | 100MHz | 14 | 3 | f411rc |
| STM32F411RET6 | 512KB | 128KB | 100MHz | 16 | 7 | f411re |
| GD32F405RGT6* | 1024KB | 128KB | 168MHz | 16 | 15 | f405rg |
| STM32F405RGT6 | 1024KB | 128KB | 168MHz | 16 | 15 | f405rg |
| STM32F446RCT6 | 256KB | 128KB | 180MHz | 16 | 3 | f446rc |
| STM32F446RETx | 512KB | 128KB | 180MHz | 16 | 7 | f446re |

(*) The GD32F405RGT6 is currently preferred above the STM32F405RGT6 as it seems more highly performant.  See [this issue](https://github.com/piersfinlayson/software-defined-retro-rom/issues/8) for more details.

Neither STM32F401RBT6 and STM32F401RCT6 have sufficient RAM to serve multi-ROM or banked sets of images, but they can serve single ROM images.

The vast majority of ROMs can be emulated by the cheapest variant, the F401.  The best bang for buck are likely to be either:

- STM32F401RET6 - [currently $1.93 on LCSC](https://lcsc.com/product-detail/Microcontrollers-MCU-MPU-SOC_ST-STM32F401RET6_C116978.html)
- STM32F411RET6 - [currently $2.69 on LCSC](https://lcsc.com/product-detail/Microcontrollers-MCU-MPU-SOC_ST-STM32F411RET6_C94355.html)

The F401 may require a small overclock for some systems, such as running the C64 multi-ROM set (~90MHz vs 84MHz rated maximum).

There are also cheap [clones available](/docs/STM32-CLONES.md), with a clone F405 coming in at around the same price as a genuine F411 (from LCSC).  The GD32F405RGT6 has been tested, and is a recommended MCU.

### Clock Speed Requirements

Ideally, you should not specify the `FREQ` build variable, which will cause the STM32 variant you choose to run at its maximum clock speed.

However, if you need to specify a lower clock speed, for example, to reduce power draw/heat you can do so by setting the `FREQ` build variable to a value that is less than or equal to the maximum clock speed of the selected STM32 variant.  See [`FREQ`](/docs/CONFIGURATION.md#freq) for more details.  It is also possible to [overclock](/docs/CONFIGURATION.md#overclock) the processor you have, at your own risk.

Different retro systems, and different use cases, have different minimum clock speed requirements.  Some example minimum clock speeds are given in the following table.  They contain both serving algorithms `A` and `B`, which can be [specified](/docs/CONFIGURATION.md#serve_alg), and `B` is the default.

They were generated by testing against real retro systems and are absolute minima.  Your mileage may vary and it would be advisable to add some headroom to these figures - and you may need to experiment with different clock speeds to find the best one for your system.

Alternatively, either a GD32F405, STM32F405 or STM32F446 and running at maximum clock speed, both of which support all of the systems below, or [overclock](/docs/CONFIGURATION.md#overclock) the processor you have.  Even the C64 multi-ROM set requires only a modest (~5%) overclock on the F401.

| System | Video | ROM Type | Default Alg `B` | Optional Alg `A` | STM32F4 Variants | Release Measured | Notes |
|--------|-------|----------|-------|-------|------------------------|------------------|-------|
| PET 4032 | 40 Col 50Hz | Kernal | 26MHz | 29MHz | All | v0.2.1 | $E000 tested |
| PET 4032 | 40 Col 50Hz | Character | 18MHz | 26MHz | All | v0.2.1 | |
| VIC 20 | PAL | Kernal | 32Mhz | 34MHz | All | v0.2.1 | |
| VIC 20 | PAL | BASIC | 32MHz | 34MHz | All | v0.2.1 | |
| VIC 20 | PAL | Character | 45MHz | 76MHz | All | v0.2.1 | |
| C64 | PAL | Kernal | 79MHz | 79MHz | All | v0.2.1 | |
| C64 | PAL | Character | 79MHz | 92MHz | All | v0.2.1 | |
| 1541 | n/a | Kernal | 18MHz | 30MHz | All | v0.2.1 | $C000 tested |
| VIC 20 | PAL | Multi-ROM Set | 32MHz | n/a | All | v0.2.1 | Kernal+BASIC |
| C64 | PAL | Multi-ROM Set | 89MHz | n/a | *F411/F405/F446 | v0.2.1 | Kernal+BASIC+Char |
| 1541 | n/a | Multi-ROM Set | 18MHz | n/a | All | v0.2.1 | $C000+$E000 |

(*) Or a slightly overclocked F401.

When running multiple One ROMs in a single system, you may find you have to run each One ROM faster than these speeds, to avoid each conflicting with the other.  In this case, consider using multi-ROM support.

When all CS lines on a ROM are active low, the serving algorithm is slightly quicker, and hence slower clock speeds are acceptable.

### ROM Image Capacity

Each ROM image consumes 16KB of flash (irrespective of the ROM type - see [Technical Details](/docs/TECHNICAL-DETAILS.md) for why), so the maximum number of ROM images that can be stored is limited by the flash size for smaller variants.  The One ROM firmware itself consumes between 16-32KB of flash.  The rest of the flash can be used for images, up to a maximum of 16, which is limited by the number of select jumpers on the PCB, which in turn is limited by the phyical size of the original ROM footprint.

If you are using multi-ROM sets (of up to 3 images), each set consumes 64KB of flash, again irrespective of the image types.

### Supporting Other Variants

If you have a different variant, the easiest way to support it is to choose a version above which is of the same or higher power than your chosen variant, and select that version.

If you want to add support for another variant:

- Add it to [`Makefile`](/Makefile) - see the `STM` variable.
- Add it to [`/rust/sddr-common`](/rust/sdrr-common), in particular:
  - [`/rust/sdrr-common/src/sdrr_types.rs`](/rust/sdrr-common/src/sdrr_types.rs):
    - `StmProcessor`
    - `StmVariant`
  - [`/rust/sdrr-common/src/args.rs`](/rust/sdrr-common/src/args.rs):
    - `parse_stm_variant()` function
  - [`/rust/sdrr-gen/src/rom_types.rs`](/rust/sdrr-gen/src/rom_types.rs):
    - `StmProcessor`
    - `StmVariant`
- Add it to the [`sdrr`] firmware:
  - [`/sdrr/src/config_base.h`](/sdrr/src/config_base.h)
    - `stm_line_t`
  - Add its max frequency to [`/sdrr/include/include.h`](/sdrr/include/include.h) - search for the pre-processor `TARGET_FREQ_MHZ` tests.
  - Add it to [`/sdrr/src/utils.c`](/sdrr/src/utils.c) - `log_init` function.
  - Add any support specifically for this STM32 variant - grep for "STM32F411" and "STM32F405" for examples of variant specific code.

### STM32F4 Family Comparisons

For completeness, all of the STM32F4xxR variants are compared in the following table.  All of these variants proivde the Cortex-M4F core.

| Feature | F401 | F405 | F410 | F411 | F412 | F413 | F415 | F423 | F446 |
|---------|------|------|------|------|------|------|------|------|------|
| **Max Freq** | 84 MHz | 168 MHz | 100 MHz | 100 MHz | 100 MHz | 100 MHz | 168 MHz | 100 MHz | 180 MHz |
| **Flash** | 128-512 KB | 512 KB-1 MB | 64-128 KB | 256-512 KB | 512 KB-1 MB | 1-1.5 MB | 512 KB-1 MB | 1-1.5 MB | 256-512 KB |
| **SRAM** | 96 KB | 192 KB | 32 KB | 128 KB | 256 KB | 320 KB | 192 KB | 320 KB | 128 KB |
