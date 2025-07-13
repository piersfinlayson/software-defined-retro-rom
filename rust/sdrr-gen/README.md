## 7. Next Steps

1. Implement Phase 1 (local file handling):
   - Basic sdrr-gen CLI structure
   - ROM type selection
   - CS configuration
   - ROM file processing
   - C file generation

2. Test with existing ROM images and build process

3. Plan Phase 2 (database integration):
   - Define database format
   - Create initial repository structure
   - Implement fetching and caching in sdrr-gen

4. Develop Phase 3 (tester integration):
   - Define format for test configurations
   - Implement test file generation in sdrr-gen
   - Validate end-to-end workflow# SDRR Workflow and sdrr-gen Design

## 1. Project Overview

The Software Defined Retro ROM (SDRR) project uses an STM32 microcontroller to emulate vintage ROM chips (2316, 2332, or 2364) for retro computing equipment. The project includes:

- **sdrr/**: Firmware written in C/assembly for the STM32
- **sdrr-tester/**: Rust-based Pico program to validate functionality
- **sdrr-gen/**: New Rust tool for configuration and generation

The sdrr-gen tool will handle:
- ROM preprocessing for performance optimization
- Support for up to 8 ROM images selectable via 3 jumpers
- Configuration options for different ROM types and chip select behavior

## 2. ROM Database Structure

### Fixed ROM Type Properties (Not Stored as Metadata)
- **2316**: 3 CS lines, 2KB storage, fixed pinout
- **2332**: 2 CS lines, 4KB storage, fixed pinout
- **2364**: 1 CS line, 8KB storage, fixed pinout

### Required Metadata

#### ROM Images
- **Name**: Descriptive name of the ROM
- **Type**: Compatible ROM type (2316, 2332, 2364)
- **Description**: What system/purpose the ROM was for
- **Origin**: Original hardware/system
- **URL**: Source location for download
- **Checksums**: SHA-256 or similar for verification
- **CS Configuration**: Active high/low setting for each CS line
- **Year**: Release year (if known)
- **Version/Revision**: If multiple versions exist
- **Access Timing**: Optional read cycle timing requirements
- **Special Handling**: Any preprocessing requirements

## 3. Workflow Process

### User Journey
1. User selects target ROM type (2316, 2332, 2364)
2. User selects STM32 variant (R8T6 or RBT6)
3. User selects ROM images (1-8) from either:
   - Local files (with optional validation) - Phase 1 priority
   - Online database (downloaded automatically) - Future phase
4. For each ROM image, user configures CS behavior (active high/low for each line)
5. sdrr-gen automatically duplicates images if fewer than 8 are selected (pattern: 1→11111111, 2→12121212, 3→12312312)
6. sdrr-gen preprocesses ROM images for optimal performance
7. sdrr-gen generates C header/source files containing ROM data
8. Makefile integrates generated files into firmware build
9. User flashes resulting firmware to STM32
10. Optional: Generate test configuration for Pico tester

### Database Update Mechanism
- sdrr-gen checks for ROM database updates on startup
- Local cache used to reduce bandwidth usage
- Manual update option for offline environments

## 4. sdrr-gen Tool Design

### Architecture
- **Language**: Rust
- **Command-line interface**: For integration with build scripts
- **Optional GUI**: For less technical users (future enhancement)

### Core Components
- **Configuration Handler**: Manages user selections and validation
  - ROM type selection (2316, 2332, 2364)
  - STM32 variant selection (R8T6 or RBT6)
  - CS configuration per image
  - Image slot allocation
  
- **File Manager**: Handles file operations
  - Local ROM file reading (Phase 1)
  - Database downloading and parsing (Phase 2)
  - Checksum validation
  
- **ROM Processor**: Performs necessary transformations
  - Image duplication to fill 8 slots
  - Access speed optimization
  - Memory layout organization
  
- **Code Generator**: Creates firmware files
  - C headers with preprocessed ROM data
  - Configuration constants for CS behavior
  - STM32 variant-specific optimizations
  
- **Tester Generator**: Creates Pico tester files (Phase 3)
  - Test vector generation
  - CS verification configuration
  - Timing validation parameters

### ROM Processing Steps
1. Validate ROM image integrity against checksums
2. Duplicate images to fill all 8 slots if needed
3. Apply transformations for access speed optimization
4. Format data for efficient storage and retrieval
5. Generate appropriate lookup tables or access methods
6. Create corresponding test vectors (when tester integration enabled)

## 5. Implementation Considerations

### Development Phases
1. **Phase 1**: Local file selection and processing
   - Basic CLI structure for sdrr-gen
   - ROM type selection
   - CS configuration
   - ROM file processing
   - C file generation

2. **Phase 2**: ROM database integration
   - Separate repository from main SDRR project
   - Simple format (JSON/TOML) for easy community contributions
   - Download and caching mechanism for sdrr-gen
   - Versioning to track database updates

3. **Phase 3**: Pico tester integration
   - Generate corresponding test configurations from sdrr-gen
   - Validate CS behavior and timing
   - Automated verification of ROM data integrity

### Hardware Support
- STM32F103R8T6: May support fewer total ROM images due to flash limitations
- STM32F103RBT6: Full support for 8 ROM images

### Performance Optimizations
- Pre-calculate address mappings
- Organize data for minimal runtime computation
- Balance between flash storage efficiency and execution speed
- Consider memory alignment requirements

### Error Handling
- Comprehensive validation of user inputs
- Clear error messages for common issues
- Fallback options for network failures
- Verification of generated outputs

### Distribution
- Document process for adding custom ROMs
- Package sdrr-gen with firmware releases

## 6. Pico Tester Integration

The SDRR project includes a Rust-based Pico tester that validates the firmware functionality:

### Tester Capabilities
- Verification of CS line behavior (active high/low)
- Timing validation for read cycles
- Data integrity checking

### Integration with sdrr-gen
- Generate compatible test configurations from the same input
- Export timing requirements for verification
- Create test vectors based on selected ROM images

### Workflow Integration
- Option to generate both firmware and tester files in one operation
- Consistent configuration across both components
- Simplified validation process for users

## 8. Technical Implementation Details

### Command-Line Interface

Uses clap for argument parsing and validation.

- **Basic Usage**: `sdrr-gen --type <ROM_TYPE> --stm <STM_VARIANT> [options] <ROM_FILES...>`
- **Examples**:
  ```
  sdrr-gen --type 2316 --stm r8t6 --cs1 0 --cs2 0 --cs3 0 apple1.bin
  sdrr-gen --type 2364 --stm rbt6 --cs1 0 --debug rom1.bin rom2.bin
  ```
- **Options**:
  - `--type`: ROM type (2316, 2332, 2364) [required]
  - `--stm`: STM32 variant (r8t6, rbt6) [required]
  - `--cs1`: CS1 configuration (0=active low, 1=active high) [required]
  - `--cs2`: CS2 configuration (required for 2316 and 2332)
  - `--cs3`: CS3 configuration (required for 2316 only)
  - `--debug`: Enable debug mode (RTT logging)
  - `--output`: Output directory [default: ./output]
  - `--db`: Use ROM database (with optional URL) [Phase 2]
  - `--tester`: Generate Pico tester configuration [Phase 3]
  
**Note**: CS configuration is for the entire firmware, not per ROM image.

### Output File Format
- **Output Structure**:
  ```
  output/
  ├── roms.h          # Header with ROM data and configuration
  ├── roms.c          # Implementation file
  └── sdrr_config.h   # Firmware configuration defines
  ```

- **Header File Format** (roms.h):
  ```c
  #ifndef SDRR_ROMS_H
  #define SDRR_ROMS_H

  #include <stdint.h>
  #include "sdrr_config.h"

  // Number of ROM images
  #define SDRR_NUM_IMAGES 8

  // ROM data (contains duplicated images as needed to fill 8 slots)
  extern const uint8_t sdrr_rom_data[SDRR_NUM_IMAGES][<ROM_SIZE>];

  #endif // SDRR_ROMS_H
  ```

- **Configuration Header** (sdrr_config.h):
  ```c
  #ifndef SDRR_CONFIG_H
  #define SDRR_CONFIG_H

  // ROM type
  #define ROM_2316 0
  #define ROM_2332 0
  #define ROM_2364 0
  #define ROM_<TYPE> 1  // Only one of the above will be set to 1

  // Debug mode
  #ifdef DEBUG_ENABLED
  #define DEBUG 1
  #endif

  // CS configuration
  #ifdef CS1_LOW
  #define CS1_ACTIVE_LOW
  #else
  #define CS1_ACTIVE_HIGH
  #endif

  #ifdef CS2_LOW
  #define CS2_ACTIVE_LOW
  #else
  #define CS2_ACTIVE_HIGH
  #endif

  #ifdef CS3_LOW
  #define CS3_ACTIVE_LOW
  #else
  #define CS3_ACTIVE_HIGH
  #endif

  #endif // SDRR_CONFIG_H
  ```

- **Implementation File Format** (roms.c):
  ```c
  #include "roms.h"

  // ROM data (preprocessed for optimal access, with duplicates as needed)
  const uint8_t sdrr_rom_data[SDRR_NUM_IMAGES][<ROM_SIZE>] = {
    {
      <PREPROCESSED_ROM_DATA_1>
    },
    {
      <PREPROCESSED_ROM_DATA_2_OR_DUPLICATE_1>
    },
    // ...and so on for all 8 slots
  };
  ```

### ROM Preprocessing Algorithm
1. Read raw binary ROM data
2. Validate size matches expected ROM type:
   - 2316: Exactly 2KB (2048 bytes)
   - 2332: Exactly 4KB (4096 bytes)
   - 2364: Exactly 8KB (8192 bytes)
3. Generate appropriate ROM type defines: `ROM_2316`, `ROM_2332`, or `ROM_2364`
4. Generate CS configuration defines: `CS1_ACTIVE_LOW`/`CS1_ACTIVE_HIGH`, etc.
5. Generate debug configuration if requested
6. Duplicate ROM images as needed to fill all 8 slots - no longer done
7. For each address:
   - Calculate corresponding index in output array
   - Apply address transformation based on ROM type
   - Store data byte at optimized location
8. Generate all output files with proper defines

### Project Structure for sdrr-gen
```
sdrr-gen/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point, CLI handling
│   ├── config.rs          # Configuration structures
│   ├── rom_types.rs       # ROM type definitions and properties
│   ├── preprocessor.rs    # ROM data preprocessing
│   ├── generator.rs       # C file generation
│   ├── database.rs        # ROM database handling (Phase 2)
│   └── tester.rs          # Pico tester integration (Phase 3)
├── tests/                 # Unit and integration tests
└── examples/              # Example ROM configurations
```

### Error Handling and Validation
- **ROM Size Validation**: Strictly validate ROM file sizes:
  - 2364: Exactly 8KB (8192 bytes)
  - 2332: Exactly 4KB (4096 bytes) 
  - 2316: Exactly 2KB (2048 bytes)
- **CS Configuration Validation**:
  - 2364: Requires only --cs1
  - 2332: Requires both --cs1 and --cs2
  - 2316: Requires all three: --cs1, --cs2, and --cs3
- **ROM Duplication Logic** - no longer done:
  - If 1 ROM provided: Duplicate 8 times (11111111)
  - If 2 ROMs provided: Alternate (12121212)
  - If 3 ROMs provided: Repeat pattern (12312312)
  - And so on
- Verify STM32 variant can support requested configuration
- Ensure all required parameters are provided based on ROM type
- Detailed error messages for common issues