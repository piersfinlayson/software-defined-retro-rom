# STM32F4 24 Pin Rev F

**Unverified** - this revision has not yet been formally tested or confirmed to work.  Order PCBs at your own risk.

One user has confirmed that this board worked with an STM32F401RB, as a replacement VIC-20 NTSC kernal ROM.

23xx (STM32F4 24 pin) ROM replacement SDRR PCB.

This revision has the following key improvements over rev E:

- 2 x additional pins are broken out (labelled 'X1' and 'X2' on the underside), to allow a single SDRR to replace up to 3 2364 ROMs simultaneously, by wiring the chip select lines from the additional ROMs directly to these new pins.  

<div style="text-align: center;">
  <a href="/sdrr-pcb/unverified/stm32f4-24-pin-rev-f/sdrr-24-pin-rev-f-side.png">
    <img src="sdrr-24-pin-rev-f-side.png" alt="SDRR STM32F4 24 pin rev F" width="400">
  </a>
</div>

## Contents

- [Schematic](sdrr-24-pin-rev-f-schematic.pdf)
- [Fab Notes](sdrr-24-pin-rev-f-fab-notes.pdf)
- [Gerbers](gerbers/) - unverified
- [Ordering Bare PCBs](#ordering-bare-pcbs)
- [Errata](#errata)
- [Notes](#notes)
- [Changelog](#changelog)
- [BOM](#bom)

## Ordering Bare PCBs

[![Order from OSH Park](https://oshpark.com/assets/badge-5b7ec47045b78aef6eb9d83b3bac6b1920de805e9a0c227658eac6e19a045b9c.png)](https://oshpark.com/shared_projects/FXTBxRfK)

## Errata

None

## Notes

None

## Changelog

- Changed RT9013 to AP2112K-3.3
- Added pin 1 silk screen marking
- PC14/PC15 broken out as X1/X2
- Move NRST R to close to MCO pad

## BOM

See notes for STM32F405/F415 BOM differences.

| Reference | Component Type | Value | Package | Quantity | Notes |
|-----------|----------------|-------|---------|----------|-------|
| C1, C2 | Capacitor | 1uF | 0603 | 2 | |
| C3 | Capacitor | 4.7uF| 0603 | 1 | |
| C4, C5, C6, C7, C8 | Capacitor | 100nF | 0603 | 5 | |
| C9 | Capacitor | 4.7uF| 0603 | 1 | **F405/F411 - DNP** |
| D1 | LED | - | 0603 | 1 | Optional, colour as desired |
| J1 | Pin header | 1x5 | - | 1 | 2.54mm (0.1") pin pitch |
| J2 | Pin header | 2x4 | - | 1 | 2.54mm (0.1") pin pitch |
| J3 | Pin header | 1x2 | - | 1 | 2.54mm (0.1") pin pitch |
| R1, R2 | Resistor | 4k7 | 0603 | 2 | |
| R3 | Resistor | 470R | 0603 | 1 | Optional, modify value to suit chosen LED |
| RC4, RC5 | Resistor | 0R | 0603 | 2 | **F405/F411 - use 2.2uF 0603 capacitor** |
| TP1 | Test point (DNP) | - | - | 1 | Exposes MCO1, configurable via software |
| U1 | Microcontroller | STM32F401RxT STM32F411RxT STM32F405RxT STM32F411RxT | - | 1 | |
| U2 | Pin header | 1x12 | - | 2 | 2.54mm (0.1") pin pitch |
| U3 | Voltage regulator | AP2112K-3.3 | SOT-23-5 | 1 | |
