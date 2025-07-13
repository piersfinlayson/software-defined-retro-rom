# Software Defined Retro ROM (SDRR) Firmware Info

This tool parses eithe a .bin or .elf SDRR firmware file, from v0.1.1 onwards, and outputs:

- Key firmware properties
- Configuration options
- Information about included ROM images
- Bytes of specific stored ROM images, to allow extraction of ROM images from the firmware.

See [`main.rs`](src/main.rs) for a description of its operation.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release -- ../sdrr/build/sdrr-yourtarget.elf
```

There are a number of commands:

- `info` - Display key firmware properties, configuration options, and ROM information - chosen automatically if no command is specified.
- `lookup` - Look up one of more bytes from a ROM image by its set and address or range.
- `lookup-raw` - Look up one or more bytes from a ROM image by its set and address **as read in by the STM32 on its address/CS port**.  Likely to be useful for debugging and developers only.
- `help <command>` - More details on the commands and options available.

Some further notes:

- All commands accept the `-d|--detail` flag to provide more detailed output.
- `lookup` and `lookup-raw` accept the `--output-mangled` flag to output the resulting byte(s) as the mangled byte that the STM32 would write to the data port.  Likely to be useful for debugging and developers only.
- `lookup` can be used with `--output-binary` to output the result as a binary file, which is useful for extracting ROMs from the firmware, for checksumming and/or comparing with the originals.

## Sample Output

The following is sample output from the following command:

```bash
cargo run --release -- info -d ../sdrr/build/sdrr-yourtarget.elf
```

```text
==========================================
Running sdrr-info, to:
- Validate SDRR firmware
- Extract key SDRR firmware properties
- Show detailed SDRR firmware properties
-----
Software Defined Retro ROM - Firmware Information
-------------------------------------------------

Core Firmware Properties
------------------------
File name:     sdrr-stm32f411re.bin
File modified: Jul 13 2025 13:43:48
File type:     Binary (.bin)
File size:     82,944 bytes (81KB)
Version:       0.2.0 (build 1)
Build Date:    Jul 13 2025 13:43:46
Git commit:    5d372a7
Hardware:      24-d
STM32:         F411RE (512KB flash, 128KB RAM)
Frequency:     100 MHz (Overclocking: false)

Configurable Options
--------------------
ROM emulation:    24 pin ROM
Serve image from: RAM
SWD enabled:      true
Boot logging:     true
Status LED:       false
STM bootloader:   false
MCO enabled:      false
Boot config:      0xFFFFFFFF - Reserved, should be 0xFFFFFFFF

Pin Configuration
-----------------

Data pin mapping:
  D0:  PA7
  D1:  PA6
  D2:  PA5
  D3:  PA4
  D4:  PA3
  D5:  PA2
  D6:  PA1
  D7:  PA0

Address pin mapping:
  A0:  PC5
  A1:  PC4
  A2:  PC6
  A3:  PC7
  A4:  PC3
  A5:  PC2
  A6:  PC1
  A7:  PC0
  A8:  PC8
  A9:  PC13
  A10: PC11
  A11: PC12
  A12: PC9

Chip select pins:
  2364 CS1: PC10
  2332 CS1: PC10
  2332 CS2: PC9
  2316 CS1: PC10
  2316 CS2: PC12
  2316 CS3: PC9

Image select pins:
  SEL0: PB0
  SEL1: PB1
  SEL2: PB2

Status LED pin:
  Pin: None

ROMs Summary:
-------------
Total sets: 4
Total ROMs: 4

ROM Details: 4
--------------
ROM Set: 0
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2364
    Name:      kernal.901227-03.bin
    CS States: Active Low/Not Used/Not Used
-----------
ROM Set: 1
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2364
    Name:      basic.901226-01.bin
    CS States: Active Low/Not Used/Not Used
-----------
ROM Set: 2
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2332
    Name:      characters.901225-01.bin
    CS States: Active Low/Active High/Not Used
-----------
ROM Set: 3
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2364
    Name:      dead%20test.BIN
    CS States: Active Low/Not Used/Not Used
==========================================
```
