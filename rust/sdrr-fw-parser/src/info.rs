// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-fw-parser
//!
//! Structures used to represent the parsed SDRR firmware information

use deku::prelude::*;

use crate::{ParseError, Parser, Reader};
use crate::{
    SdrrAddress, SdrrCsState, SdrrLogicalAddress, SdrrRomType, SdrrServe, SdrrStmPort, StmLine,
    StmStorage,
};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec, vec::Vec};

/// Container for both the parsed firmware information and runtime information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Sdrr {
    pub flash: Option<SdrrInfo>,
    pub ram: Option<SdrrRuntimeInfo>,
}

/// Main SDRR runtime information data structure.  Contains all data parsed
/// from RAM.
/// 
/// Reflects `sdrr_runtime_info_t` from `sdrr/include/config_base.h`
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SdrrRuntimeInfo {
    pub image_sel: u8,
    pub rom_set_index: u8,
    pub count_rom_access: u8,
    pub last_parsed_access_count: u32,
    pub account_count_address: u32,
    pub rom_table_address: u32,
    pub rom_table_size: u32,
}

/// Main SDRR firmware information data structure.  Contains all data parsed
/// from the firmware file.
///
/// Reflects `sdrr_info_t` from `sdrr/include/config_base.h`
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SdrrInfo {
    // Core fields that are always present
    pub major_version: u16,
    pub minor_version: u16,
    pub patch_version: u16,
    pub build_number: u16,
    pub commit: [u8; 8],
    pub stm_line: StmLine,
    pub stm_storage: StmStorage,
    pub freq: u16,
    pub overclock: bool,
    pub swd_enabled: bool,
    pub preload_image_to_ram: bool,
    pub bootloader_capable: bool,
    pub status_led_enabled: bool,
    pub boot_logging_enabled: bool,
    pub mco_enabled: bool,
    pub rom_set_count: u8,
    pub count_rom_access: bool,
    pub boot_config: [u8; 4],

    // Fields that might fail to parse
    pub build_date: Option<String>,
    pub hw_rev: Option<String>,
    pub rom_sets: Vec<SdrrRomSet>, // Empty if failed
    pub pins: Option<SdrrPins>,

    /// Parse errors encountered during parsing
    pub parse_errors: Vec<ParseError>,
}

impl SdrrInfo {
    /// Demangles a byte from the physical pin representation to the logical
    /// representation which is served on D0-D7.  Use when looking up a byte
    /// from the ROM image data to get the "real" byte.
    pub fn demangle_byte(&self, byte: u8) -> Result<u8, String> {
        let pins = self
            .pins
            .as_ref()
            .ok_or("Pin configuration not available")?;

        assert!(pins.data.len() == 8, "Expected 8 data pins");
        let mut result = 0u8;
        for (logic_bit, &phys_pin) in pins.data.iter().enumerate() {
            assert!(phys_pin < 8, "Physical pin {} out of range", phys_pin);
            if (byte & (1 << phys_pin)) != 0 {
                result |= 1 << logic_bit;
            }
        }
        Ok(result)
    }

    /// Takes a logical address and all chip select line states, and produces
    /// a mangled address, as the firmware uses to lookup a byte in the ROM
    /// image stored in firmware.  Use to get the address to index into the
    /// ROM data stored in the firmware, and then use `demangle_byte()` to
    /// turn into a logical byte.
    #[allow(unused_variables)]
    pub fn mangle_address(&self, addr: &SdrrLogicalAddress) -> Result<u32, String> {
        let cs1 = addr.cs_set().cs1();
        let cs2 = addr.cs_set().cs2();
        let cs3 = addr.cs_set().cs3();
        let x1 = addr.cs_set().x1();
        let x2 = addr.cs_set().x2();
        let addr = addr.addr();

        let pins = self
            .pins
            .as_ref()
            .ok_or("Pin configuration not available")?;

        if self.rom_sets.is_empty() {
            return Err("No ROM sets available".into());
        }

        let mut pin_to_addr_map = [None; 16];
        assert!(pins.addr.len() <= 16, "Expected up to 16 address pins");
        for (addr_bit, &phys_pin) in pins.addr.iter().enumerate() {
            if phys_pin < 16 {
                pin_to_addr_map[phys_pin as usize] = Some(addr_bit);
            }
        }

        let num_roms = self.rom_sets[0].rom_count as usize;
        if num_roms > 1 {
            assert!(
                pins.x1 < 16 && pins.x2 < 16,
                "X1 and X2 pins must be less than 16"
            );
            assert!(
                pin_to_addr_map[pins.x1 as usize].is_none()
                    && pin_to_addr_map[pins.x2 as usize].is_none(),
                "X1 and X2 pins must not overlap with other address pins"
            );
            pin_to_addr_map[pins.x1 as usize] = Some(14);
            pin_to_addr_map[pins.x2 as usize] = Some(15);
        }

        let rom_type = self.rom_sets[0].roms[0].rom_type;
        let addr_mask = match rom_type {
            SdrrRomType::Rom2364 => {
                assert!(pins.cs1_2364 < 16, "CS1 pin for 2364 must be less than 16");
                pin_to_addr_map[pins.cs1_2364 as usize] = Some(13);
                0x1FFF // 13-bit address
            }
            SdrrRomType::Rom2332 => {
                assert!(pins.cs1_2332 < 16, "CS1 pin for 2332 must be less than 16");
                assert!(pins.cs2_2332 < 16, "CS2 pin for 2332 must be less than 16");
                pin_to_addr_map[pins.cs1_2332 as usize] = Some(13);
                pin_to_addr_map[pins.cs2_2332 as usize] = Some(12);
                0x0FFF // 12-bit address
            }
            SdrrRomType::Rom2316 => {
                assert!(pins.cs1_2316 < 16, "CS1 pin for 2316 must be less than 16");
                assert!(pins.cs2_2316 < 16, "CS2 pin for 2316 must be less than 16");
                assert!(pins.cs3_2316 < 16, "CS3 pin for 2316 must be less than 16");
                pin_to_addr_map[pins.cs1_2316 as usize] = Some(13);
                pin_to_addr_map[pins.cs2_2316 as usize] = Some(11);
                pin_to_addr_map[pins.cs3_2316 as usize] = Some(12);
                0x07FF // 11-bit address
            }
        };

        let overflow = addr & !addr_mask;
        if overflow != 0 {
            return Err(format!(
                "Requested Address 0x{:08X} overflows the address space for ROM type {}",
                addr, rom_type
            ));
        }

        let mut input_addr = addr & addr_mask;
        match rom_type {
            SdrrRomType::Rom2364 => {
                if cs1 {
                    input_addr |= 1 << 13;
                }
            }
            SdrrRomType::Rom2332 => {
                if cs1 {
                    input_addr |= 1 << 13;
                }
                if let Some(cs2) = cs2 {
                    if cs2 {
                        input_addr |= 1 << 12;
                    }
                }
            }
            SdrrRomType::Rom2316 => {
                if cs1 {
                    input_addr |= 1 << 13;
                }
                if let Some(cs2) = cs2 {
                    if cs2 {
                        input_addr |= 1 << 12;
                    }
                }
                if let Some(cs3) = cs3 {
                    if cs3 {
                        input_addr |= 1 << 11;
                    }
                }
            }
        };

        if num_roms > 1 {
            if let Some(x1) = x1 {
                if x1 {
                    input_addr |= 1 << 14;
                }
            }
            if let Some(x2) = x2 {
                if x2 {
                    input_addr |= 1 << 15;
                }
            }
        }

        let mut result = 0;
        for (pin, item) in pin_to_addr_map.iter().enumerate() {
            if let Some(addr_bit) = item {
                if (input_addr & (1 << addr_bit)) != 0 {
                    result |= 1 << pin;
                }
            }
        }

        Ok(result)
    }

    /// Read a range of bytes from a ROM set.
    pub async fn read_rom_set_data(
        &mut self,
        parser: &mut Parser<impl Reader>,
        set: u8,
        offset: u32,
        buf: &mut [u8],
    ) -> Result<(), String> {
        let rom_set = self
            .rom_sets
            .get(set as usize)
            .ok_or_else(|| format!("ROM set {} not found", set))?;

        if offset + buf.len() as u32 > rom_set.size {
            return Err("Read extends past ROM set data".into());
        }

        let addr = rom_set.data_ptr + offset;
        parser
            .reader
            .read(addr, buf)
            .await
            .map_err(|_| "Failed to read ROM data".into())
    }

    /// Gets the size of a ROM set in bytes.
    pub fn get_rom_set_size(&self, set: u8) -> Result<usize, String> {
        let rom_set = self
            .rom_sets
            .get(set as usize)
            .ok_or_else(|| format!("ROM set {} not found", set))?;

        Ok(rom_set.size as usize)
    }

    /// Read a single byte from a ROM image at the specified logical address.
    pub async fn read_rom_byte_demangled(
        &mut self,
        parser: &mut Parser<impl Reader>,
        set: u8,
        addr: SdrrAddress,
    ) -> Result<u8, String> {
        let byte = self.read_rom_byte_raw(parser, set, addr).await?;

        self.demangle_byte(byte)
    }

    pub async fn read_rom_byte_raw(
        &mut self,
        parser: &mut Parser<impl Reader>,
        set: u8,
        addr: SdrrAddress,
    ) -> Result<u8, String> {
        let physical_addr = match addr {
            SdrrAddress::Raw(raw_addr) => raw_addr,
            SdrrAddress::Logical(logical_addr) => {
                // Mangle the logical address to get the physical address
                logical_addr.mangle(self)?
            }
        };

        // Read the byte
        let mut buf = [0u8; 1];
        self.read_rom_set_data(parser, set, physical_addr, &mut buf)
            .await?;

        Ok(buf[0])
    }
}

/// Information about a set of ROMs in an SDRR firmware
///
/// If individual ROMs are being servd, there is a set for each ROM image.
/// If multiple ROMs are being served, all ROMs served together are contained
/// in a single set.
///
/// Current maximum number of ROMs in a set is 3.
///
/// Reflects `sdrr_rom_set_t` from `sdrr/include/config_base.h`
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SdrrRomSet {
    /// Pointer to the ROM image data in the firmware.
    pub(crate) data_ptr: u32,

    /// The size of the ROM image data in bytes.
    ///
    /// Currently 16KB for single ROM sets, 64KB for multi-ROM and banked
    /// switched ROM sets.
    pub size: u32,

    /// The ROMs in this set.
    pub roms: Vec<SdrrRomInfo>,

    /// The number of ROMs in this set.
    pub rom_count: u8,

    /// The serving algorithm used for this set.
    pub serve: SdrrServe,

    /// The state of the CS1 line for all ROMs in this set (active low/high/
    /// unused).  Only used for multi-ROM and bank switched sets.
    pub multi_rom_cs1_state: SdrrCsState,
}

/// Information about a single ROM in an SDRR firmware
///
/// Reflects `sdrr_rom_info_t` from `sdrr/include/config_base.h`
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SdrrRomInfo {
    /// The type of the ROM
    pub rom_type: SdrrRomType,

    /// The state of the CS1 line (active low/high/unused)
    pub cs1_state: SdrrCsState,

    /// The state of the CS2 line (active low/high/unused)
    pub cs2_state: SdrrCsState,

    /// The state of the CS3 line (active low/high/unused)
    pub cs3_state: SdrrCsState,

    /// The filename used to create the ROM image (if present in the firmware)
    pub filename: Option<String>,
}

/// SDRR pin configuration
///
/// All pin fields refer to the physical STM32 port pin number.
/// Indexes of arrays/Vecs are the address/data lines (Ax/Dx).
///
/// A pin value of 255 is used to indicate that the pin is not used.
///
/// Reflects `sdrr_pins_t` from `sdrr/include/config_base.h`
#[derive(Debug, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize)]
pub struct SdrrPins {
    pub data_port: SdrrStmPort,
    pub addr_port: SdrrStmPort,
    pub cs_port: SdrrStmPort,
    pub sel_port: SdrrStmPort,
    pub status_port: SdrrStmPort,
    pub rom_pins: u8,
    #[deku(pad_bytes_before = "2")]
    #[deku(count = "8")]
    pub data: Vec<u8>,
    #[deku(count = "16")]
    pub addr: Vec<u8>,
    #[deku(pad_bytes_before = "4")]
    pub cs1_2364: u8,
    pub cs1_2332: u8,
    pub cs1_2316: u8,
    pub cs2_2332: u8,
    pub cs2_2316: u8,
    pub cs3_2316: u8,
    pub x1: u8,
    pub x2: u8,
    pub ce_23128: u8,
    pub oe_23128: u8,
    #[deku(pad_bytes_before = "6")]
    pub sel0: u8,
    pub sel1: u8,
    pub sel2: u8,
    pub sel3: u8,
    #[deku(pad_bytes_before = "4")]
    #[deku(pad_bytes_after = "3")]
    pub status: u8,
}

impl SdrrPins {
    const SDRR_PINS_SIZE: usize = 64;
    pub(crate) const fn size() -> usize {
        // Cannot assert this against SdrrPins size, as contains Vecs, which
        // increase its size.
        Self::SDRR_PINS_SIZE
    }
}
