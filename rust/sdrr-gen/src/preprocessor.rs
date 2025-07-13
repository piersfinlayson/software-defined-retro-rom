// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use crate::config::{SizeHandling, RomInSet};
use sdrr_common::sdrr_types::{RomType, CsLogic};
use sdrr_common::hardware::HwConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

// A ROM image that has been validated and loaded
#[derive(Debug, Clone)]
pub struct RomImage {
    pub data: Vec<u8>,
}

impl RomImage {
    pub fn load_from_file(
        file_path: &Path,
        rom_type: &RomType,
        size_handling: &SizeHandling,
    ) -> Result<Self> {
        let data = fs::read(file_path)
            .with_context(|| format!("Failed to read ROM file: {}", file_path.display()))?;

        let expected_size = rom_type.size_bytes();

        let final_data = match data.len().cmp(&expected_size) {
            std::cmp::Ordering::Equal => {
                // Exact match - error if dup/pad specified unnecessarily
                match size_handling {
                    SizeHandling::None => data,
                    _ => anyhow::bail!(
                        "ROM file is already correct size ({} bytes), 'dup' or 'pad' not needed",
                        expected_size
                    ),
                }
            }
            std::cmp::Ordering::Less => {
                // File too small - handle with dup/pad
                match size_handling {
                    SizeHandling::None => anyhow::bail!(
                        "Invalid ROM size for {}: expected {} bytes, got {} bytes",
                        file_path.display(),
                        expected_size,
                        data.len()
                    ),
                    SizeHandling::Duplicate => {
                        if expected_size % data.len() != 0 {
                            anyhow::bail!(
                                "ROM size {} is not an exact divisor of {} bytes",
                                data.len(),
                                expected_size
                            );
                        }
                        let repeat_count = expected_size / data.len();
                        data.repeat(repeat_count)
                    }
                    SizeHandling::Pad => {
                        let mut padded = data;
                        padded.resize(expected_size, 0xAA);
                        padded
                    }
                }
            }
            std::cmp::Ordering::Greater => {
                anyhow::bail!(
                    "ROM file too large: expected {} bytes, got {} bytes",
                    expected_size,
                    data.len()
                );
            }
        };

        Ok(Self { data: final_data })
    }

    /// Transforms from a physical address (based on the hardware pins) to
    /// a logical ROM address, so we store the physical ROM mapping, rather
    /// than the logical one.
    pub fn transform_address(
        &self,
        address: usize,
        phys_pin_to_addr_map: &[Option<usize>]
    ) -> usize {
        // Start with 0 result
        let mut result = 0;

        for pin in 0..phys_pin_to_addr_map.len() {
            if let Some(addr_bit) = phys_pin_to_addr_map[pin] {
                // Check if this pin is set in the original address
                if (address & (1 << pin)) != 0 {
                    // Set the corresponding address bit in the result
                    result |= 1 << addr_bit;
                }
            }
        }

        result
    }

    /// Transforms a data byte by rearranging its bit positions to match the hardware's
    /// data pin connections.
    ///
    /// The hardware has a non-standard mapping for data pins, so we need to rearrange
    /// the bits to ensure correct data is read/written.
    ///
    /// Bit mapping:
    /// Original:  7 6 5 4 3 2 1 0
    /// Mapped to: 3 4 5 6 7 2 1 0
    ///
    /// For example:
    /// - Original bit 7 (MSB) moves to position 3
    /// - Original bit 3 moves to position 7 (becomes new MSB)
    /// - Bits 2, 1, and 0 remain in the same positions
    ///
    /// This transformation ensures that when the hardware reads a byte through its
    /// data pins, it gets the correct bit values despite the non-standard connections.
    pub fn transform_byte(byte: u8, phys_pin_to_data_map: &[usize]) -> u8 {
        // Start with 0 result
        let mut result = 0;

        // For each bit in the original byte
        for bit_pos in 0..8 {
            // Check if this bit is set in the original byte
            if (byte & (1 << bit_pos)) != 0 {
                // Get the new position for this bit
                let new_pos = phys_pin_to_data_map[bit_pos];
                // Set the bit in the result at its new position
                result |= 1 << new_pos;
            }
        }

        result
    }

    /// Get byte at the given address with both address and data
    /// transformations applied.
    ///
    /// This function:
    /// 1. Transforms the address to match the hardware's address pin mapping
    /// 2. Retrieves the byte at that transformed address
    /// 3. Transforms the byte's bit pattern to match the hardware's data pin
    ///    mapping
    ///
    /// This ensures that when the hardware reads from a certain address
    /// through its GPIO pins, it gets the correct byte value with bits
    /// arranged according to its data pin connections.
    pub fn get_byte(&self, address: usize, phys_pin_to_addr_map: &[Option<usize>], phys_pin_to_data_map: &[usize]) -> u8 {
        // We have been passed a physical address based on the hardware pins,
        // so we need to transform it to a logical address based on the ROM
        // image.
        let transformed_address = self.transform_address(address, phys_pin_to_addr_map);

        // Sanity check that we did get a logical address, which must by
        // definition fit within the actual ROM size.
        assert!(transformed_address < self.data.len());

        // Get the byte from the logical ROM address.
        let byte = self.data.get(transformed_address).copied().expect(
            // Hitting this error suggests an internal error
            format!(
                "Address {} out of bounds for ROM image of size {}",
                transformed_address,
                self.data.len()
            )
            .as_str(),
        );

        // Now transform the byte, as the physical data lines are not in the
        // expected order (0-7).
        Self::transform_byte(byte, phys_pin_to_data_map)
    }
}

#[derive(Debug, Clone)]
pub struct RomSet {
    pub id: usize,
    pub roms: Vec<RomInSet>,
}

impl RomSet {
    pub fn get_byte(&self, address: usize, hw: &HwConfig) -> u8 {
        let phys_pin_to_data_map = hw.get_phys_pin_to_data_map();

            // Backward compatibility: single ROM uses existing behavior
        if self.roms.len() == 1 {
            let num_addr_lines = self.roms[0].config.rom_type.num_addr_lines();
            let phys_pin_to_addr_map = hw.get_phys_pin_to_addr_map(num_addr_lines);

            return self.roms[0].image.get_byte(address, &phys_pin_to_addr_map, &phys_pin_to_data_map);
        }

        // Multiple ROMs: check CS line states to select responding ROM
        for (index, rom_in_set) in self.roms.iter().enumerate() {
            // Get the physical addr and data pin mappings.  We have to
            // retrieve this for each ROM in the set, as each ROM may be
            // a different type (size).
            let num_addr_lines = rom_in_set.config.rom_type.num_addr_lines();
            let phys_pin_to_addr_map = hw.get_phys_pin_to_addr_map(num_addr_lines);

            // All of CS1/X1/X2 have to have the same active low/high status
            // so we retrieve that from CS1 (as X1/X2 aren't specifically
            // configured in the rom sets).
            let pins_active_high = rom_in_set.config.cs_config.cs1 == CsLogic::ActiveHigh;

            // Get the CS pin that controls this ROM's selection
            let cs_pin = hw.cs_pin_for_rom_in_set(&rom_in_set.config.rom_type, index);
            assert!(cs_pin <= 15, "Internal error: CS pin is > 15");

            fn is_pin_active(active_high: bool, address: usize, pin: u8) -> bool {
                if active_high {
                    (address & (1 << pin)) != 0
                } else {
                    (address & (1 << pin)) == 0
                }
            }

            let cs_active = is_pin_active(pins_active_high, address, cs_pin);
            
            if cs_active {
                // Verify exactly one CS pin is active
                let cs1_pin = hw.pin_cs1(&rom_in_set.config.rom_type);
                let x1_pin = hw.pin_x1();
                let x2_pin = hw.pin_x2();

                let cs1_is_active = is_pin_active(pins_active_high, address, cs1_pin);
                let x1_is_active = is_pin_active(
                    pins_active_high,
                    address,
                    x1_pin
                );
                let x2_is_active = is_pin_active(
                    pins_active_high,
                    address,
                    x2_pin
                );

                let active_count = [cs1_is_active, x1_is_active, x2_is_active].iter().filter(|&&x| x).count();
                
                // Only return the byte for a single CS active, otherwise
                // it'll get 0xAA
                if active_count == 1 && self.check_rom_cs_requirements(rom_in_set, address, hw) {
                    return rom_in_set.image.get_byte(address, &phys_pin_to_addr_map, &phys_pin_to_data_map);
                }
            }
        }

        RomImage::transform_byte(0xAA, &phys_pin_to_data_map) // No ROM selected
    }

    fn check_rom_cs_requirements(&self, rom_in_set: &RomInSet, address: usize, hw: &HwConfig) -> bool {
        let cs_config = &rom_in_set.config.cs_config;
        let rom_type = &rom_in_set.config.rom_type;
        
        // Check CS2 if specified
        if let Some(cs2_logic) = cs_config.cs2 {
            match cs2_logic {
                CsLogic::Ignore => {
                    // CS2 state doesn't matter
                },
                CsLogic::ActiveLow => {
                    let cs2_pin = hw.pin_cs2(rom_type);
                    let cs2_active = (address & (1 << cs2_pin)) == 0;
                    if !cs2_active { return false; }
                },
                CsLogic::ActiveHigh => {
                    let cs2_pin = hw.pin_cs2(rom_type);
                    let cs2_active = (address & (1 << cs2_pin)) != 0;
                    if cs2_active { return false; }
                },
            }
        }
        
        // Check CS3 if specified
        if let Some(cs3_logic) = cs_config.cs3 {
            match cs3_logic {
                CsLogic::Ignore => {
                    // CS3 state doesn't matter
                },
                CsLogic::ActiveLow => {
                    let cs3_pin = hw.pin_cs3(rom_type);
                    let cs3_active = (address & (1 << cs3_pin)) == 0;
                    if !cs3_active { return false; }
                },
                CsLogic::ActiveHigh => {
                    let cs3_pin = hw.pin_cs3(rom_type);
                    let cs3_active = (address & (1 << cs3_pin)) != 0;
                    if cs3_active { return false; }
                },
            }
        }
        
        true
    }

    #[allow(dead_code)]
    fn mask_cs_selection_bits(&self, address: usize, rom_type: &RomType, hw: HwConfig) -> usize {
        let mut masked_address = address;
        
        // Remove the CS selection bits - only mask bits that exist on this hardware
        masked_address &= !(1 << hw.pin_cs1(rom_type));

        // Only mask X1/X2 on hardware that has them (revision F)
        if hw.supports_multi_rom_sets() {
            let x1 = hw.pin_x1();
            let x2 = hw.pin_x2();
            assert!(x1 < 15 && x2 < 15, "X1/X2 pins must be less than 15");
            masked_address &= !(1 << x1);
            masked_address &= !(1 << x2);
        }
        
        // Remove CS2/CS3 bits based on ROM type
        match rom_type {
            RomType::Rom2332 => {
                masked_address &= !(1 << hw.pin_cs2(rom_type));
            },
            RomType::Rom2316 => {
                masked_address &= !(1 << hw.pin_cs2(rom_type));
                masked_address &= !(1 << hw.pin_cs3(rom_type));
            },
            RomType::Rom2364 => {
                // 2364 only uses CS1, no additional bits to remove
            },
            RomType::Rom23128 => {
                // No additional bits to remove
            },
        }
        
        // Ensure address fits within ROM size
        masked_address & ((1 << 13) - 1) // Mask to 13 bits max (8KB)
    }
}
