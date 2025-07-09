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
Core Firmware Properties
------------------------
File type:     Binary (.bin)
File size:     82,524 bytes (81KB)
Version:       0.1.1 (build 1)
Build Date:    Jul  9 2025 18:50:57
Git commit:    171a135
Hardware:      24 pin rev F
STM32:         F411RE (512KB flash, 128KB RAM)
Frequency:     100 MHz (Overclocking: false)

Configurable Options
--------------------
Serve image from: RAM
SWD enabled:      true
Boot logging:     true
Status LED:       false
STM bootloader:   false
MCO enabled:      false

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
    Name:      kernal.901486-07.bin
    CS States: Active Low/Not Used/Not Used
    CS Lines:  ROM Pin 20/ROM Pin 18/ROM Pin 21
-----------
ROM Set: 1
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2364
    Name:      basic.901486-01.bin
    CS States: Active Low/Not Used/Not Used
    CS Lines:  ROM Pin 20/ROM Pin 18/ROM Pin 21
-----------
ROM Set: 2
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2332
    Name:      characters.901460-03.bin
    CS States: Active Low/Active Low/Not Used
    CS Lines:  ROM Pin 20/ROM Pin 21/None
-----------
ROM Set: 3
  Size: 16384 bytes
  ROM Count:     1
  Algorithm:     A = Two CS checks for every address check
  Multi-ROM CS1: Not Used
  ROM: 0
    Type:      2364
    Name:      dead-test.pal.e0
    CS States: Active Low/Not Used/Not Used
    CS Lines:  ROM Pin 20/ROM Pin 18/ROM Pin 21
```
