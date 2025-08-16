# Technical Summary

This document summarises how One ROM provides the ROM emulation function, and hits the required timings.

If you'd rather consume similar content in video form:

[![How It's Done - Software ROMs](https://img.youtube.com/vi/pOZ2-W3dpZ8/maxresdefault.jpg)](https://youtu.be/pOZ2-W3dpZ8)

## Overview

One ROM replaces mask-programmed ROMs (2364/8KB, 2332/4KB, 2316/2KB) in retro systems using an STM32F4 microcontroller.  Unlike traditional ROM replacements that require hardware modifications or expensive programming equipment, One ROM provides a software-configurable solution that can store multiple ROM images of different types and switch between them via jumpers.  It can also be reprogrammed in situ, without needing to remove it from the system.

## Timing Requirements

23 series mask ROMs in retro systems must respond to address changes within tight timing windows. For example:

- **PET systems**: ROM access time 450ns (2MHz bus cycles)
- **VIC-20/C64**: ROM access time 450ns (faster video chip demands)
- **C64 character ROM**: Access time ~350ns (VIC-II requirements)

A key challenge is that these access times reference time to go from address line change to the data being accurate.  Chip select activation - essentially the time to get the data lines to outputs with the correct data - typically has 200ns access times.

## Solution Overview

### Hardware

One ROM uses STM32F4 Cortex-M4 based microcontrollers because they provide:

- **5V-tolerant GPIO pins**: Compatible with retro system logic levels without level shifters
- **Optimal pin arrangement**: Sufficient contiguous 5V-tolerant pins for efficient assembly code
- **High clock speeds**: 84MHz (F401) to 180MHz (F446) - to meet the timing requirements
- **Efficient instruction prefetch/cache**: Improves performance by eliminating flash wait states
- **Internal oscillator and flash memory**: Minimises components required and reduces PCB size and manufacturing cost

### The Main Loop Theory

The core of One ROM is an assembly main loop that continuously monitors the address and chip select lines, then responds by driving the data lines.  Interrupt latency on the Cortex-M4 is a worst case of 12 cycles (120ns at 100MHz- more than half of the chip select access time budget) which means that a polling approach is taken to hit the required timing.

Prior to entering this main loop, the code does various setup which keeps the loop as simple and fast as possible:

1. **Pre-load ROM data into RAM**: Copies the selected ROM image from flash to RAM for faster access.
2. **Calculate chip select masks**: Generates masks for chip select lines based on type and configuration of ROM to be served.
3. **Store values in registers**: Pre-loads all data which is used by the main loop into registers to avoid moving data during the loop.

The assembly code is designed so that no registers need to be reused for any other purpose, hence we avoid unnecessary movs.  The code prefers the low registers (r0-r7), as instructions using these are tyically encoded in 16-bits vs 32-bits meaning they take less space in flash, and hence can be fetched faster.  Given the STM32F4's flash prefetch and instruction cache, this is probably unnecessary.

The assembly code operates as follows:

#### Main loop - CS inactive

Access times for the 23 series ROMs are asymmetric (chip select access time is around 200ns vs 350/450ns for the address lines), so the main loop tests for chip select active roughly twice as often as it loads data from the address lines.  This means the loop responds faster to the chip select active/deactive state changes than to address line changes - as required.

One ROM takes advantage of this by interleaving the code to read the data byte from RAM, and store it to the data lines (two instructions in total) with the additional chip select state checks - this allows us to avoid ARM load use penalties (we are essentially using this penalty for useful work instead).

Main loop:

1. Read address lines + chip select state (one GPIO port read for both address and chip select)
2. Test for chip select state using the GPIO value just retrieved and pre-computed chip select mask - if active, move to chip select active loop
3. Loads the ROM data byte using the GPIO value just retrieved as an offset into the pre-loaded ROM data, now in RAM
4. Read address lines + chip select state
5. Store the data byte to the data lines (one GPIO port write) - note the data lines are currently inputs, so not transmitting the value onto the bus
6. Test for chip select again - if active, move to chip select active loop
7. Otherwise loop back to step 1

#### Chip Select active loop

It is important to note that when chip select is active, One ROM must still query the address lines, as well as the chip select state.  Again, the timings are asymmetric, so the loop queries the chip select(s) roughly twice as often as the address lines.  As before the code is interleaved to read the data byte and store it with the chip select state checks, avoiding some load use penalties.

By definition, when entering this main loop, the "calling" code just read the address lines, as well as the chip select state, so it has the address lines sitting in a register.  These can then be used immedaitely to read the ROM data byte, without having to re-read the address lines.

1. Set the data lines to output mode
2. Load the data byte from RAM using the address value already in hand
3. Load the address lines + chip select state
4. Test chip select state - if inactive, move to chip select inactive _with a data byte in hand_ code
5. Store the data byte to the data lines - note they are currently outputs, so changes the value transmitted onto the bus
6. Load the address lines + chip select state again
7. Test chip select state - if inactive, move to chip select inactive _without a data byte in hand_ code
8. Load the data byte (same as step 2)
9. Load the address lines + chip select state again
10. Test chip select state
11. If chip select is still active, loop back to step 2
12. Otherwise fall through to the chip select inactive _with a data byte in hand_ code (avoids branch penalty)

#### Chip select inactive - with a data byte in hand

The ROM must continue transmitting valid data for a minimum time after chip select is deasserted.  This is typically 40ns.  One ROM currently doesn't implement this explicitly, and instead relies on the fact that getting to the code to set the data lines as inputs will take sufficient time.

Should implementing this be required, one solution would be to have different chunks of chip select inactive code to branch to, depending on how quickly after loading the chip select state the code spotted it was now inactive, and pad the code with instructions that the Cortex-M4 cannot optimise out (like it can with NOPs).  This would need to be modified for different clock speeds.

Chip select inactive with a data byte in hand code:

1. Set the data lines to input mode
2. Store the data byte to the data lines - note they are now inputs, so not transmitting the value onto the bus
3. Fall through to the main loop to start polling for chip select active again

#### Chip select inactive - without a data byte in hand

1. Set the data lines to input mode
2. Fall into _another_ copy of the main loop to start polling for chip select active again.  Another copy is used here to avoid a branch penalty.
3. At the end of this copy of the main loop, branch to the beginning of the original main loop.  This branch is unavoidable - see [unrolling the loops](#unrolling-the-loops).

#### Unrolling the loops

While it would be possible to "unroll" the assembly loops ("copy and paste" the code many times end-to-end to avoid the branch penalty in many cases), at some point the code needs to branch back to the beginning.  This makes just one time "slower" - when it has to branch back.

However, we're not interested in making 99% of the cases hit the timing window - we need 100%, as otherwise the CPU will crash when it hits that 1% window and gets garbage.

It's also worth bearing in mind that the STM32F4 has a finite instruction cache (1024 bytes), so unrolling the loops too much could lead to cache misses and reduced performance.

### ROM Data "Mangling"

Due to PCB size and trace routing constraints, the address and data lines of the STM32F4 are not mapped to the equivalent ROM address and data lines.  For example, in hardware revision E, STM port C line 0 maps to A7, PC1 maps to A6, etc, and PA0 maps to D7, PA1 maps to D6, etc.

One ROM pre-processes the ROM images during the build process to match the exact GPIO pin mapping.  Therefore _PC0_ is actually used as address line 0 looking up in the ROM image, but the data bytes in the ROM have been reordered so the correct byte is retrieved.  The bits within the data byte are also reordered, so that the real bit 7 is actually at the bit 0 location, etc.

This is all done during the build process to avoid either the startup code, or, worse, the main loop, from having to do any bit manipulation at runtime - keeping everything as fast as possible.

### 16KB Storage Allocation

All ROM images are stored as 16KB blocks in flash, regardless of actual size.  This is because, in additional to the address lines being re-mapped, the chip select lines are also embedded within the address line port on the STM32F4.

As an example, in hardware revision E, the 2364 chip select line is PC10, with A12 and A10 on either side.

This means that 14 bits (A0-12, plus one CS line) = 16KB of address space needs to be looked up to find the correct ROM image.  It also means that, as we look up the data byte using the output of the address port, the chip select line(s) can either be 0 _or_ 1 when we read the address lines.  This means that we have to store duplicates of the ROM images - 2x the images for a 2364, 4x for a 2332, and 8x for a 2316.  Hence the need for 16KB blocks.

Again, the pre-processing into 16KB blocks is done during the build process.

### Configuration Flexibility

Traditional ROMs had mask-programmed chip select behavior (active high or low). One ROM can emulate any combination - for example:

```bash
# 2364 ROM, CS1 active low
ROM_CONFIGS=file=kernal.rom,type=2364,cs1=0

# 2332 ROM, CS1 active low, CS2 active high  
ROM_CONFIGS=file=char.rom,type=2332,cs1=0,cs2=1

# 2316 ROM, CS1 active low, CS2 active high, CS3 active low
ROM_CONFIGS=file=other.rom,type=2316,cs1=0,cs2=1,cs3=0
```

The firmware currently generates appropriate chip select masks just before entering the main loop.  This could be optimised to be done at build time, but this would probably only save 10s of nanoseconds, and is only done once, so has not been deemed worth the complexity.

### Multi-Image Storage

Up to 16 different ROM configurations can be stored in a single firmware build depending on the flash size of the STM32F4 variant used.  This limit isn't enforced by the software, but rather by 4x image select jumpers.

Images are selected via the jumpers binary encoding 0-15 which are read at startup.  When emulating ROMs with fast access times, on the slower STM32F4 variants, there is no time in the main loop to do any additional processing, so the jumpers must be read at startup.  It _may_ be possible when emulatig slow ROMs, with the fastest STM32F4 variants, to do additional processing the main loop, like testing the jumpers, but this has not been validated.

## Multi-ROM Support

One ROM implements multi-ROM support, allowing replacement of up to 3 ROMs simultaneously, with a single One ROM.  This is done by installing One ROM in one of the ROM sockets, and depopulating the other ROM sockets.  A flying lead is then used to connect the chip select line (pin 20) of the empty ROM sockets to pins X1 and X2 on the One ROM board.

A ROM set configuration must be used, to program One ROM with the combined ROM images - see [`config/set*.mk`](/config/README.md#multi-image-rom-sets).

This feature works by

- combining all of the ROM images to be served into a single 64KB image, and then using X1/X2 (HW revision F onwards) as additional address lines into the image
- a different data serving algorithm, which only serves data when any one of the chip select lines (CS1, X1, X2) is active.
- using the entire 16-bit input value - including X1 and X2 state - from the STM32F4's address/CS port as index into the 64KB image.

The serving algorithm is slightly less performant than the single default ROM serving algorithm, as it cannot load address bytes and write then to the data port (in input state) before CS lines go active - as without the CS line active, it doesn't know which ROM image to serve from.  It is difficuly to give a precise figure for the perfomance hit as it seems to differ by systems/ROMs replaced, but 10% is a decent rule of thumb.  See [Multi-ROM performance](#multi-rom-performance) for more details.

There are currently various limitations in this support.  For example:

- All replaced ROMs must share both address and data buses.  The VIC-20 character ROM does not share the address bus with the Kernal and BASIC ROMs, so cannot be replaced by a multi-ROM set, but the C64 character ROM does share a bus, so all of Kernal, BASIC and character ROMs can be replaced by a single multi-ROM set.
- Mixed ROM type support is only possible when the extra chip select lines of the smaller ROMs are tied to always active.  Again, the C64 character ROM is a good example - it is a 2332 ROM, unlike the Kernal and BASIC ROMs which are 2364 ROMs.  However, the 2332's second cs line is always tied active.
- Only 2364, and 2364/2332 ROM types have been tested.
- While single ROM and multi-ROM sets can be used in the same One ROM firmware image, you have to disconnect X1/X2 when using a single ROM set.  (And X2 must be disconnected when using a 2-ROM set.)

It may be possible to lift some of these limitations, but that may require an (even) slower serving algorithm, and hence limit the systems it works on/STM32F4s you can run it on.  Having said that, smaller ROMs are typically used in slower systems.

## Performance Analysis

### Timing Requirements vs. STM32 Performance

These figures were measured using single systems during testing.  Your values may vary, and these should be considered absolute minima.

| System | Min Frequency | STM32 Variant Required |
|--------|---------------|------------------------|
| PET 4032 (50Hz) | 26MHz | F401/F411/F405/F446 |
| VIC-20 (PAL) Kernal | 37MHz | F401/F411/F405/F446 |
| VIC-20 (PAL) Character | 72MHz | F401/F411/F405/F446 |
| C64 (PAL) Kernal | 75MHz | F401/F411/F405/F446 |
| C64 (PAL) Character | 98MHz | F411/F405/F446 |

The F405 at 168MHz and F446 at 180MHz provide substantial headroom for all these systems.

### Multi-ROM performance

When replacing multiple ROMs simultaneously, One ROM has to use a different, slighty less performant algorithm to serve the data.  This means that for multi-ROM support you require a faster variant than the above figures suggest:

| System | Min Frequency | STM32 Variant Required | Notes |
|--------|---------------|------------------------|-------|
| C64 (PAL) | 88MHz | F411/F405/F446 | Kernal & BASIC & Char ROM |

### Startup Performance

Startup performance varies based on the STM32F4 variant used and logging (and other) configuration.  These values were measured on a F411RE board:

- **With logging**: ~3ms boot time
- **Without logging**: ~1.5ms boot time

These values were measured from a stable 5V power rail, to just before the device entering its main loop.  It is likely that most of the 1.5ms startup time is the STM32F4's hardware intialisation rather than the One ROM code itself, as avoiding copying the ROM data into RAM only (the bulk of the startup processing) saves a few 100us.

As the processor is typically held at reset after power on, this gives One ROM sufficient time to start up and be ready to respond to the first access from the system.  It is possible that a non-"held-in-reset" device, like a video chip, may access the character ROM before One ROM is ready, in which case garbage (likely 0xFF) will be returned.  This is unlikely to be a problem in a practice, as the video output is probably not yet visible due to a CRT warming up, or monitor still locking to the video signal.  Worse case, garbage may appear for a few ms.

### Power Consumption

Total power draw at 5V is estimated to be around 100mA, but this has not been measured.
