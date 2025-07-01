# Config

Contains configuration files for various sample ROM collections.

## Usage

Use these as follows to build and flash the SDRR firmware with your chosen collection of ROM images and configuration:

```bash
STM=f411re CONFIG=config/1541.mk make run
```

Replace `f411re` with your [target STM32 variant](/README.md#supported-stm32-microcontrollers) and `1541.mk` with the desired [configuration file](#available-configurations).

## Available Configurations

| Configuration | Description |
|---------------|-------------|
| [`c64.mk`](c64.mk) | C64 stock and diags ROMs |
| [`vic20-pal.mk`](vic20-pal.mk) | VIC-20 PAL version stock and diag ROMs |
| [`pet-4-40-50.mk`](pet-4-40-50.mk) | PET BASIC 4, 40 column, 50Hz stock and diag ROMs |
| [`1541.mk`](1541.mk) | 1540 and 1541 disk drive stock ROMs |
| [`2040.mk`](2040.mk) | 2040/3040 DOS1 disk drive stock ROMs |
| [`4040.mk`](4040.mk) | 4040 (and 2040/3040) DOS2 disk drive stock ROMs |
| [`ieee-diag.mk`](ieee-diag.mk) | 2040/3040/4040/8050/8250 IEEE-488 disk drive diagnostics ROMs |
| [`test-sdrr-0.mk`](test-sdrr-0.mk) | SDRR test ROMs (for testing SDRR functionality) |
