use crate::config::SizeHandling;
use crate::rom_types::{RomType, StmFamily};
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

    fn transform_address_f1(address: usize) -> usize {
        // This array maps each address bit (index) to its corresponding GPIO
        // pin number.  For example, address bit 1 (A1) is connected to GPIO
        // pin 7 (actually PB10 shifted right twice and the PB5/bit 3
        // ignored).
        const ADDRESS_TO_PIN_MAP: [usize; 13] = [
            0,  // A0 is connected to GPIO pin "0" PB2
            7,  // A1 is connected to GPIO pin "7" PB10
            8,  // A2 is connected to GPIO pin "8" PB11
            9,  // A3 is connected to GPIO pin "9" PB12
            10, // A4 is connected to GPIO pin "10" PB13
            11, // A5 is connected to GPIO pin "11" PB14
            12, // A6 is connected to GPIO pin "12" PB15
            1,  // A7 is connected to GPIO pin "1" PB3
            2,  // A8 is connected to GPIO pin "2" PB4
            3,  // A9 is connected to GPIO pin "3" PB5
            6,  // A10 is connected to GPIO pin "6" PB9
            5,  // A11 is connected to GPIO pin "5" PB8
            4,  // A12 is connected to GPIO pin "4" PB7
        ];

        // Start with 0 result
        let mut result = 0;

        // For each address bit, check if it's set in the original address
        // If it is, set the corresponding GPIO pin bit in the result
        for bit in 0..ADDRESS_TO_PIN_MAP.len() {
            if (address & (1 << bit)) != 0 {
                // Map this address bit to its GPIO pin position
                let pin = ADDRESS_TO_PIN_MAP[bit];
                result |= 1 << pin;
            }
        }

        result
    }

    fn transform_address_f4(address: usize, rom_type: &RomType) -> usize {
        // This is even more mind-bending that the F1 mapping.  Here we have
        // to cope with the fact that the hardware pins have one or more
        // invalid bits (CS lines).  So we need ignore that bit/those bits.
        // This involves us running the mapping logic in reverse to
        // transform_address_f1().
        let pin_to_addr_map = match rom_type {
            RomType::Rom2364 => [
                Some(7),
                Some(6),
                Some(5),
                Some(4),
                Some(1),
                Some(0),
                Some(2),
                Some(3),
                Some(8),
                Some(12),
                None, // CS1
                Some(10),
                Some(11),
                Some(9),
            ],
            RomType::Rom2332 => [
                Some(7),
                Some(6),
                Some(5),
                Some(4),
                Some(1),
                Some(0),
                Some(2),
                Some(3),
                Some(8),
                None, // CS2
                None, // CS1
                Some(10),
                Some(11),
                Some(9),
            ],
            RomType::Rom2316 => [
                Some(7),
                Some(6),
                Some(5),
                Some(4),
                Some(1),
                Some(0),
                Some(2),
                Some(3),
                Some(8),
                None, // CS3
                None, // CS1
                Some(10),
                None, // CS2
                Some(9),
            ],
        };

        // Start with 0 result
        let mut result = 0;

        for pin in 0..pin_to_addr_map.len() {
            if let Some(addr_bit) = pin_to_addr_map[pin] {
                // Check if this pin is set in the original address
                if (address & (1 << pin)) != 0 {
                    // Set the corresponding address bit in the result
                    result |= 1 << addr_bit;
                }
            }
        }

        result
    }

    /// Transforms from a physical address (based on the hardware pins) to
    /// a logical ROM address, so we store the physical ROM mapping, rather
    /// than the logical one.
    pub fn transform_address(
        &self,
        address: usize,
        family: &StmFamily,
        rom_type: &RomType,
    ) -> usize {
        match family {
            StmFamily::F1 => Self::transform_address_f1(address),
            StmFamily::F4 => Self::transform_address_f4(address, rom_type),
        }
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
    ///
    /// The hardware mapping (STM32F1) is:
    /// PC6 - D0
    /// PC7 - D1
    /// PC8 - D2
    /// PC9 - D7
    /// PA8 - D6
    /// PA9 - D5
    /// PA10 - D4
    /// PA11 - D3
    ///
    /// When reading the byte the hardware
    /// - Masks with 0x0F and shifts left 6 bits to apply to port C
    /// - Masks with 0xF0 and shifts left 4 (more) bits to apply to port A
    pub fn transform_byte(&self, byte: u8, family: &StmFamily) -> u8 {
        // This array maps each original bit position to its new position
        let bit_posn_map = match family {
            StmFamily::F1 => [
                0, // Bit 0 stays at position 0
                1, // Bit 1 stays at position 1
                2, // Bit 2 stays at position 2
                7, // Bit 3 moves to position 7 (MSB)
                6, // Bit 4 moves to position 6
                5, // Bit 5 stays at position 5
                4, // Bit 6 moves to position 4
                3, // Bit 7 moves to position 3
            ],
            StmFamily::F4 => [7, 6, 5, 4, 3, 2, 1, 0],
        };

        // Start with 0 result
        let mut result = 0;

        // For each bit in the original byte
        for bit_pos in 0..8 {
            // Check if this bit is set in the original byte
            if (byte & (1 << bit_pos)) != 0 {
                // Get the new position for this bit
                let new_pos = bit_posn_map[bit_pos];
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
    pub fn get_byte(&self, address: usize, family: &StmFamily, rom_type: &RomType) -> u8 {
        // We have been passed a physical address based on the hardware pins,
        // so we need to transform it to a logical address based on the ROM
        // image.
        let transformed_address = self.transform_address(address, family, rom_type);

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
        self.transform_byte(byte, &family)
    }
}
