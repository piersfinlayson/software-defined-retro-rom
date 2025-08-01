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
//! use sdrr_fw_parser::SdrrInfo;
//! let sdrr_info = SdrrInfo::from_firmware_bytes(
//!     SdrrFileType::Elf,
//!     &sdrr_info, // Reference to sdrr_info_t from firmware file
//!     &full_fw,   // Reference to full firmware data
//!     file_size   // Size of the full firmware file in bytes
//! );
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

/// Maximum SDRR firmware versions supported by this version of`sdrr-fw-parser`
pub const MAX_VERSION_MAJOR: u16 = 0;
pub const MAX_VERSION_MINOR: u16 = 3;
pub const MAX_VERSION_PATCH: u16 = 0;

// lib.rs - Public API and core traits
pub mod info;
mod parsing;
pub mod readers;
pub mod types;

// Use alloc if no-std.
#[cfg(not(feature = "std"))]
extern crate alloc;

use core::fmt;

pub use info::{SdrrInfo, SdrrPins, SdrrRomInfo, SdrrRomSet};
pub use types::{
    SdrrAddress, SdrrCsSet, SdrrCsState, SdrrLogicalAddress, SdrrRomType, SdrrServe, SdrrStmPort,
    StmLine, StmStorage,
};

use crate::parsing::{SdrrInfoHeader, parse_and_validate_header};

/// Offset from start of the firmware where the SDRR info header is located.
///
/// The first 4 "magic" bytes are "SDRR".
pub const SDRR_INFO_FW_OFFSET: u32 = 0x200;

// Use std/no-std String and Vec types
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

// STM32F4 flash base address.  Required to find offset from pointers
pub(crate) const STM32F4_FLASH_BASE: u32 = 0x08000000;

/// Trait for reading firmware data from a source.
///
/// This trait abstracts over different ways of reading SDRR firmware data,
/// allowing the parser to work with various sources without knowing the
/// underlying implementation details.
///
/// # Implementations
///
/// - For PC applications: Read from in-memory buffers or memory-mapped files
/// - For embedded devices: Read from flash via SWD, JTAG, or other debug interfaces
/// - For bootloaders: Read directly from flash memory
///
/// # Address Space
///
/// The `read` method uses absolute addresses as they appear in the target's
/// memory map. For STM32F4 devices, flash typically starts at `0x08000000`.
/// The implementation is responsible for translating these addresses to
/// whatever internal representation it uses (file offsets, SWD commands, etc.).
///
/// # Example
///
/// ```rust,no_run
/// # struct MyReader;
/// # impl Reader for MyReader {
/// #     type Error = std::io::Error;
/// #     fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
/// #         Ok(())
/// #     }
/// # }
/// use sdrr_fw_parser::{Reader, SdrrInfo};
///
/// let mut reader = MyReader::new();
/// let sdrr_info = SdrrInfo::from_reader(&mut reader)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait Reader {
    /// The error type returned by read operations.
    ///
    /// This allows implementations to use their own error types
    /// (e.g., `std::io::Error` for file I/O, custom errors for SWD).
    type Error;

    /// Read bytes from the firmware at the specified absolute address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The absolute address to read from (e.g., `0x08000200`)
    /// * `buf` - Buffer to fill with the read data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The address is out of bounds for the firmware
    /// - The underlying read operation fails (I/O error, communication error, etc.)
    /// - The requested read size would exceed firmware boundaries
    ///
    /// # Performance Notes
    ///
    /// Implementations should optimize for small reads (1-256 bytes) as the parser
    /// typically reads headers and metadata in small chunks. For embedded implementations
    /// reading via debug interfaces, consider implementing bulk reads and internal
    /// buffering to reduce round-trip overhead.
    fn read(
        &mut self,
        addr: u32,
        buf: &mut [u8],
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// Parser for Software Defined Retro ROM (SDRR) firmware images.
///
/// This parser extracts configuration and ROM data from SDRR firmware files,
/// which are used in devices that emulate vintage ROM chips (2316/2332/2364).
/// The parser is designed to work efficiently in both PC and embedded environments.
///
/// # Architecture
///
/// The parser uses a two-phase approach:
///
/// 1. **Metadata parsing** - Headers, pin configurations, and ROM set information
///    are parsed immediately into memory (typically just a few KB)
/// 2. **ROM data access** - ROM images (up to 64KB each) remain in the source
///    and are accessed lazily through reader callbacks
///
/// This design allows embedded devices with limited RAM to parse and work with
/// SDRR firmware without loading entire ROM images into memory.
///
/// # Usage
///
/// ```rust,no_run
/// # use sdrr_fw_parser::{Parser, Reader};
/// # struct MyReader;
/// # impl Reader for MyReader {
/// #     type Error = std::io::Error;
/// #     fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// // Create a reader for your data source
/// let reader = MyReader::new("firmware.bin")?;
///
/// // Create parser and parse metadata
/// let mut parser = Parser::new(reader);
/// let info = parser.parse()?;
///
/// // Access ROM data lazily
/// let byte = parser.read_rom_byte(&info, 0, 0x1000, true, None, None)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Firmware Structure
///
/// SDRR firmware contains:
/// - A header with "SDRR" magic bytes at offset 0x200 from base
/// - Version information and build metadata  
/// - Pin mapping configuration for the STM32F4 microcontroller
/// - One or more ROM sets, each containing up to 3 ROM images
/// - ROM data that has been pre-processed for efficient serving
///
/// # Address Translation
///
/// ROM addresses and data bytes are "mangled" in the firmware for efficient
/// real-time serving. The parser handles the translation between logical
/// addresses/data and their physical representation in the firmware.
pub struct Parser<R: Reader> {
    reader: R,
    base_address: u32,
}

impl<R: Reader> Parser<R> {
    /// Create a new parser with the default STM32F4 base address (0x08000000).
    ///
    /// # Arguments
    ///
    /// * `reader` - Implementation of [`Reader`] trait that provides access to firmware bytes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use sdrr_fw_parser::{Parser, Reader};
    /// # struct MyReader;
    /// # impl Reader for MyReader {
    /// #     type Error = std::io::Error;
    /// #     fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let reader = MyReader::new();
    /// let mut parser = Parser::new(reader);
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            base_address: STM32F4_FLASH_BASE,
        }
    }

    /// Create a new parser with a custom base address.
    ///
    /// Use this when parsing firmware for devices with non-standard memory maps
    /// or when analyzing relocated firmware images.
    ///
    /// # Arguments
    ///
    /// * `reader` - Implementation of [`Reader`] trait that provides access to firmware bytes
    /// * `base_address` - Base address where flash memory begins (e.g., 0x08000000 for STM32F4)
    pub fn with_base_address(reader: R, base_address: u32) -> Self {
        Self {
            reader,
            base_address,
        }
    }

    // Retrieve the SDRR info header from the firmware.
    async fn retrieve_header(&mut self) -> Result<SdrrInfoHeader, String> {
        // Try to find SDRR info at standard location
        let sdrr_info_addr = self.base_address + SDRR_INFO_FW_OFFSET;

        // Read the header
        let mut header_buf = [0u8; SdrrInfoHeader::size()];
        self.reader
            .read(sdrr_info_addr, &mut header_buf)
            .await
            .map_err(|_| "Failed to read SDRR header")?;

        // Parse and validate header using the helper
        parse_and_validate_header(&header_buf)
    }

    /// Function to do a brief check whether this is an SDRR device.
    ///
    /// Returns:
    /// - `true` if the SDRR header was found and is valid
    /// - `false` if the header was not found (or an error occured)
    pub async fn detect(&mut self) -> bool {
        match self.retrieve_header().await {
            Ok(_header) => true,
            Err(_) => false,
        }
    }

    /// Parse SDRR metadata from the firmware.
    ///
    /// This method reads and parses all structural information from the firmware,
    /// including headers, version info, pin configurations, and ROM set descriptors.
    /// ROM image data is NOT loaded - only pointers to where it exists in the firmware.
    ///
    /// # What gets parsed
    ///
    /// - SDRR header with version and build information
    /// - Pin mapping configuration for STM32F4 GPIO
    /// - ROM set headers with serving algorithms
    /// - ROM information (type, CS line configuration)
    /// - String data (build date, hardware revision, ROM filenames)
    ///
    /// # Error handling
    ///
    /// The parser attempts to continue parsing even when encountering errors in
    /// non-critical sections. Failed sections are recorded in [`SdrrInfo::parse_errors`]
    /// while their fields are set to `None`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(SdrrInfo)` if the header was found and core fields parsed successfully.
    /// Returns `Err` if:
    /// - SDRR magic bytes not found at expected location
    /// - Version is newer than this parser supports
    /// - Critical header fields are corrupted
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use sdrr_fw_parser::{Parser, Reader};
    /// # struct MyReader;
    /// # impl Reader for MyReader {
    /// #     type Error = std::io::Error;
    /// #     fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # let reader = MyReader::new();
    /// let mut parser = Parser::new(reader);
    /// match parser.parse() {
    ///     Ok(info) => {
    ///         println!("Parsed SDRR v{}.{}.{}",
    ///                  info.major_version,
    ///                  info.minor_version,
    ///                  info.patch_version);
    ///         if !info.parse_errors.is_empty() {
    ///             println!("Encountered {} non-fatal errors", info.parse_errors.len());
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to parse: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn parse(&mut self) -> Result<SdrrInfo, String> {
        // Parse and validate header using the helper
        let header = self.retrieve_header().await?;

        let mut parse_errors = Vec::new();

        // Parse strings with error collection
        let build_date = match self.read_string_at_ptr(header.build_date_ptr).await {
            Ok(s) => Some(s),
            Err(e) => {
                parse_errors.push(ParseError::new("Build Date", e));
                None
            }
        };

        let hw_rev = match self.read_string_at_ptr(header.hw_rev_ptr).await {
            Ok(s) => Some(s),
            Err(e) => {
                parse_errors.push(ParseError::new("Hardware Revision", e));
                None
            }
        };

        // Parse ROM sets with error collection
        let rom_sets = match parsing::read_rom_sets(
            &mut self.reader,
            header.rom_sets_ptr,
            header.rom_set_count,
            self.base_address,
            header.boot_logging_enabled != 0,
        )
        .await
        {
            Ok(sets) => sets,
            Err(e) => {
                parse_errors.push(ParseError::new("ROM Sets", e));
                Vec::new()
            }
        };

        // Parse pins
        let pins =
            match parsing::read_pins(&mut self.reader, header.pins_ptr, self.base_address).await {
                Ok(p) => Some(p),
                Err(e) => {
                    parse_errors.push(ParseError::new("Pins", e));
                    None
                }
            };

        Ok(SdrrInfo {
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
            count_rom_access: header.count_rom_access != 0,
            rom_sets,
            pins,
            boot_config: header.boot_config,
            parse_errors,
        })
    }

    async fn read_string_at_ptr(&mut self, ptr: u32) -> Result<String, String> {
        if ptr < self.base_address {
            return Err(format!("Invalid pointer: 0x{:08X}", ptr));
        }

        // Read in chunks to find null terminator
        let mut result = Vec::new();
        let mut addr = ptr;
        let mut buf = [0u8; 64];

        loop {
            let chunk_size = buf.len().min(1024 - result.len()); // Limit total size
            self.reader
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
}

/// Error information for non-fatal parsing failures.
///
/// When parsing SDRR firmware, some sections may fail to parse due to corruption,
/// invalid pointers, or other issues. Rather than failing the entire parse operation,
/// these errors are collected and reported while the parser continues with other
/// sections.
///
/// # Examples
///
/// ```rust
/// # use sdrr_fw_parser::ParseError;
/// let error = ParseError {
///     field: "build_date".to_string(),
///     reason: "Invalid pointer: 0xFFFFFFFF".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParseError {
    /// The field or structure that failed to parse.
    ///
    /// Examples:
    /// - `"build_date"` - Build date string
    /// - `"hw_rev"` - Hardware revision string  
    /// - `"rom_set[0]"` - First ROM set
    /// - `"rom_set[1].roms[2]"` - Third ROM in second ROM set
    /// - `"pins"` - Pin configuration structure
    pub field: String,

    /// Human-readable description of why parsing failed.
    ///
    /// Examples:
    /// - `"Invalid pointer: 0xFFFFFFFF"`
    /// - `"String not null-terminated within bounds"`
    /// - `"ROM data extends past end of firmware"`
    /// - `"Unsupported ROM type value: 255"`
    pub reason: String,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}
