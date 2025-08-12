# sdrr-check

`sdrr-check` is a utility for testing the SDRR (Software Defined ROM Replacement) hardware post manufacturing.

See [Manufacturing Test](/docs/MANUFACTURING-TEST.md) for a detailed guide on how to use this utility during the manufacturing process.

## Supported Hardware

Only specific hardware revisions are currently supported by this tool:

- SDRR hardware revision 24-f

## Features

- **Jumper Testing**: Verify the correct  of image/set and X1/X2 jumpers
- **LED Testing**: Check the operation of the onboard LED

## Dependencies

```bash
rustup target add thumbv7em-none-eabi
```

## Building

**From within the sdrr-check directory**

```bash
cargo build
```

## Usage

To use `sdrr-check`, flash it to the SDRR device and connect to the logging interface. Then:

- check the on-board LED is flashing (at a frequency of 1Hz)
- check the jumpers work, by opening and closing them and checking the logs.

The NRST pin can be verified by ensuring the LED "freezes" when the NRST jumper is closed, and restarts flashing when the jumper is opened.

## Binary

To get a binary file, in addition to the ELF file which cargo produces:

```bash
cargo install bin-utils
rust-objcopy -O binary ../target/thumbv7em-none-eabi/release/sdrr-check /tmp/sdrrcheck.bin
```

