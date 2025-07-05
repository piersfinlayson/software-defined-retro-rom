# 24 Pin Rev D

23xx (24 pin) ROM replacement SDRR PCB.

This revision has been verified to support

- the STM32F401 and STM32F411 variants
- acting as 2364 and 2332 ROM replacements in the Commodore PET, VIC-20 and C64 (50Hz/PAL).

Note that the gerbers are currently untested - the design has been verified via [OSH Park](#ordering-bare-pcbs).

## Contents

- [Schematic](sdrr-rev-d-schematic.pdf)
- [Fab Notes](sdrr-rev-d-fab-notes.pdf)
- [Gerbers](gerbers/)
- [Ordering Bare PCBs](#ordering-bare-pcbs)
- [Errata](#errata)
- [Notes](#notes)
- [Changelog](#changelog)
- [BOM](#bom)

## Ordering Bare PCBs

[![Order from OSH Park](https://oshpark.com/assets/badge-5b7ec47045b78aef6eb9d83b3bac6b1920de805e9a0c227658eac6e19a045b9c.png)](https://oshpark.com/shared_projects/E7lLKM9E)

Alternatively, you can order from your preferred PCB manufacturer using the Gerber files provided in the [`gerbers/`](gerbers/) directory.

## Errata

All corrections have been reflected in the BOM below, but not the schematic or gerbers.

- **C9** should be 4.7uF, or the STM32F411 will lock up at 100MHz.
- **U3** recommend using AP2112K-3.3 in place of the RT9013, to reduce switching noise, which can cause failure to boot on some systems.
- **R4, R5, R6** do not need to be populated - the firmware uses internal pull-downs.

## Notes

- **Does not** support the STM32F405 or STM32F415 variants.
- There is no status LED on this revision, although there is a power LED.
- There are only 3 image select jumpers on this revision, so only up to 8 images can be selected.
- Most passives are 0402.

## Changelog

n/a

## BOM

| Reference | Component Type | Value | Package | Quantity | Notes |
|-----------|----------------|-------|---------|----------|-------|
| C1, C2 | Capacitor | 1uF | 0402 | 2 | |
| C3, C9 | Capacitor | 4.7uF | 0603 | 2 | |
| C4, C5, C6, C7, C10 | Capacitor | 100nF | 0402 | 5 | |
| C8 | Capacitor | DNP | 0603 | 1 | Do not populate |
| D1 | LED | - | 0402 | 1 | Colour as desired |
| J1 | Pin header | 1x5 | - | 1 | 2.54mm (0.1") pin pitch |
| J3 | Pin header | 2x4 | - | 1 | 2.54mm (0.1") pin pitch |
| R1, R2 | Resistor | 4k7 | 0402 | 2 | |
| R3 | Resistor | 470R | 0402 | 1 | Modify value to suit chosen LED |
| R4, R5, R6 | Resistor | DNP | 0402 | 3 | Do not populate |
| R7, R9 | Resistor | 0R | 0402 | 2 | |
| R8 | Resistor | DNP | 0402 | 1 | Do not populate |
| TP1 | Test point (DNP) | - | - | 1 | |
| U1 | Microcontroller | STM32F401RxT/STM32F411RxT | - | 1 | |
| U2 | Pin header | 1x12 | - | 2 | 2.54mm (0.1") pin pitch |
| U3 | Voltage regulator | AP2112K-3.3 | SOT-23-5 | 1 | |
