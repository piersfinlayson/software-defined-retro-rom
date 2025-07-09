# Software Defined Retro ROM (SDRR) Firmware Info

This tool parses eithe a .bin or .elf SDRR firmware file, from v0.1.1 onwards, and outputs:

- Key firmware properties
- Configuration options
- Information about included ROM images

See [`main.rs`](src/main.rs) for a description of its operation.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release -- ../sdrr/build/sdrr-yourtarget.elf
```

## Sample Output

```text
Software Defined Retro ROM - Firmware Information
=================================================

Core Firmware Properties
------------------------
File type:     Binary (.bin)
Version:       0.1.1 (build 1)
Build Date:    Jul  9 2025 14:14:23
Git commit:    73987d0
Hardware:      24 pin rev F
STM32:         F411RE (512KB flash, 128KB RAM)
Frequency:     100 MHz (Overclocking: false)

Configurable Options
--------------------
Serve image from: RAM
SWD enabled:      true
Boot logging:     true
Status LED:       true
STM bootloader:   true (close all image select jumpers to activate)
MCO enabled:      true (exposed via test pad)

ROM Sets: 4
-----------
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
