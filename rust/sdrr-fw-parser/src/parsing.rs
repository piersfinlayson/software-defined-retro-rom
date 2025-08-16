// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-fw-parser
//!
//! Contains code and internal structures for parsing the SDRR firmware

use deku::prelude::*;
use static_assertions::const_assert_eq;

use crate::Reader;
use crate::{MAX_VERSION_MAJOR, MAX_VERSION_MINOR, MAX_VERSION_PATCH};
use crate::{SdrrCsState, SdrrRomType, SdrrServe, McuLine, McuStorage};
use crate::{SdrrPins, SdrrRomInfo, SdrrRomSet};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec, vec::Vec};

// Maximum length of strings and bits of strings read from firmware
const MAX_STRING_LEN: usize = 1024;
const STRING_READ_CHUNK_SIZE: usize = 64;

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "little", magic = b"sdrr")]
// Used internally to construct [`SdrrRuntimeInfo`]
pub(crate) struct SdrrRuntimeInfoHeader {
    pub runtime_info_size: u8,
    pub image_sel: u8,
    pub rom_set_index: u8,
    pub count_rom_access: u8,
    #[deku(endian = "little")]
    pub access_count: u32,
    #[deku(endian = "little")]
    pub rom_table_ptr: u32,
    #[deku(endian = "little")]
    pub rom_table_size: u32,
}

impl SdrrRuntimeInfoHeader {
    // Cannot assert this against SdrrInfoHeader size, as contains Vecs, which
    // increase its size.
    const SDRR_RUNTIME_INFO_HEADER_SIZE: usize = 20;

    // Offset of the access count field in the runtime info header
    const ACCESS_COUNT_OFFSET: usize = 8;

    pub(crate) const fn size() -> usize {
        // Rrust struct size ignored the magic bytes
        const_assert_eq!(
            core::mem::size_of::<SdrrRuntimeInfoHeader>(),
            SdrrRuntimeInfoHeader::SDRR_RUNTIME_INFO_HEADER_SIZE-4
        );
        Self::SDRR_RUNTIME_INFO_HEADER_SIZE
    }

    pub(crate) const fn access_count_offset() -> usize {
        Self::ACCESS_COUNT_OFFSET
    }
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "little", magic = b"SDRR")]
// Used internally to construct [`SdrrInfo`]
//
// Reflects `sdrr_info_t` from `sdrr/include/config_base.h`
pub(crate) struct SdrrInfoHeader {
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
    pub stm_line: McuLine,
    pub stm_storage: McuStorage,
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
    pub count_rom_access: u8,
    #[deku(pad_bytes_before = "1")]
    #[deku(endian = "little")]
    pub rom_sets_ptr: u32,
    #[deku(endian = "little")]
    pub pins_ptr: u32,
    #[deku(bytes = "4")]
    pub boot_config: [u8; 4],
}

impl SdrrInfoHeader {
    // Cannot assert this against SdrrInfoHeader size, as contains Vecs, which
    // increase its size.
    const SDRR_INFO_HEADER_SIZE: usize = 56;

    pub(crate) const fn size() -> usize {
        Self::SDRR_INFO_HEADER_SIZE
    }
}

// Information about a specific ROM set
//
// Reflects `sdrr_rom_set_info_t` from `sdrr/include/config_base.h`
//
// Used internally to construct [`SdrrRomSet`]
#[derive(Debug, DekuRead, DekuWrite)]
pub(crate) struct SdrrRomSetHeader {
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

impl SdrrRomSetHeader {
    // Cannot assert this against SdrrRomSetHeader size, as contains Vecs, which
    // increase its size.
    const ROM_SET_HEADER_SIZE: usize = 16;

    pub(crate) const fn size() -> usize {
        const_assert_eq!(
            core::mem::size_of::<SdrrRomSetHeader>(),
            SdrrRomSetHeader::ROM_SET_HEADER_SIZE
        );
        Self::ROM_SET_HEADER_SIZE
    }
}

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

impl SdrrRomInfoBasic {
    const ROM_INFO_BASIC_SIZE: usize = 4;
    pub(crate) fn size() -> usize {
        const_assert_eq!(
            core::mem::size_of::<SdrrRomInfoBasic>(),
            SdrrRomInfoBasic::ROM_INFO_BASIC_SIZE
        );
        Self::ROM_INFO_BASIC_SIZE
    }
}

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

impl SdrrRomInfoWithLogging {
    const ROM_INFO_WITH_LOGGING_SIZE: usize = 8;
    pub(crate) fn size() -> usize {
        const_assert_eq!(
            core::mem::size_of::<SdrrRomInfoWithLogging>(),
            SdrrRomInfoWithLogging::ROM_INFO_WITH_LOGGING_SIZE
        );
        Self::ROM_INFO_WITH_LOGGING_SIZE
    }
}

/// Parse and validate runtime information from buffer
pub(crate) fn parse_and_validate_runtime_info(data: &[u8]) -> Result<SdrrRuntimeInfoHeader, String> {
    if data.len() < SdrrRuntimeInfoHeader::size() {
        return Err("Runtime info data too small".into());
    }

    let (_, header) = SdrrRuntimeInfoHeader::from_bytes((data, 0))
        .map_err(|e| format!("Failed to parse runtime info header: {}", e))?;

    if header.runtime_info_size < SdrrRuntimeInfoHeader::size() as u8 {
        return Err(format!(
            "Invalid runtime info size: {} < {}",
            header.runtime_info_size,
            SdrrRuntimeInfoHeader::size()
        ));
    }

    Ok(header)
}

/// Parse and validate SDRR header from buffer
pub(crate) fn parse_and_validate_header(data: &[u8]) -> Result<SdrrInfoHeader, String> {
    if data.len() < SdrrInfoHeader::size() {
        return Err("Header data too small".into());
    }

    let (_, header) = SdrrInfoHeader::from_bytes((data, 0))
        .map_err(|e| format!("Failed to parse header: {}", e))?;

    // Validate version
    if header.major_version > MAX_VERSION_MAJOR
        || (header.major_version == MAX_VERSION_MAJOR && header.minor_version > MAX_VERSION_MINOR)
        || (header.major_version == MAX_VERSION_MAJOR
            && header.minor_version == MAX_VERSION_MINOR
            && header.patch_version > MAX_VERSION_PATCH)
    {
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

    Ok(header)
}

/// Read a null-terminated string from the given pointer
pub(crate) async fn read_string_at_ptr<R: Reader>(
    reader: &mut R,
    ptr: u32,
    base_addr: u32,
) -> Result<String, String> {
    if ptr < base_addr {
        return Err(format!("Invalid pointer: 0x{:08X}", ptr));
    }

    let mut result = Vec::new();
    let mut addr = ptr;
    let mut buf = [0u8; STRING_READ_CHUNK_SIZE];

    loop {
        let chunk_size = buf.len().min(MAX_STRING_LEN - result.len());
        reader
            .read(addr, &mut buf[..chunk_size])
            .await
            .map_err(|_| format!("Failed to read string at 0x{:08X}", ptr))?;

        if let Some(null_pos) = buf[..chunk_size].iter().position(|&b| b == 0) {
            result.extend_from_slice(&buf[..null_pos]);
            break;
        }

        result.extend_from_slice(&buf[..chunk_size]);
        addr += chunk_size as u32;

        if result.len() >= 1024 {
            return Err("String too long (>1KB)".into());
        }
    }

    String::from_utf8(result).map_err(|_| "Invalid UTF-8 string".into())
}

/// Read ROM sets from firmware
pub(crate) async fn read_rom_sets<R: Reader>(
    reader: &mut R,
    ptr: u32,
    count: u8,
    base_addr: u32,
    boot_logging_enabled: bool,
) -> Result<Vec<SdrrRomSet>, String> {
    if ptr < base_addr || count == 0 {
        return Ok(Vec::new());
    }

    let mut rom_sets = Vec::with_capacity(count as usize);

    for i in 0..count {
        let header_addr = ptr + (i as u32 * SdrrRomSetHeader::size() as u32);

        // Read ROM set header
        let mut header_buf = [0u8; SdrrRomSetHeader::size()];
        reader
            .read(header_addr, &mut header_buf)
            .await
            .map_err(|_| format!("Failed to read ROM set header {}", i))?;

        let (_, header) = SdrrRomSetHeader::from_bytes((&header_buf, 0))
            .map_err(|e| format!("Failed to parse ROM set header {}: {}", i, e))?;

        // Read ROM infos
        let roms = read_rom_infos(
            reader,
            header.roms_ptr,
            header.rom_count,
            base_addr,
            boot_logging_enabled,
        )
        .await?;

        // Note: We don't read the ROM data itself - just store where it is
        rom_sets.push(SdrrRomSet {
            data_ptr: header.data_ptr, // Store pointer, not data
            size: header.size,
            roms,
            rom_count: header.rom_count,
            serve: header.serve,
            multi_rom_cs1_state: header.multi_rom_cs1_state,
        });
    }

    Ok(rom_sets)
}

// Read ROM info structures
async fn read_rom_infos<R: Reader>(
    reader: &mut R,
    ptr: u32,
    count: u8,
    base_addr: u32,
    boot_logging_enabled: bool,
) -> Result<Vec<SdrrRomInfo>, String> {
    if ptr < base_addr || count == 0 {
        return Ok(Vec::new());
    }

    let mut rom_infos = Vec::with_capacity(count as usize);

    for i in 0..count {
        // Read pointer to ROM info
        let ptr_addr = ptr + (i as u32 * core::mem::size_of::<u32>() as u32);
        let mut ptr_buf = [0u8; core::mem::size_of::<u32>()];
        reader
            .read(ptr_addr, &mut ptr_buf)
            .await
            .map_err(|_| format!("Failed to read ROM info pointer {}", i))?;

        let rom_info_ptr = u32::from_le_bytes(ptr_buf);

        // Read the ROM info itself
        let info_size = if boot_logging_enabled {
            SdrrRomInfoWithLogging::size()
        } else {
            SdrrRomInfoBasic::size()
        };
        let mut info_buf = vec![0u8; info_size];
        reader
            .read(rom_info_ptr, &mut info_buf)
            .await
            .map_err(|_| format!("Failed to read ROM info {}", i))?;

        let rom_info = if boot_logging_enabled {
            let (_, info) = SdrrRomInfoWithLogging::from_bytes((&info_buf, 0))
                .map_err(|e| format!("Failed to parse ROM info with logging {}: {}", i, e))?;

            let filename = if info.filename_ptr >= base_addr {
                read_string_at_ptr(reader, info.filename_ptr, base_addr)
                    .await
                    .ok()
            } else {
                None
            };

            SdrrRomInfo {
                rom_type: info.rom_type,
                cs1_state: info.cs1_state,
                cs2_state: info.cs2_state,
                cs3_state: info.cs3_state,
                filename,
            }
        } else {
            let (_, info) = SdrrRomInfoBasic::from_bytes((&info_buf, 0))
                .map_err(|e| format!("Failed to parse ROM info basic {}: {}", i, e))?;

            SdrrRomInfo {
                rom_type: info.rom_type,
                cs1_state: info.cs1_state,
                cs2_state: info.cs2_state,
                cs3_state: info.cs3_state,
                filename: None,
            }
        };

        rom_infos.push(rom_info);
    }

    Ok(rom_infos)
}

/// Read pin configuration
pub(crate) async fn read_pins<R: Reader>(
    reader: &mut R,
    ptr: u32,
    base_addr: u32,
) -> Result<SdrrPins, String> {
    if ptr < base_addr {
        return Err(format!("Invalid pins pointer: 0x{:08X}", ptr));
    }

    let mut pins_buf = [0u8; SdrrPins::size()];
    reader
        .read(ptr, &mut pins_buf)
        .await
        .map_err(|_| "Failed to read pins data")?;

    SdrrPins::from_bytes((&pins_buf, 0))
        .map_err(|e| format!("Failed to parse pins: {}", e))
        .map(|(_, pins)| pins)
}
