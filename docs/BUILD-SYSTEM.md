# SDRR Build System Explanation

The SDRR build system is a multi-stage pipeline that transforms ROM images and configuration into flashable STM32 firmware. This document explains how the build process works internally.

## Build Pipeline Overview

```ascii
User Input → Top-level Make → sdrr-gen → sdrr Make → test (optional) → probe-rs → STM32
    ↓              ↓            ↓          ↓                              ↓         ↓
Config Files  ROM Download    Code Gen    Compile      Compile test     Flash     Running
ROM Images    & Validation    & Mangle    & Link          & Run        Firmware
```

The build process involves four main components:

1. **Top-level Makefile** - Configuration processing and orchestration
2. **sdrr-gen** - Rust tool for ROM processing and generation of firmware code containing the ROMs
3. **sdrr Makefile** - STM32 ARM firmware compilation
4. **test** - Optioanl tests to verify the generated "mangled" ROMs
5. **probe-rs** - STM32 flashing and debugging

## Component Breakdown

### 1. Top-level Makefile ([`/Makefile`](/Makefile))

**Purpose**: Configuration management and build orchestration

**Key Responsibilities**:

- Parse configuration variables (e.g. STM variant, HW revision, ROM configs)
- Convert configuration into command-line arguments for sdrr-gen
- Coordinate the build pipeline stages
- Provide user-friendly error messages and warnings

**Configuration Processing**:

```bash
# User can override via config file or command line
CONFIG=config/c64.mk STM=f411re make run

# Makefile processes ROM_CONFIGS like:
ROM_CONFIGS=file=kernel.rom,type=2364,cs1=0 file=basic.rom,type=2364,cs1=0

# Into sdrr-gen arguments:
--rom file=../kernel.rom,type=2364,cs1=0 --rom file=../basic.rom,type=2364,cs1=0
```

**Build Stages**:

1. `sdrr-gen` target - Build the Rust code generator
2. `gen` target - Run sdrr-gen to create firmware files
3. `firmware` target - Compile the STM32 firmware
4. `run` target - Flash to hardware via probe-rs

### 2. sdrr-gen (Rust Tool)

**Purpose**: ROM processing and firmware code generation

**Key Responsibilities**:

- Download ROM images from HTTP/HTTPS URLs
- Extract files from ZIP archives
- Validate ROM sizes against declared types
- "Mangle" ROM data to match PCB pin mapping
- Generate C source files with embedded ROM data
- Create Makefile fragments with build settings
- Create linker scripts for STM32 variant
- Handle license acceptance for ROM images

**ROM Processing Pipeline**:

Example for a `type=2364` ROM image:

```ascii
Original ROM → Size Validation → Pin Mapping → Flash Layout → C Arrays
      ↓                ↓              ↓             ↓             ↓
  8KB file      Must fit 2364      Rearrange     16KB blocks  static const
                  (8KB) type       for GPIO       in flash    uint8_t arrays
```

**Generated Files** (in [`/output/`](/output/) directory):

- `sdrr_config.h` - SDRR configuration constants
- `roms.c` - ROM data as C arrays
- `roms.h` - ROM declarations and types  
- `generated.mk` - Makefile fragment with STM32 variant information
- `linker.ld` - Linker script for the STM32 variant

### 3. sdrr Makefile ([`/sdrr/Makefile`](/sdrr/Makefile))

**Purpose**: ARM firmware compilation using GCC toolchain

**Key Responsibilities**:

- Compile C source files for target STM32 variant
- Link with SEGGER RTT library for logging output via RTT over SWD
- Include auto-generated ROM data from sdrr-gen
- Generate ELF, binary, and disassembly files

**Toolchain Setup**:

```makefile
TOOLCHAIN := /opt/arm-gnu-toolchain-14.2.rel1-x86_64-arm-none-eabi
CC := $(TOOLCHAIN)/bin/arm-none-eabi-gcc
OBJCOPY := $(TOOLCHAIN)/bin/arm-none-eabi-objcopy
```

**Source File Compilation**:

- `/sdrr/src/*.c` - Main firmware source
- `/sdrr/include/*.h` - Header files
- `/sdrr/link/stm32-common.ld` - Common linker script for all STM32 variants
- `/output/roms.c` - Generated ROM data
- `/output/roms.h` - Generated ROM data
- `/output/sdrr_config.h` - Generated SDRR configuration
- `/output/linker.ld` - Generated linker script for specific STM32 variant
- `/sdrr/segger-rtt/RTT/*.c` - Debug logging library (downloaded automatically by build process)

### 4. Test (Optional)

**Purpose**: Validate the generated mangled ROM data, which is built into the firmware image

**Configuration**:

- ROM_CONFIGS as defined in the top-level Makefile

**Operation**:

The test program builds in the `sdrr-gen` auto-generated `roms.c` output, loads in the original ROM files, and does a byte-by-byte comparison by

- Choosing the next address
- Mangling the address, as it done in the real hardware
- Looking up the byte in the `roms.c` generated data
- Demangling the byte
- Comparing the byte with the original ROM file

This is a complex process when testing rom sets, as the appropriate chip select line(s) have to be activated.

### 5. probe-rs Integration

**Purpose**: Hardware flashing and debugging

**Configuration**:

- Chip ID passed via `generated.mk` (e.g., `STM32F411RETx`), generated by [`sdrr-gen`](#2-sdrr-gen-rust-tool).
- Automatic detection of connected programmers
- Support for ST-Link, Pi Debug Probe, Pi Pico, etc.

## Configuration System

### Variable Hierarchy

1. **Makefile defaults** - Safe baseline settings
2. **Config file** - Project-specific collections (e.g., `config/c64.mk`)
3. **Command line** - Override specific settings

### Key Configuration

See [`CONFIGURATION`](/docs/CONFIGURATION.md).
