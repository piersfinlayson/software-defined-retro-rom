// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-fw-parser
//!
//! Parses [Software Defined Retro ROM](https://piers.rocks/u/sdrr) (SDRR)
//! firmware.
//! 
//! This is a `no_std` compatible library, which can be used in both `std` and
//! `no_std` environments and can read and extract information from SDRR
//! firmware - either from
//! - a binary file
//! - an ELF file
//! - raw bytes, e.g. from bytes read directly from a device's flash or RAM
//! 
//! This is used directly within the SDRR repository, by:
//! - [`sdrr-info`](https://github.com/piersfinlayson/software-defined-retro-rom/blob/main/rust/sdrr-info/README.md) -
//!   PC based tool to analyse SDRR firmware source code
//! 
//! It can also be used by external tools.
//! 
//! Typically used like this:
//! 
//! ```rust ignore
//! use sdrr_fw_parser::{SdrrInfo, SdrrFileType};
//! let sdrr_info = SdrrInfo::from_firmware_bytes(
//!     SdrrFileType::Elf,
//!     &sdrr_info, // Reference to sdrr_info_t from firmware file 
//!     &full_fw,   // Reference to full firmware data
//!     file_size   // Size of the full firmware file in bytes
//! );
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

/// Maximum SDRR firmware versions supported by this version of`sdrr-fw-parser`
pub const MAX_VERSION_MAJOR: u16 = 0;
pub const MAX_VERSION_MINOR: u16 = 2;
pub const MAX_VERSION_PATCH: u16 = 1;

// Use alloc if no-std.
#[cfg(not(feature = "std"))]
extern crate alloc;

use core::fmt;
use deku::prelude::*;

// Use std/no-std String and Vec types
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

// Used to check various data structure sizes
use static_assertions::const_assert_eq;

/// Supports SDRR firmware file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrFileType {
    /// A .elf file
    Elf,

    /// A .bin file
    Orc,
}

impl core::fmt::Display for SdrrFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrFileType::Elf => write!(f, "ELF (.elf)"),
            SdrrFileType::Orc => write!(f, "Binary (.bin)"),
        }
    }
}

// STM32F4 flash base address.  Required to find offset from pointers
const STM32F4_FLASH_BASE: u32 = 0x08000000;

// ROM image size constants
const ROM_IMAGE_SIZE: usize = 16384;
const ROM_SET_IMAGE_SIZE: usize = 65536;

/// STM32F4 product line options
/// 
/// Relflects `stm_line_t` from `sdrr/include/config_base.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(id_type = "u16", ctx = "endian: deku::ctx::Endian")]
pub enum StmLine {
    /// F401D/E - 96KB RAM
    #[deku(id = "0x0000")]
    F401DE,

    /// F405
    #[deku(id = "0x0001")]
    F405,

    /// F411
    #[deku(id = "0x0002")]
    F411,

    /// F446
    #[deku(id = "0x0003")]
    F446,

    /// F401B/C - 64KB RAM
    #[deku(id = "0x0004")]
    F401BC,
}

impl fmt::Display for StmLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StmLine::F401DE => write!(f, "STM32F401DE"),
            StmLine::F405 => write!(f, "STM32F405"),
            StmLine::F411 => write!(f, "STM32F411"),
            StmLine::F446 => write!(f, "STM32F446"),
            StmLine::F401BC => write!(f, "STM32F401BC"),
        }
    }
}

impl StmLine {
    /// Returns the amount of SRAM of the device (not including any CCM RAM)
    pub fn ram_kb(&self) -> &str {
        match self {
            StmLine::F401DE => "96",
            StmLine::F401BC => "64",
            StmLine::F405 | StmLine::F411 | StmLine::F446 => "128",
        }
    }
}

/// STM32F4 package flash storage code
///
/// For example "E" in STM32F401RET6 means 512KB of flash storage.
/// 
/// Reflects `stm_storage_t` from `sdrr/include/config_base.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(id_type = "u16", ctx = "endian: deku::ctx::Endian")]
pub enum StmStorage {
    /// 8 = 64KB
    #[deku(id = "0")]
    Storage8,

    /// B = 128KB
    #[deku(id = "1")]
    StorageB,

    /// C = 256KB
    #[deku(id = "2")]
    StorageC,

    /// D = 384KB
    #[deku(id = "3")]
    StorageD,

    /// E = 512KB
    #[deku(id = "4")]
    StorageE,

    /// F = 768KB
    #[deku(id = "5")]
    StorageF,

    /// G = 1024KB
    #[deku(id = "6")]
    StorageG,
}

impl StmStorage {
    /// Returns the storage size in kilobytes
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

    /// Returns the storage package code
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

/// Type of ROMs supported by SDRR
/// 
/// Reflects `sdrr_rom_type_t` from `sdrr/include/config_base.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum SdrrRomType {
    /// 2316 ROM, 11-bit address, 3 CS lines, 2KB size
    #[deku(id = "0")]
    Rom2316,

    /// 2332 ROM, 12-bit address, 2 CS lines, 4KB size
    #[deku(id = "1")]
    Rom2332,

    /// 2364 ROM, 13-bit address, 1 CS line, 8KB size
    #[deku(id = "2")]
    Rom2364,
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
    /// Returns the size of the ROM in bytes
    pub fn rom_size(&self) -> usize {
        self.rom_size_kb() * 1024
    }

    /// Returns the size of the ROM in KB
    pub fn rom_size_kb(&self) -> usize {
        match self {
            SdrrRomType::Rom2316 => 2,
            SdrrRomType::Rom2332 => 4,
            SdrrRomType::Rom2364 => 8,
        }
    }

    /// Returns the maximum addressable location in the ROM
    pub fn max_addr(&self) -> u32 {
        (self.rom_size() - 1) as u32
    }

    /// Checks if the ROM type supports the CS2 line
    pub fn supports_cs2(&self) -> bool {
        match self {
            SdrrRomType::Rom2316 => true,
            SdrrRomType::Rom2332 => true,
            SdrrRomType::Rom2364 => false,
        }
    }

    /// Checks if the ROM type supports the CS3 line
    pub fn supports_cs3(&self) -> bool {
        match self {
            SdrrRomType::Rom2316 => true,
            SdrrRomType::Rom2332 => false,
            SdrrRomType::Rom2364 => false,
        }
    }
}

/// SDRR chip select active options
/// 
/// Reflects `sdrr_cs_state_t` from `sdrr/include/config_base.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum SdrrCsState {
    /// Chip select line is active low
    #[deku(id = "0")]
    ActiveLow,

    /// Chip select line is active high
    #[deku(id = "1")]
    ActiveHigh,

    /// Chip select line is not used
    #[deku(id = "2")]
    NotUsed,
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

/// SDRR serving algorithm options
/// 
/// Reflects `sdrr_serve_t` from `sdrr/include/config_base.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum SdrrServe {
    /// Original algorithm - two CS checks for every address check, checks
    /// while CS is inactive as well as active
    #[deku(id = "0")]
    TwoCsOneAddr,

    /// Default algorithm from v0.2.1 as it is more performant in every case
    /// tested - checks address only on CS active, and at same frequency
    #[deku(id = "1")]
    AddrOnCs,

    /// Multi-ROM set algorithm - as `AddrOnCs`, but ROM is considered active
    /// when _any_ ROM's CS line is active
    #[deku(id = "2")]
    AddrOnAnyCs,
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

/// SDRR STM32 port options
/// 
/// Reflects `sdrr_stm_port_t` from `sdrr/include/config_base.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum SdrrStmPort {
    /// No port (pin set is not exposed/used)
    #[deku(id = "0x00")]
    None,

    /// Port A
    #[deku(id = "0x01")]
    PortA,

    /// Port B
    #[deku(id = "0x02")]
    PortB,

    /// Port C
    #[deku(id = "0x03")]
    PortC,

    /// Port D
    #[deku(id = "0x04")]
    PortD,
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

// Cannot assert this against SdrrPins size, as contains Vecs, which increase
// its size.
const SDRR_PINS_SIZE: usize = 64;

/// SDRR pin configuration
///
/// All pin fields refer to the physical STM32 port pin number.
/// Indexes of arrays/Vecs are the address/data lines (Ax/Dx).
///
/// A pin value of 255 is used to indicate that the pin is not used.
/// 
/// Reflects `sdrr_pins_t` from `sdrr/include/config_base.h`
#[derive(Debug, DekuRead, DekuWrite)]
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

const ROM_INFO_BASIC_SIZE: usize = 4;
const_assert_eq!(
    core::mem::size_of::<SdrrRomInfoBasic>(),
    ROM_INFO_BASIC_SIZE
);

// Contains information about a specific ROM image
// 
// This version is used when the BOOT_LOGGING is not defined in the C code
// 
// Reflects `sdrr_rom_info_t` from `sdrr/include/config_base.h`
//
// Only used internally
#[derive(Debug, DekuRead, DekuWrite)]
struct SdrrRomInfoBasic {
    pub rom_type: SdrrRomType,
    pub cs1_state: SdrrCsState,
    pub cs2_state: SdrrCsState,
    pub cs3_state: SdrrCsState,
}

const ROM_INFO_WITH_LOGGING_SIZE: usize = 8;
const_assert_eq!(
    core::mem::size_of::<SdrrRomInfoWithLogging>(),
    ROM_INFO_WITH_LOGGING_SIZE
);

// Contains information about a specific ROM image
// 
// This version is used when the BOOT_LOGGING is defined in the C code
// 
// Reflects `sdrr_rom_info_t` from `sdrr/include/config_base.h`
//
// Only used internally
#[derive(Debug, DekuRead, DekuWrite)]
struct SdrrRomInfoWithLogging {
    pub rom_type: SdrrRomType,
    pub cs1_state: SdrrCsState,
    pub cs2_state: SdrrCsState,
    pub cs3_state: SdrrCsState,
    #[deku(endian = "little")]
    pub filename_ptr: u32,
}

/// Information about a single ROM in an SDRR firmware
/// 
/// Reflects `sdrr_rom_info_t` from `sdrr/include/config_base.h`
#[derive(Debug)]
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

const ROM_SET_HEADER_SIZE: usize = 16;
const_assert_eq!(
    ROM_SET_HEADER_SIZE,
    core::mem::size_of::<SdrrRomSetHeader>()
);
// Information about a specific ROM set
//
// Reflects `sdrr_rom_set_info_t` from `sdrr/include/config_base.h`
//
// Used internally to construct [`SdrrRomSet`]
#[derive(Debug, DekuRead, DekuWrite)]
struct SdrrRomSetHeader {
    #[deku(endian = "little")]
    pub data_ptr: u32,
    #[deku(endian = "little")]
    pub size: u32,
    #[deku(endian = "little")]
    pub roms_ptr: u32,
    pub rom_count: u8,
    pub serve: SdrrServe,
    #[deku(pad_bytes_after = "1")]
    pub multi_rom_cs1_state: SdrrCsState,
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
#[derive(Debug)]
pub struct SdrrRomSet {
    /// The ROM image data for this set - this is post-processed, so as stored
    /// in the firmware, and used by the firmware to serve data bytes.
    ///
    /// To look up bytes in this using a true address, use
    /// [`SdrrInfo::mangle_address()`].  Use [`SdrrInfo::demangle_byte()]` to
    /// convert the byte read from this data into a logical byte
    pub data: Vec<u8>,

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

// Cannot assert this against SdrrInfoHeader size, as contains Vecs, which
// increase its size.
const SDRR_INFO_HEADER_SIZE: usize = 56;
#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "little", magic = b"SDRR")]
// Used internally to construct [`SdrrInfo`]
//
// Reflects `sdrr_info_t` from `sdrr/include/config_base.h`
struct SdrrInfoHeader {
    #[deku(endian = "little")]
    pub major_version: u16,
    #[deku(endian = "little")]
    pub minor_version: u16,
    #[deku(endian = "little")]
    pub patch_version: u16,
    #[deku(endian = "little")]
    pub build_number: u16,
    #[deku(endian = "little")]
    pub build_date_ptr: u32,
    #[deku(bytes = "8")]
    pub commit: [u8; 8],
    #[deku(endian = "little")]
    pub hw_rev_ptr: u32,
    pub stm_line: StmLine,
    pub stm_storage: StmStorage,
    #[deku(endian = "little")]
    pub freq: u16,
    pub overclock: u8,
    pub swd_enabled: u8,
    pub preload_image_to_ram: u8,
    pub bootloader_capable: u8,
    pub status_led_enabled: u8,
    pub boot_logging_enabled: u8,
    pub mco_enabled: u8,
    pub rom_set_count: u8,
    #[deku(pad_bytes_before = "2")]
    #[deku(endian = "little")]
    pub rom_sets_ptr: u32,
    #[deku(endian = "little")]
    pub pins_ptr: u32,
    #[deku(bytes = "4")]
    pub boot_config: [u8; 4],
}

/// Main SDRR firmware information data structure.  Contains all data parsed
/// from the firmware file.
/// 
/// Reflects `sdrr_info_t` from `sdrr/include/config_base.h`
#[derive(Debug)]
pub struct SdrrInfo {
    pub file_type: SdrrFileType,
    pub file_size: usize,
    pub major_version: u16,
    pub minor_version: u16,
    pub patch_version: u16,
    pub build_number: u16,
    pub build_date: String,
    pub commit: [u8; 8],
    pub hw_rev: String,
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
    pub rom_sets: Vec<SdrrRomSet>,
    pub pins: SdrrPins,
    pub boot_config: [u8; 4],
}

impl SdrrInfo {
    /// Creates a new `SdrrInfo` instance from the firmware bytes.
    ///
    /// Arguments:
    /// * `file_type` - The type of the firmware file, ELF or Orc (Binary)
    /// * `data` - Reference to the sdrr_info structure bytes
    /// * `full_firmware` - The full firmware binary data, from its start
    /// * `file_size` - The size of the full firmware file in bytes
    pub fn from_firmware_bytes(
        file_type: SdrrFileType,
        data: &[u8],
        full_firmware: &[u8],
        file_size: usize,
    ) -> Result<Self, String> {
        // Parse and validate header
        let header = Self::parse_header(data)?;
        Self::validate_version(&header)?;

        // Resolve string pointers
        let build_date =
            read_string_at_ptr(full_firmware, header.build_date_ptr, STM32F4_FLASH_BASE)?;
        let hw_rev = read_string_at_ptr(full_firmware, header.hw_rev_ptr, STM32F4_FLASH_BASE)?;

        // Parse ROM sets and pins
        let rom_sets = read_rom_sets(
            full_firmware,
            header.rom_sets_ptr,
            header.rom_set_count,
            STM32F4_FLASH_BASE,
            header.boot_logging_enabled != 0,
        )?;

        let pins =
            SdrrPins::from_firmware_data(full_firmware, header.pins_ptr, STM32F4_FLASH_BASE)?;

        // Build final structure
        Ok(SdrrInfo {
            file_type,
            file_size,
            major_version: header.major_version,
            minor_version: header.minor_version,
            patch_version: header.patch_version,
            build_number: header.build_number,
            build_date,
            commit: header.commit,
            hw_rev,
            stm_line: header.stm_line,
            stm_storage: header.stm_storage,
            freq: header.freq,
            overclock: header.overclock != 0,
            swd_enabled: header.swd_enabled != 0,
            preload_image_to_ram: header.preload_image_to_ram != 0,
            bootloader_capable: header.bootloader_capable != 0,
            status_led_enabled: header.status_led_enabled != 0,
            boot_logging_enabled: header.boot_logging_enabled != 0,
            mco_enabled: header.mco_enabled != 0,
            rom_set_count: header.rom_set_count,
            rom_sets,
            pins,
            boot_config: header.boot_config,
        })
    }

    fn parse_header(data: &[u8]) -> Result<SdrrInfoHeader, String> {
        if data.len() < SDRR_INFO_HEADER_SIZE {
            return Err("Firmware data too small".into());
        }

        let header_data = &data[0..SDRR_INFO_HEADER_SIZE];
        SdrrInfoHeader::from_bytes((header_data, 0))
            .map_err(|e| format!("Failed to parse header: {}", e))
            .map(|(_, header)| header)
    }

    fn validate_version(header: &SdrrInfoHeader) -> Result<(), String> {
        let version_ok = header.major_version <= MAX_VERSION_MAJOR
            && header.minor_version <= MAX_VERSION_MINOR
            && header.patch_version <= MAX_VERSION_PATCH;

        if !version_ok {
            return Err(format!(
                "SDRR firmware version v{}.{}.{} unsupported - max version v{}.{}.{}",
                header.major_version,
                header.minor_version,
                header.patch_version,
                MAX_VERSION_MAJOR,
                MAX_VERSION_MINOR,
                MAX_VERSION_PATCH
            ));
        }
        Ok(())
    }

    /// Demangles a byte from the physical pin representation to the logical
    /// representation which is served on D0-D7.  Use when looking up a byte
    /// from the ROM image data to get the "real" byte.
    pub fn demangle_byte(&self, byte: u8) -> u8 {
        assert!(self.pins.data.len() == 8, "Expected 8 data pins");
        let mut result = 0u8;
        for (logic_bit, &phys_pin) in self.pins.data.iter().enumerate() {
            assert!(phys_pin < 8, "Physical pin {} out of range", phys_pin);
            if (byte & (1 << phys_pin)) != 0 {
                result |= 1 << logic_bit;
            }
        }
        result
    }

    /// Takes a logical address and all chip select line states, and produces
    /// a mangled address, as the firmware uses to lookup a byte in the ROM
    /// image stored in firmware.  Use to get the address to index into the
    /// ROM data stored in the firmware, and then use `demangle_byte()` to
    /// turn into a logical byte.
    #[allow(unused_variables)]
    pub fn mangle_address(
        &self,
        addr: u32,
        cs1: bool,
        cs2: Option<bool>,
        cs3: Option<bool>,
        x1: Option<bool>,
        x2: Option<bool>,
    ) -> Result<u32, String> {
        let mut pin_to_addr_map = [None; 16];
        assert!(self.pins.addr.len() <= 16, "Expected up to 16 address pins");
        for (addr_bit, &phys_pin) in self.pins.addr.iter().enumerate() {
            if phys_pin < 16 {
                pin_to_addr_map[phys_pin as usize] = Some(addr_bit);
            }
        }

        let num_roms = self.rom_sets[0].rom_count as usize;
        if num_roms > 1 {
            assert!(
                self.pins.x1 < 16 && self.pins.x2 < 16,
                "X1 and X2 pins must be less than 16"
            );
            assert!(
                pin_to_addr_map[self.pins.x1 as usize].is_none()
                    && pin_to_addr_map[self.pins.x2 as usize].is_none(),
                "X1 and X2 pins must not overlap with other address pins"
            );
            pin_to_addr_map[self.pins.x1 as usize] = Some(14);
            pin_to_addr_map[self.pins.x2 as usize] = Some(15);
        }

        let rom_type = self.rom_sets[0].roms[0].rom_type;
        let addr_mask = match rom_type {
            SdrrRomType::Rom2364 => {
                assert!(
                    self.pins.cs1_2364 < 16,
                    "CS1 pin for 2364 must be less than 16"
                );
                pin_to_addr_map[self.pins.cs1_2364 as usize] = Some(13);
                0x1FFF // 13-bit address
            }
            SdrrRomType::Rom2332 => {
                assert!(
                    self.pins.cs1_2332 < 16,
                    "CS1 pin for 2332 must be less than 16"
                );
                assert!(
                    self.pins.cs2_2332 < 16,
                    "CS2 pin for 2332 must be less than 16"
                );
                pin_to_addr_map[self.pins.cs1_2332 as usize] = Some(13);
                pin_to_addr_map[self.pins.cs2_2332 as usize] = Some(12);
                0x0FFF // 12-bit address
            }
            SdrrRomType::Rom2316 => {
                assert!(
                    self.pins.cs1_2316 < 16,
                    "CS1 pin for 2316 must be less than 16"
                );
                assert!(
                    self.pins.cs2_2316 < 16,
                    "CS2 pin for 2316 must be less than 16"
                );
                assert!(
                    self.pins.cs3_2316 < 16,
                    "CS3 pin for 2316 must be less than 16"
                );
                pin_to_addr_map[self.pins.cs1_2316 as usize] = Some(13);
                pin_to_addr_map[self.pins.cs2_2316 as usize] = Some(11);
                pin_to_addr_map[self.pins.cs3_2316 as usize] = Some(12);
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
        for pin in 0..pin_to_addr_map.len() {
            if let Some(addr_bit) = pin_to_addr_map[pin] {
                if (input_addr & (1 << addr_bit)) != 0 {
                    result |= 1 << pin;
                }
            }
        }

        Ok(result)
    }

    /// Helper function to get a pointer to the appropriate ROM set image data.
    /// This returns the entire ROM set data - addresses and data are mangled
    /// in this data.
    pub fn get_rom_set_image(&self, set: u8) -> Option<&[u8]> {
        self.rom_sets
            .get(set as usize)
            .map(|rom_set| rom_set.data.as_slice())
    }
}

impl SdrrRomSet {
    fn from_header_and_firmware_data(
        header: SdrrRomSetHeader,
        data: &[u8],
        base_addr: u32,
        boot_logging_enabled: bool,
    ) -> Result<Self, String> {
        // Read ROM data using the header
        let rom_data = if header.data_ptr >= base_addr {
            let data_offset = (header.data_ptr - base_addr) as usize;
            let data_end = data_offset.saturating_add(header.size as usize);

            if data_end <= data.len() {
                data[data_offset..data_end].to_vec()
            } else {
                return Err(format!(
                    "ROM data out of bounds: offset={}, size={}, data_len={}",
                    data_offset,
                    header.size,
                    data.len()
                ));
            }
        } else {
            Vec::new()
        };

        // Read ROM infos using the type-specific method
        let roms = SdrrRomInfo::read_multiple_from_firmware_data(
            data,
            header.roms_ptr,
            header.rom_count,
            base_addr,
            boot_logging_enabled,
        )?;

        Ok(SdrrRomSet {
            data: rom_data,
            size: header.size,
            roms,
            rom_count: header.rom_count,
            serve: header.serve,
            multi_rom_cs1_state: header.multi_rom_cs1_state,
        })
    }
}

impl SdrrRomInfo {
    // Handle the Vec iteration for multiple rom infos
    fn read_multiple_from_firmware_data(
        data: &[u8],
        ptr: u32,
        count: u8,
        base_addr: u32,
        boot_logging_enabled: bool,
    ) -> Result<Vec<SdrrRomInfo>, String> {
        if ptr < base_addr || count == 0 {
            return Ok(Vec::new());
        }

        let offset = (ptr - base_addr) as usize;
        let mut rom_infos = Vec::with_capacity(count as usize);

        for i in 0..count {
            let ptr_offset = offset + (i as usize * 4);

            if ptr_offset + 4 > data.len() {
                return Err(format!("ROM info pointer {} out of bounds", i));
            }

            let rom_info_ptr = u32::from_le_bytes([
                data[ptr_offset],
                data[ptr_offset + 1],
                data[ptr_offset + 2],
                data[ptr_offset + 3],
            ]);

            // Delegate to single rom info constructor
            let rom_info =
                Self::from_firmware_data(data, rom_info_ptr, base_addr, boot_logging_enabled)
                    .map_err(|e| format!("Failed to parse ROM info {}: {}", i, e))?;

            rom_infos.push(rom_info);
        }

        Ok(rom_infos)
    }

    // Parse a single rom info
    fn from_firmware_data(
        data: &[u8],
        ptr: u32,
        base_addr: u32,
        boot_logging_enabled: bool,
    ) -> Result<Self, String> {
        if ptr < base_addr {
            return Err(format!("Invalid ROM info pointer: 0x{:08X}", ptr));
        }

        let info_offset = (ptr - base_addr) as usize;
        let required_size = if boot_logging_enabled { 8 } else { 4 };

        if info_offset + required_size > data.len() {
            return Err("ROM info data out of bounds".into());
        }

        let info_data = &data[info_offset..];

        let (rom_type, cs1_state, cs2_state, cs3_state, filename) = if boot_logging_enabled {
            let (_, rom_info) = SdrrRomInfoWithLogging::from_bytes((info_data, 0))
                .map_err(|e| format!("Failed to parse ROM info with logging: {}", e))?;

            let filename = if rom_info.filename_ptr >= base_addr {
                read_string_at_ptr(data, rom_info.filename_ptr, base_addr).ok()
            } else {
                None
            };

            (
                rom_info.rom_type,
                rom_info.cs1_state,
                rom_info.cs2_state,
                rom_info.cs3_state,
                filename,
            )
        } else {
            let (_, rom_info) = SdrrRomInfoBasic::from_bytes((info_data, 0))
                .map_err(|e| format!("Failed to parse ROM info basic: {}", e))?;

            (
                rom_info.rom_type,
                rom_info.cs1_state,
                rom_info.cs2_state,
                rom_info.cs3_state,
                None,
            )
        };

        Ok(SdrrRomInfo {
            rom_type,
            cs1_state,
            cs2_state,
            cs3_state,
            filename,
        })
    }
}

impl SdrrPins {
    fn from_firmware_data(data: &[u8], ptr: u32, base_addr: u32) -> Result<Self, String> {
        let offset = (ptr - base_addr) as usize;
        if offset + SDRR_PINS_SIZE > data.len() {
            return Err("Pins data out of bounds".into());
        }

        let pins_data = &data[offset..offset + SDRR_PINS_SIZE];
        SdrrPins::from_bytes((pins_data, 0))
            .map_err(|e| format!("Failed to parse pins: {}", e))
            .map(|(_, pins)| pins)
    }
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
    core::str::from_utf8(string_bytes)
        .map(|s| s.to_string())
        .map_err(|_| "Invalid UTF-8 string".into())
}

fn read_rom_sets(
    data: &[u8],
    ptr: u32,
    count: u8,
    base_addr: u32,
    boot_logging_enabled: bool,
) -> Result<Vec<SdrrRomSet>, String> {
    if ptr < base_addr || count == 0 {
        return Ok(Vec::new());
    }

    let offset = (ptr - base_addr) as usize;
    let mut rom_sets = Vec::with_capacity(count as usize);

    for i in 0..count {
        let set_offset = offset + (i as usize * ROM_SET_HEADER_SIZE);

        if set_offset + ROM_SET_HEADER_SIZE > data.len() {
            return Err(format!("ROM set header {} out of bounds", i));
        }

        let set_data = &data[set_offset..];
        let (_, header) = SdrrRomSetHeader::from_bytes((set_data, 0))
            .map_err(|e| format!("Failed to parse ROM set header {}: {}", i, e))?;

        // Delegate to the type-specific constructor
        let rom_set = SdrrRomSet::from_header_and_firmware_data(
            header,
            data,
            base_addr,
            boot_logging_enabled,
        )
        .map_err(|e| format!("Failed to construct ROM set {}: {}", i, e))?;

        rom_sets.push(rom_set);
    }

    Ok(rom_sets)
}
