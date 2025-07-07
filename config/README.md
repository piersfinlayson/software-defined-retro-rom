# Config

Contains configuration files for various sample ROM collections.

## Usage

Use these as follows to build and flash the SDRR firmware with your chosen collection of ROM images and configuration:

```bash
STM=f411re CONFIG=config/1541.mk make run
```

Replace `f411re` with your [target STM32 variant](/README.md#supported-stm32-microcontrollers) and `1541.mk` with the desired [configuration file](#available-configurations).

## Available Configurations

### Single image ROM sets

Each of the configurations below contains a set of single ROM images, which can be selected using SDRR's image select jumpers.  When using these configurations, SDRR emulates a single ROM chip, for the socket it is inserted in.

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

### Multi-image ROM sets

These configurations contain multiple ROM image sets, each set of which can be selected using SDRR's image select jumpers.  When using these configurations, SDRR emulates a multi-ROM chip, for the socket it is inserted in, plus any other (empty) sockets:

- which shares both address and data bus of the socket SDRR is installed in
- whose chip select lines (pin 20) are connected to SDRR pins X1 (the second image of the set) and optionally X2 (third image of the set).

Some of the multi-image ROM sets have different numbers of images in each set - for example:

- Set 0 of [`set-c64.mk`](set-c64.mk) contains 3 images - the kernal ROM, the character ROM and the basic ROM
- Set 1 of [`set-c64.mk`](set-c64.mk) contains 2 images - the kernal ROM and the character ROM - as the dead test kernal does not use/require a basic ROM.

This allows you to use the image select jumper to select set 0, and if that doesn't boot, to select set 1, to give you additional diagnostics.

| Configuration | Description |
|---------------|-------------|
| [`set-c64.mk`](set-c64.mk) | C64 kernal/basic/char ROM set, and diag/char ROM set |
| [`set-vic20-pal.mk`](set-vic20-pal.mk) | VIC-20 PAL kernal/basic ROM set, and diagnostics ROM image |
