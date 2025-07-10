// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#![allow(dead_code)]

use std::fmt;

use crate::load::FileType;
use crate::{SDRR_VERSION_MAJOR, SDRR_VERSION_MINOR, SDRR_VERSION_PATCH};

// STM32F4 flash base address
pub const STM32F4_FLASH_BASE: u32 = 0x08000000;

// Hardware revision constants
const HW_DEV_24: u32 = 0x00000000;
const HW_DEV_28: u32 = 0x00000020;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrHwRev {
    None = 0xFFFFFFFF,
    Rev24A = HW_DEV_24 | 0x00,
    Rev24B = HW_DEV_24 | 0x01,
    Rev24C = HW_DEV_24 | 0x02,
    Rev24D = HW_DEV_24 | 0x03,
    Rev24E = HW_DEV_24 | 0x04,
    Rev24F = HW_DEV_24 | 0x05,
    Rev28A = HW_DEV_28 | 0x00,
}

// impl display

impl fmt::Display for SdrrHwRev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrHwRev::None => write!(f, "None"),
            SdrrHwRev::Rev24A => write!(f, "24 pin rev A"),
            SdrrHwRev::Rev24B => write!(f, "24 pin rev B"),
            SdrrHwRev::Rev24C => write!(f, "24 pin rev C"),
            SdrrHwRev::Rev24D => write!(f, "24 pin rev D"),
            SdrrHwRev::Rev24E => write!(f, "24 pin rev E"),
            SdrrHwRev::Rev24F => write!(f, "24 pin rev F"),
            SdrrHwRev::Rev28A => write!(f, "28 pin rev A"),
        }
    }
}

// from u32 for SdrrHwRev
impl SdrrHwRev {
    fn from_u32(value: u32) -> Option<Self> {
        if value == 0xFFFFFFFF {
            Some(SdrrHwRev::None)
        } else if value == (HW_DEV_24 | 0x00) {
            Some(SdrrHwRev::Rev24A)
        } else if value == (HW_DEV_24 | 0x01) {
            Some(SdrrHwRev::Rev24B)
        } else if value == (HW_DEV_24 | 0x02) {
            Some(SdrrHwRev::Rev24C)
        } else if value == (HW_DEV_24 | 0x03) {
            Some(SdrrHwRev::Rev24D)
        } else if value == (HW_DEV_24 | 0x04) {
            Some(SdrrHwRev::Rev24E)
        } else if value == (HW_DEV_24 | 0x05) {
            Some(SdrrHwRev::Rev24F)
        } else if value == (HW_DEV_28 | 0x00) {
            Some(SdrrHwRev::Rev28A)
        } else {
            None
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmLine {
    F401 = 0x0000,
    F405 = 0x0001,
    F411 = 0x0002,
    F446 = 0x0003,
}

impl fmt::Display for StmLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StmLine::F401 => write!(f, "STM32F401"),
            StmLine::F405 => write!(f, "STM32F405"),
            StmLine::F411 => write!(f, "STM32F411"),
            StmLine::F446 => write!(f, "STM32F446"),
        }
    }
}

// from u16 for StmLine
impl StmLine {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0000 => Some(StmLine::F401),
            0x0001 => Some(StmLine::F405),
            0x0002 => Some(StmLine::F411),
            0x0003 => Some(StmLine::F446),
            _ => None,
        }
    }

    pub fn ram_kb(&self) -> &str {
        match self {
            StmLine::F401 => "96",
            StmLine::F405 | StmLine::F411 | StmLine::F446 => "128",
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmStorage {
    Storage8 = 0x00,
    StorageB = 0x01,
    StorageC = 0x02,
    StorageD = 0x03,
    StorageE = 0x04,
    StorageF = 0x05,
    StorageG = 0x06,
}

impl StmStorage {
    pub fn kb(&self) -> &str {
        match self {
            StmStorage::Storage8 => "64",
            StmStorage::StorageB => "128",
            StmStorage::StorageC => "256",
            StmStorage::StorageD => "384",
            StmStorage::StorageE => "512",
            StmStorage::StorageF => "768",
            StmStorage::StorageG => "1024",
        }
    }

    pub fn package_code(&self) -> &str {
        match self {
            StmStorage::Storage8 => "8",
            StmStorage::StorageB => "B",
            StmStorage::StorageC => "C",
            StmStorage::StorageD => "D",
            StmStorage::StorageE => "E",
            StmStorage::StorageF => "F",
            StmStorage::StorageG => "G",
        }
    }
}

impl StmStorage {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(StmStorage::Storage8),
            1 => Some(StmStorage::StorageB),
            2 => Some(StmStorage::StorageC),
            3 => Some(StmStorage::StorageD),
            4 => Some(StmStorage::StorageE),
            5 => Some(StmStorage::StorageF),
            6 => Some(StmStorage::StorageG),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrRomType {
    Rom2316 = 0,
    Rom2332 = 1,
    Rom2364 = 2,
}

impl fmt::Display for SdrrRomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrRomType::Rom2316 => write!(f, "2316"),
            SdrrRomType::Rom2332 => write!(f, "2332"),
            SdrrRomType::Rom2364 => write!(f, "2364"),
        }
    }
}

impl SdrrRomType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SdrrRomType::Rom2316),
            1 => Some(SdrrRomType::Rom2332),
            2 => Some(SdrrRomType::Rom2364),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrCsState {
    ActiveLow = 0,
    ActiveHigh = 1,
    NotUsed = 2,
}

impl fmt::Display for SdrrCsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrCsState::ActiveLow => write!(f, "Active Low"),
            SdrrCsState::ActiveHigh => write!(f, "Active High"),
            SdrrCsState::NotUsed => write!(f, "Not Used"),
        }
    }
}

impl SdrrCsState {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SdrrCsState::ActiveLow),
            1 => Some(SdrrCsState::ActiveHigh),
            2 => Some(SdrrCsState::NotUsed),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrServe {
    TwoCsOneAddr = 0,
    AddrOnCs = 1,
    AddrOnAnyCs = 2,
}

impl fmt::Display for SdrrServe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrServe::TwoCsOneAddr => write!(f, "A = Two CS checks for every address check"),
            SdrrServe::AddrOnCs => write!(f, "B = Check address only when CS active"),
            SdrrServe::AddrOnAnyCs => write!(f, "C = Check address on any CS active"),
        }
    }
}

// from u8 for SdrrServe
impl SdrrServe {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SdrrServe::TwoCsOneAddr),
            1 => Some(SdrrServe::AddrOnCs),
            2 => Some(SdrrServe::AddrOnAnyCs),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrCsPin {
    None = 0,
    Pin18 = 1, // 23xx pin 18, CS2 on 2316
    Pin20 = 2, // 23xx pin 20, CS1 on 2364, 2332 and 2316
    Pin21 = 3, // 23xx pin 21, CS3 on 2316, CS2 on 2332
    PinX1 = 4, // Pin X1 on SDRR, STM32F4 PC14 in hw rev f
    PinX2 = 5, // Pin X2 on SDRR, STM32F4 PC15 in hw rev f
}

impl fmt::Display for SdrrCsPin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrCsPin::None => write!(f, "None"),
            SdrrCsPin::Pin18 => write!(f, "ROM Pin 18"),
            SdrrCsPin::Pin20 => write!(f, "ROM Pin 20"),
            SdrrCsPin::Pin21 => write!(f, "ROM Pin 21"),
            SdrrCsPin::PinX1 => write!(f, "SDRR X1"),
            SdrrCsPin::PinX2 => write!(f, "SDRR X2"),
        }
    }
}

impl SdrrCsPin {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SdrrCsPin::None),
            1 => Some(SdrrCsPin::Pin18),
            2 => Some(SdrrCsPin::Pin20),
            3 => Some(SdrrCsPin::Pin21),
            4 => Some(SdrrCsPin::PinX1),
            5 => Some(SdrrCsPin::PinX2),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrStmPort {
    None = 0x00,
    PortA = 0x01,
    PortB = 0x02,
    PortC = 0x03,
    PortD = 0x04,
}

impl fmt::Display for SdrrStmPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrStmPort::None => write!(f, "None"),
            SdrrStmPort::PortA => write!(f, "A"),
            SdrrStmPort::PortB => write!(f, "B"),
            SdrrStmPort::PortC => write!(f, "C"),
            SdrrStmPort::PortD => write!(f, "D"),
        }
    }
}

impl SdrrStmPort {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(SdrrStmPort::None),
            0x01 => Some(SdrrStmPort::PortA),
            0x02 => Some(SdrrStmPort::PortB),
            0x03 => Some(SdrrStmPort::PortC),
            0x04 => Some(SdrrStmPort::PortD),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SdrrPins {
    pub data_port: SdrrStmPort,
    pub addr_port: SdrrStmPort,
    pub cs_port: SdrrStmPort,
    pub sel_port: SdrrStmPort,
    pub addr: [u8; 16],
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
    pub sel0: u8,
    pub sel1: u8,
    pub sel2: u8,
    pub sel3: u8,
}

// ROM image size constants
pub const ROM_IMAGE_SIZE_2316: usize = 2048;
pub const ROM_IMAGE_SIZE_2332: usize = 4096;
pub const ROM_IMAGE_SIZE_2364: usize = 8192;
pub const ROM_IMAGE_SIZE: usize = 16384;
pub const ROM_SET_IMAGE_SIZE: usize = 65536;

#[repr(C)]
#[derive(Debug)]
pub struct SdrrRomInfo {
    pub rom_type: SdrrRomType,
    pub cs1_state: SdrrCsState,
    pub cs2_state: SdrrCsState,
    pub cs3_state: SdrrCsState,
    pub filename: Option<String>, // Only present with BOOT_LOGGING
}

#[repr(C)]
#[derive(Debug)]
pub struct SdrrRomSet {
    pub data: Vec<u8>,
    pub size: u32,
    pub roms: Vec<SdrrRomInfo>,
    pub rom_count: u8,
    pub serve: SdrrServe,
    pub multi_rom_cs1_state: SdrrCsState,
}

#[repr(C)]
#[derive(Debug)]
pub struct SdrrInfo {
    pub file_type: FileType,
    pub file_size: usize,
    pub magic: [u8; 4],             // Offset: 0
    pub major_version: u16,         // Offset: 4
    pub minor_version: u16,         // Offset: 6
    pub patch_version: u16,         // Offset: 8
    pub build_number: u16,          // Offset: 10
    pub build_date: String,         // Offset: 12 (pointer resolved)
    pub commit: [u8; 8],            // Offset: 16
    pub hw_rev: SdrrHwRev,          // Offset: 24
    pub stm_line: StmLine,          // Offset: 28
    pub stm_storage: StmStorage,    // Offset: 30
    pub freq: u16,                  // Offset: 32
    pub overclock: bool,            // Offset: 34
    pub swd_enabled: bool,          // Offset: 35
    pub preload_image_to_ram: bool, // Offset: 36
    pub bootloader_capable: bool,   // Offset: 37
    pub status_led_enabled: bool,   // Offset: 38
    pub boot_logging_enabled: bool, // Offset: 39
    pub mco_enabled: bool,          // Offset: 40
    pub rom_set_count: u8,          // Offset: 41
    pub rom_sets: Vec<SdrrRomSet>,  // Offset: 44 (pointer resolved)
    pub pins: SdrrPins,             // Offset: 48 (pointer resolved)
}

impl SdrrInfo {
    pub fn from_firmware_bytes(
        file_type: FileType,
        data: &[u8],
        full_firmware: &[u8],
        _base_addr: u32,
        info_offset: usize,
        file_size: usize,
    ) -> Result<Self, String> {
        if data.len() < 48 {
            return Err("Firmware data too small".into());
        }

        // Check magic bytes
        let magic = [data[0], data[1], data[2], data[3]];
        if &magic != b"SDRR" {
            return Err(format!(
                "Invalid magic bytes found at offset 0x{:x}",
                info_offset
            ));
        }

        // Read basic fields (assuming little-endian ARM data)
        let major_version = u16::from_le_bytes([data[4], data[5]]);
        let minor_version = u16::from_le_bytes([data[6], data[7]]);
        let patch_version = u16::from_le_bytes([data[8], data[9]]);
        let build_number = u16::from_le_bytes([data[10], data[11]]);

        // Test against handled versions
        let version_ok = if major_version > SDRR_VERSION_MAJOR {
            false
        } else if minor_version > SDRR_VERSION_MINOR {
            false
        } else if patch_version > SDRR_VERSION_PATCH {
            false
        } else {
            true
        };
        if !version_ok {
            return Err(format!(
                "SDRR firmware version v{}.{}.{} unsupported - max version v{}.{}.{}",
                major_version,
                minor_version,
                patch_version,
                SDRR_VERSION_MAJOR,
                SDRR_VERSION_MINOR,
                SDRR_VERSION_PATCH
            ));
        }

        // Build date pointer (32-bit address)
        let build_date_ptr = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        // Commit hash
        let mut commit = [0u8; 8];
        commit.copy_from_slice(&data[16..24]);

        // Hardware revision
        let hw_rev_val = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let hw_rev = SdrrHwRev::from_u32(hw_rev_val)
            .ok_or_else(|| format!("Unknown hardware revision {}", hw_rev_val))?;

        // STM line and storage
        let stm_line_val = u16::from_le_bytes([data[28], data[29]]);
        let stm_line = StmLine::from_u16(stm_line_val)
            .ok_or_else(|| format!("Unknown STM line {}", stm_line_val))?;

        let stm_storage_val = u16::from_le_bytes([data[30], data[31]]);
        let stm_storage = StmStorage::from_u16(stm_storage_val)
            .ok_or_else(|| format!("Unknown STM storage {}", stm_storage_val))?;

        // Frequency and flags
        let freq = u16::from_le_bytes([data[32], data[33]]);
        let overclock = data[34];
        let swd_enabled = data[35];
        let preload_image_to_ram = data[36];
        let bootloader_capable = data[37];
        let status_led_enabled = data[38];
        let boot_logging_enabled = data[39];
        let mco_enabled = data[40];

        // ROM set info
        let rom_set_count = data[41];
        // data[41..44] is padding
        let rom_sets_ptr = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);

        // Resolve build date string
        let build_date =
            Self::read_string_at_ptr(full_firmware, build_date_ptr, STM32F4_FLASH_BASE)?;

        // Parse ROM sets
        let rom_sets = Self::read_rom_sets(
            full_firmware,
            rom_sets_ptr,
            rom_set_count,
            STM32F4_FLASH_BASE,
            boot_logging_enabled,
        )?;

        // Parse pins if present (at offset 48 from structure start)
        let pins_ptr = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
        let pins = Self::read_pins_at_ptr(full_firmware, pins_ptr, STM32F4_FLASH_BASE)?;

        Ok(SdrrInfo {
            file_type,
            file_size,
            magic,
            major_version,
            minor_version,
            patch_version,
            build_number,
            build_date,
            commit,
            hw_rev,
            stm_line,
            stm_storage,
            freq,
            overclock: overclock != 0,
            swd_enabled: swd_enabled != 0,
            preload_image_to_ram: preload_image_to_ram != 0,
            bootloader_capable: bootloader_capable != 0,
            status_led_enabled: status_led_enabled != 0,
            boot_logging_enabled: boot_logging_enabled != 0,
            mco_enabled: mco_enabled != 0,
            rom_set_count,
            rom_sets,
            pins,
        })
    }

    fn read_string_at_ptr(data: &[u8], ptr: u32, base_addr: u32) -> Result<String, String> {
        if ptr < base_addr {
            return Err("Invalid pointer".into());
        }

        let offset = (ptr - base_addr) as usize;
        if offset >= data.len() {
            return Err("Pointer out of bounds".into());
        }

        // Find null terminator
        let end = data[offset..]
            .iter()
            .position(|&b| b == 0)
            .ok_or("Unterminated string".to_string())?;

        let string_bytes = &data[offset..offset + end];
        String::from_utf8(string_bytes.to_vec()).map_err(|_| "Invalid UTF-8 string".into())
    }

    fn read_rom_sets(
        data: &[u8],
        ptr: u32,
        count: u8,
        base_addr: u32,
        boot_logging_enabled: u8,
    ) -> Result<Vec<SdrrRomSet>, String> {
        if ptr < base_addr || count == 0 {
            return Ok(Vec::new());
        }

        let offset = (ptr - base_addr) as usize;
        let mut rom_sets = Vec::new();

        // Each sdrr_rom_set_t is 16 bytes (with padding)
        const ROM_SET_SIZE: usize = 16;

        for i in 0..count {
            let set_offset = offset + (i as usize * ROM_SET_SIZE);
            if set_offset + ROM_SET_SIZE > data.len() {
                return Err("ROM set data out of bounds".into());
            }

            let set_data = &data[set_offset..set_offset + ROM_SET_SIZE];

            // Parse sdrr_rom_set_t structure
            let data_ptr = u32::from_le_bytes([set_data[0], set_data[1], set_data[2], set_data[3]]);
            let size = u32::from_le_bytes([set_data[4], set_data[5], set_data[6], set_data[7]]);
            let roms_ptr =
                u32::from_le_bytes([set_data[8], set_data[9], set_data[10], set_data[11]]);
            let rom_count = set_data[12];
            let serve = SdrrServe::from_u8(set_data[13])
                .ok_or_else(|| format!("Invalid serve algorithm {}", set_data[13]))?;
            let multi_rom_cs1_state = SdrrCsState::from_u8(set_data[14])
                .ok_or_else(|| format!("Invalid multi-ROM CS1 state {}", set_data[14]))?;

            // Read ROM data
            let rom_data = if data_ptr >= base_addr {
                let data_offset = (data_ptr - base_addr) as usize;
                if data_offset + size as usize <= data.len() {
                    data[data_offset..data_offset + size as usize].to_vec()
                } else {
                    return Err("ROM data out of bounds".into());
                }
            } else {
                Vec::new()
            };

            // Read ROM info structures
            let roms =
                Self::read_rom_infos(data, roms_ptr, rom_count, base_addr, boot_logging_enabled)?;

            let rom_set = SdrrRomSet {
                data: rom_data,
                size,
                roms,
                rom_count,
                serve,
                multi_rom_cs1_state,
            };

            rom_sets.push(rom_set);
        }

        Ok(rom_sets)
    }

    fn read_rom_infos(
        data: &[u8],
        ptr: u32,
        count: u8,
        base_addr: u32,
        boot_logging_enabled: u8,
    ) -> Result<Vec<SdrrRomInfo>, String> {
        if ptr < base_addr || count == 0 {
            return Ok(Vec::new());
        }

        let offset = (ptr - base_addr) as usize;
        let mut rom_infos = Vec::new();

        // Array of pointers to sdrr_rom_info_t (4 bytes each)
        for i in 0..count {
            let ptr_offset = offset + (i as usize * 4);
            if ptr_offset + 4 > data.len() {
                return Err("ROM info pointer out of bounds".into());
            }

            let rom_info_ptr = u32::from_le_bytes([
                data[ptr_offset],
                data[ptr_offset + 1],
                data[ptr_offset + 2],
                data[ptr_offset + 3],
            ]);

            if rom_info_ptr < base_addr {
                return Err("Invalid ROM info pointer".into());
            }

            let info_offset = (rom_info_ptr - base_addr) as usize;

            // sdrr_rom_info_t structure size depends on BOOT_LOGGING
            let rom_info_size = if boot_logging_enabled != 0 {
                8
            } else {
                4
            };

            if info_offset + rom_info_size > data.len() {
                return Err("ROM info data out of bounds".into());
            }

            let info_data = &data[info_offset..info_offset + rom_info_size];

            let rom_type = SdrrRomType::from_u8(info_data[0])
                .ok_or_else(|| format!("Invalid ROM type {}", info_data[0]))?;

            let cs1_state = SdrrCsState::from_u8(info_data[1])
                .ok_or_else(|| format!("Invalid CS1 state {}", info_data[1]))?;
            let cs2_state = SdrrCsState::from_u8(info_data[2])
                .ok_or_else(|| format!("Invalid CS2 state {}", info_data[2]))?;
            let cs3_state = SdrrCsState::from_u8(info_data[3])
                .ok_or_else(|| format!("Invalid CS3 state {}", info_data[3]))?;

            let filename = if boot_logging_enabled != 0 {
                // Read filename if BOOT_LOGGING is enabled
                let filename_ptr =
                    u32::from_le_bytes([info_data[4], info_data[5], info_data[6], info_data[7]]);

                let filename = if filename_ptr >= base_addr {
                    Self::read_string_at_ptr(data, filename_ptr, base_addr).ok()
                } else {
                    None
                };

                filename
            } else {
                None
            };

            let rom_info = SdrrRomInfo {
                rom_type,
                cs1_state,
                cs2_state,
                cs3_state,
                filename,
            };

            rom_infos.push(rom_info);
        }

        Ok(rom_infos)
    }

    pub fn is_hw_rev_f(&self) -> bool {
        matches!(self.hw_rev, SdrrHwRev::Rev24F)
    }

    pub fn demangle_byte(&self, byte: u8) -> u8 {
        match self.hw_rev {
            SdrrHwRev::Rev24D | SdrrHwRev::Rev24E | SdrrHwRev::Rev24F | SdrrHwRev::Rev28A => {
                // Bit 0 -> 7
                // Bit 1 -> 6
                // Bit 2 -> 5
                // Bit 3 -> 4
                // Bit 4 -> 3
                // Bit 5 -> 2
                // Bit 6 -> 1
                // Bit 7 -> 0
                byte.reverse_bits()
            }
            _ => {
                panic!(
                    "Unsupported hardware revision for demangling: {}",
                    self.hw_rev
                );
            }
        }
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    pub fn mangle_address(
        &self,
        addr: u32,
        cs1: bool,
        cs2: Option<bool>,
        c3: Option<bool>,
        x1: Option<bool>,
        x2: Option<bool>,
    ) -> u32 {
        if self.hw_rev != SdrrHwRev::Rev24D
            && self.hw_rev != SdrrHwRev::Rev24E
            && self.hw_rev != SdrrHwRev::Rev24F
        {
            panic!("Mangle address is only supported for hardware revisions 24-D, 24-E and 24-F");
        }

        let mut pin_to_addr_map = [
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
            None,
            Some(10),
            Some(11),
            Some(9),
            None,
            None,
        ];

        let num_roms = self.rom_sets[0].rom_count as usize;
        if num_roms > 1 {
            // X1 and X2 pins
            pin_to_addr_map[14] = Some(14);
            pin_to_addr_map[15] = Some(15);
        }

        let rom_type = self.rom_sets[0].roms[0].rom_type;
        let addr_mask = match rom_type {
            SdrrRomType::Rom2364 => {
                pin_to_addr_map[10] = Some(13);
                0x1FFF // 13-bit address
            }
            SdrrRomType::Rom2332 => {
                pin_to_addr_map[10] = Some(13);
                pin_to_addr_map[9] = Some(12);
                0x0FFF // 12-bit address
            }
            SdrrRomType::Rom2316 => {
                pin_to_addr_map[10] = Some(13);
                pin_to_addr_map[9] = Some(11);
                pin_to_addr_map[12] = Some(12);
                0x07FF // 11-bit address
            }
        };

        let overflow = addr & !addr_mask;
        if overflow != 0 {
            panic!(
                "Requested Address 0x{:08X} overflows the address space for ROM type {}",
                addr, rom_type
            );
        }

        let mut input_addr = addr & addr_mask;
        match rom_type {
            SdrrRomType::Rom2364 => {
                if cs1 {
                    input_addr |= 1 << 13; // Set CS1 bit for 2364
                }
            }
            SdrrRomType::Rom2332 => {
                if cs1 {
                    input_addr |= 1 << 13; // Set CS1 bit for 2332
                }
                if let Some(cs2) = cs2 {
                    if cs2 {
                        input_addr |= 1 << 12; // Set CS2 bit for 2332
                    }
                }
            }
            SdrrRomType::Rom2316 => {
                if cs1 {
                    input_addr |= 1 << 13; // Set CS1 bit for 2316
                }
                if let Some(cs2) = cs2 {
                    if cs2 {
                        input_addr |= 1 << 12; // Set CS2 bit for 2316
                    }
                }
                if let Some(c3) = c3 {
                    if c3 {
                        input_addr |= 1 << 11; // Set CS3 bit for 2316
                    }
                }
            }
        };

        if num_roms > 1 {
            // Handle X1 and X2 pins
            if let Some(x1) = x1 {
                if x1 {
                    input_addr |= 1 << 14; // Set X1 pin
                }
            }
            if let Some(x2) = x2 {
                if x2 {
                    input_addr |= 1 << 15; // Set X2 pin
                }
            }
        }

        // Apply the pin mapping
        let mut result = 0;
        for pin in 0..pin_to_addr_map.len() {
            if let Some(addr_bit) = pin_to_addr_map[pin] {
                // Check if this address bit is set in the input address
                if (input_addr & (1 << addr_bit)) != 0 {
                    // Set the corresponding pin in the result
                    result |= 1 << pin;
                }
            }
        }

        result
    }

    pub fn get_rom_set_image(&self, set: u8) -> Option<&[u8]> {
        self.rom_sets
            .get(set as usize)
            .map(|rom_set| rom_set.data.as_slice())
    }

    fn read_pins_at_ptr(data: &[u8], ptr: u32, base_addr: u32) -> Result<SdrrPins, String> {
        let offset = (ptr - base_addr) as usize;
        if offset + 52 > data.len() {
            return Err("Pins data out of bounds".into());
        }
        
        Self::read_pins(&data[offset..offset + 52])
    }

    fn read_pins(data: &[u8]) -> Result<SdrrPins, String> {
        if data.len() < 52 {
            return Err("Pins data too small".into());
        }

        let data_port = SdrrStmPort::from_u8(data[0])
            .ok_or_else(|| format!("Invalid data port {}", data[0]))?;
        let addr_port = SdrrStmPort::from_u8(data[1])
            .ok_or_else(|| format!("Invalid addr port {}", data[1]))?;
        let cs_port = SdrrStmPort::from_u8(data[2])
            .ok_or_else(|| format!("Invalid cs port {}", data[2]))?;
        let sel_port = SdrrStmPort::from_u8(data[3])
            .ok_or_else(|| format!("Invalid sel port {}", data[3]))?;

        let mut addr = [0u8; 16];
        addr.copy_from_slice(&data[8..24]);

        Ok(SdrrPins {
            data_port,
            addr_port,
            cs_port,
            sel_port,
            addr,
            cs1_2364: data[28],
            cs1_2332: data[29],
            cs1_2316: data[30],
            cs2_2332: data[31],
            cs2_2316: data[32],
            cs3_2316: data[33],
            x1: data[34],
            x2: data[35],
            ce_23128: data[36],
            oe_23128: data[37],
            sel0: data[44],
            sel1: data[45],
            sel2: data[46],
            sel3: data[47],
        })
    }
}
