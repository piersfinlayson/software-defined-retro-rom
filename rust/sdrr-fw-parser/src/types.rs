// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-fw-parser
//!
//! Enums and types used in SDRR firmware parsing

use core::fmt;
use deku::prelude::*;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

use crate::SdrrInfo;

/// STM32F4 product line options
///
/// Relflects `stm_line_t` from `sdrr/include/config_base.h`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize,
)]
#[deku(id_type = "u16", ctx = "endian: deku::ctx::Endian")]
pub enum McuLine {
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

    /// RP2350
    #[deku(id = "0x0005")]
    Rp2350,
}

impl fmt::Display for McuLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McuLine::F401DE => write!(f, "STM32F401DE"),
            McuLine::F405 => write!(f, "STM32F405"),
            McuLine::F411 => write!(f, "STM32F411"),
            McuLine::F446 => write!(f, "STM32F446"),
            McuLine::F401BC => write!(f, "STM32F401BC"),
            McuLine::Rp2350 => write!(f, "RP2350"),
        }
    }
}

impl McuLine {
    /// Returns the amount of SRAM of the device (not including any CCM RAM)
    pub fn ram_kb(&self) -> &str {
        match self {
            McuLine::F401DE => "96",
            McuLine::F401BC => "64",
            McuLine::F405 | McuLine::F411 | McuLine::F446 => "128",
            McuLine::Rp2350 => "520",
        }
    }
}

/// STM32F4 package flash storage code
///
/// For example "E" in STM32F401RET6 means 512KB of flash storage.
///
/// Reflects `stm_storage_t` from `sdrr/include/config_base.h`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize,
)]
#[deku(id_type = "u16", ctx = "endian: deku::ctx::Endian")]
pub enum McuStorage {
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

    /// 2MB
    #[deku(id = "7")]
    Storage2MB,
}

impl McuStorage {
    /// Returns the storage size in kilobytes
    pub fn kb(&self) -> &str {
        match self {
            McuStorage::Storage8 => "64",
            McuStorage::StorageB => "128",
            McuStorage::StorageC => "256",
            McuStorage::StorageD => "384",
            McuStorage::StorageE => "512",
            McuStorage::StorageF => "768",
            McuStorage::StorageG => "1024",
            McuStorage::Storage2MB => "2048",
        }
    }

    /// Returns the storage package code
    pub fn package_code(&self) -> &str {
        match self {
            McuStorage::Storage8 => "8",
            McuStorage::StorageB => "B",
            McuStorage::StorageC => "C",
            McuStorage::StorageD => "D",
            McuStorage::StorageE => "E",
            McuStorage::StorageF => "F",
            McuStorage::StorageG => "G",
            McuStorage::Storage2MB => "2MB",
        }
    }
}

impl fmt::Display for McuStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.package_code())
    }
}

/// Type of ROMs supported by SDRR
///
/// Reflects `sdrr_rom_type_t` from `sdrr/include/config_base.h`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize,
)]
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize,
)]
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize,
)]
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite, serde::Serialize, serde::Deserialize,
)]
#[deku(id_type = "u8")]
pub enum SdrrMcuPort {
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

    /// Port 0 (RP2350 only)
    #[deku(id = "0x05")]
    Port0,
}

impl fmt::Display for SdrrMcuPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdrrMcuPort::None => write!(f, "None"),
            SdrrMcuPort::PortA => write!(f, "A"),
            SdrrMcuPort::PortB => write!(f, "B"),
            SdrrMcuPort::PortC => write!(f, "C"),
            SdrrMcuPort::PortD => write!(f, "D"),
            SdrrMcuPort::Port0 => write!(f, "0"),
        }
    }
}

/// Struct representing the state of the chip select (CS) lines in a given
/// logical address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdrrCsSet {
    /// The state of CS1 (1 = high, 0 = low)
    cs1: bool,

    /// The state of CS2 (1 = high, 0 = low) - 2332/2316 ROM types only
    cs2: Option<bool>,

    /// The state of CS3 (1 = high, 0 = low) - 2316 ROM type only
    cs3: Option<bool>,

    /// The state of X1 (1 = high, 0 = low) - rom sets only
    x1: Option<bool>,

    /// The state of X2 (1 = high, 0 = low) - rom sets only
    x2: Option<bool>,
}

impl SdrrCsSet {
    /// Constructs a new CS set
    pub fn new(
        cs1: bool,
        cs2: Option<bool>,
        cs3: Option<bool>,
        x1: Option<bool>,
        x2: Option<bool>,
    ) -> Self {
        Self {
            cs1,
            cs2,
            cs3,
            x1,
            x2,
        }
    }

    /// Returns the state of CS1
    pub fn cs1(&self) -> bool {
        self.cs1
    }

    /// Returns the state of CS2
    pub fn cs2(&self) -> Option<bool> {
        self.cs2
    }

    /// Returns the state of CS3
    pub fn cs3(&self) -> Option<bool> {
        self.cs3
    }

    /// Returns the state of X1
    pub fn x1(&self) -> Option<bool> {
        self.x1
    }

    /// Returns the state of X2
    pub fn x2(&self) -> Option<bool> {
        self.x2
    }
}

/// Struct representing the information required to construct a logical address
/// to access a byte in an SDRR rom set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdrrLogicalAddress {
    /// The logical address of the ROM to access - i.e the value of the address
    /// pins on the original ROM chip.
    addr: u32,

    /// The CS state associated with this address
    cs_set: SdrrCsSet,
}

impl SdrrLogicalAddress {
    /// Constructs a new logical address
    pub fn new(addr: u32, cs_set: SdrrCsSet) -> Self {
        Self { addr, cs_set }
    }

    /// Mangles the logical address into a raw address using the pin config from
    /// the given `SdrrInfo`.
    pub fn mangle(&self, info: &SdrrInfo) -> Result<u32, String> {
        info.mangle_address(self)
    }

    /// Returns the logical address pins state from the original ROM
    pub fn addr(&self) -> u32 {
        self.addr
    }

    /// Returns the CS set associated with this logical address
    pub fn cs_set(&self) -> &SdrrCsSet {
        &self.cs_set
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdrrAddress {
    /// Raw address - used to index directly into the ROM set
    Raw(u32),

    /// Logical address - the logical address on the original ROM, plus CS
    /// and X1/X2 lines
    Logical(SdrrLogicalAddress),
}

impl SdrrAddress {
    /// Creates a new raw address from the given value.
    pub fn from_raw(addr: u32) -> Self {
        Self::Raw(addr)
    }

    /// Creates a new logical address from the given components.
    pub fn from_logical(addr: u32, cs_set: &SdrrCsSet) -> Self {
        Self::Logical(SdrrLogicalAddress::new(addr, *cs_set))
    }
}

impl fmt::Display for SdrrAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            match self {
                SdrrAddress::Raw(addr) => write!(f, "Raw address: 0x{:04X}", addr),
                SdrrAddress::Logical(logical) => write!(
                    f,
                    "Logical address: 0x{:04X} (CS1: {}, CS2: {:?}, CS3: {:?}, X1: {:?}, X2: {:?})",
                    logical.addr,
                    logical.cs_set.cs1(),
                    logical.cs_set.cs2(),
                    logical.cs_set.cs3(),
                    logical.cs_set.x1(),
                    logical.cs_set.x2()
                ),
            }
        } else {
            match self {
                SdrrAddress::Raw(addr) => write!(f, "0x{:04X}", addr),
                SdrrAddress::Logical(logical) => write!(f, "0x{:04X}", logical.addr),
            }
        }
    }
}
