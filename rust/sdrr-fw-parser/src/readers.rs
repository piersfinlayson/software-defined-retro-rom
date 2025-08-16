// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-fw-parser
//!
//! Contains standard re-usable reader implementations for parsing SDRR firmware

use crate::Reader;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// A reader that operates on an in-memory firmware image.
///
/// This is the standard reader for PC applications that can load the entire
/// firmware file into memory. It handles translation between absolute memory
/// addresses and offsets within the firmware data.
///
/// This is unlikely to be appropriate for embedded applications, which
/// should implement their own `Reader` trait to read from flash or other
/// memory directly.
///
/// # Example
///
/// ```rust
/// use sdrr_fw_parser::{Parser, readers::MemoryReader};
///
/// // Load firmware file
/// let firmware_data = std::fs::read("firmware.bin")?;
///
/// // Create reader starting at STM32F4 flash base
/// let reader = MemoryReader::new(firmware_data, 0x08000000);
///
/// // Parse the firmware
/// let mut parser = Parser::new(reader);
/// let info = parser.parse()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct MemoryReader {
    data: Vec<u8>,
    base_address: u32,
}

impl MemoryReader {
    /// Create a new memory reader.
    ///
    /// # Arguments
    ///
    /// * `data` - The complete firmware image data
    /// * `base_address` - The base address where this firmware would be loaded
    ///   in the target device (typically `0x08000000` for STM32F4)
    pub fn new(data: Vec<u8>, base_address: u32) -> Self {
        Self { data, base_address }
    }

}

impl Reader for MemoryReader {
    type Error = String;

    async fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        if addr < self.base_address {
            return Err(format!(
                "Address 0x{:08X} is below base address 0x{:08X}",
                addr, self.base_address
            ));
        }

        let offset = (addr - self.base_address) as usize;
        let end = offset.saturating_add(buf.len());

        if end > self.data.len() {
            return Err(format!(
                "Read at 0x{:08X} (offset {}) extends past firmware end",
                addr, offset
            ));
        }

        buf.copy_from_slice(&self.data[offset..end]);
        Ok(())
    }

    fn update_base_address(&mut self, new_base: u32) {
        self.base_address = new_base;
    }
}
