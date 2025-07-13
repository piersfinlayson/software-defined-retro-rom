# 🚀 Software Defined Retro Rom Tester

This application is a high-performance tester for the Software Defined Retro Rom project. The test setup consists of an SDRR device under test (based on the STM32F103 microcontroller) running the SDRR firmware, and a Raspberry Pi Pico running this tester firmware.

## 🔍 Overview

The tester firmware acts like a retro computer's CPU, querying data from the SDRR device and validating the results against expected values. It performs two critical functions:

- ✅ **Functional testing**: Verifies that the correct data is returned for each address
- ⏱️ **Performance testing**: Measures response times to ensure the SDRR device meets timing targets specified in [SDRR's README](../README.md)

## 💻 Technical Details

This tester is implemented as a Rust [embassy-rs](https://embassy.dev/) application that uses direct hardware register access for maximum performance:

- 🔧 **Direct register access**: Uses memory-mapped hardware registers for GPIO operations with minimal overhead
- 📊 **Precise timing**: Utilizes the RP2040's hardware timer for accurate performance measurements
- 📈 **Statistics**: Calculates min/max/average response times and other performance metrics
- 🧪 **Test patterns**: Can verify ROM contents against repeating test patterns

## 📋 Hardware Requirements

- 🧠 **Raspberry Pi Pico**: Standard model running at 125MHz (for precise timing calculations)
- 🔌 **SWD Connection**: Uses RTT over SWD for logging without impacting performance testing

## 🔌 GPIO Configuration

The tester uses the following GPIO pin assignments:

### Address Pins
- For ROM 2364: GPIO 0-12 (13 pins for A0-A12)
- For ROM 2332: GPIO 0-11 (12 pins for A0-A11)
- For ROM 2316: GPIO 0-10 (11 pins for A0-A10)

### Data Pins
- GPIO 14-21 for D0-D7 (8 data pins)

### Chip Select Pins
- CS1: GPIO 13 (used by all ROM types)
- CS2: GPIO 12 (only used by 2332 & 2316)
- CS3: GPIO 11 (only used by 2316)

## 🛠️ Building and Running

```bash
cd rust/sdrr-tester

# Build for your target ROM type (2316, 2332, or 2364)
cargo build --release --no-default-fatures --features "rom_2364 cs1_active_low"

# Flash to Pico using SWD
probe-rs run --release
```

## 🎛️ Feature Configuration

The tester supports various ROM types and chip select configurations through feature flags:

### ROM Type Features (pick exactly one)
- `rom_2316`: 2KB ROM (2316 type) with 11 address lines
- `rom_2332`: 4KB ROM (2332 type) with 12 address lines
- `rom_2364`: 8KB ROM (2364 type) with 13 address lines

### Chip Select Features
Different ROM types require different chip select configurations:

#### For ROM 2364
- Requires exactly one of: `cs1_active_low` or `cs1_active_high`

#### For ROM 2332
- Requires exactly one of: `cs1_active_low` or `cs1_active_high`
- Requires exactly one of: `cs2_active_low` or `cs2_active_high`

#### For ROM 2316
- Requires exactly one of: `cs1_active_low` or `cs1_active_high`
- Requires exactly one of: `cs2_active_low` or `cs2_active_high`
- Requires exactly one of: `cs3_active_low` or `cs3_active_high`

### Valid Feature Combinations Examples
```
# For 2364 ROM
--features "rom_2364 cs1_active_low"
--features "rom_2364 cs1_active_high"

# For 2332 ROM
--features "rom_2332 cs1_active_low cs2_active_low"
--features "rom_2332 cs1_active_high cs2_active_low"
--features "rom_2332 cs1_active_low cs2_active_high"
--features "rom_2332 cs1_active_high cs2_active_high"

# For 2316 ROM
--features "rom_2316 cs1_active_low cs2_active_low cs3_active_low"
--features "rom_2316 cs1_active_high cs2_active_low cs3_active_low"
--features "rom_2316 cs1_active_low cs2_active_high cs3_active_low"
--features "rom_2316 cs1_active_high cs2_active_high cs3_active_low"
--features "rom_2316 cs1_active_low cs2_active_low cs3_active_high"
--features "rom_2316 cs1_active_high cs2_active_low cs3_active_high"
--features "rom_2316 cs1_active_low cs2_active_high cs3_active_high"
--features "rom_2316 cs1_active_high cs2_active_high cs3_active_high"
```

## 📢 Output Examples

When running, the tester outputs detailed timing statistics through RTT:

```
=== ROM Access Timing Statistics ===
Sample size: 0x2000 addresses
--- ROM Read Time (D7 going low) ---
Min: 34 ticks (272 ns)
Max: 41 ticks (328 ns)
Avg: 36 ticks (288 ns)
StdDev: 1 ticks (8 ns)
--- ROM Reset Time (D7 going high) ---
Min: 5 ticks (40 ns)
Max: 8 ticks (64 ns)
Avg: 6 ticks (48 ns)
StdDev: 0 ticks (0 ns)
--- Total Cycle Time (Read + Reset) ---
Min cycle: 39 ticks (312 ns)
Max cycle: 49 ticks (392 ns)
Avg cycle: 42 ticks (336 ns)
Read time variation: 2.77%
Reset time variation: 0.00%
```

## ⚡ Performance Notes

While the SDRR is designed for 5V logic, both the SDRR's STM32 and the Raspberry Pi Pico are 3.3V devices. The SDRR uses 5V tolerant pins, but the Pico drives them with 3.3V logic. This may result in small timing differences compared to a real retro computer with 5V logic.
