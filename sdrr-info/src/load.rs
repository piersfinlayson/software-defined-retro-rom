// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use anyhow::Result;
use goblin::elf::Elf;
use std::path::Path;
use std::{fmt, fs};

use crate::symbols::SdrrInfo;
use crate::{SDRR_INFO_OFFSET, STM32F4_FLASH_BASE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Elf,
    Orc,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::Elf => write!(f, "ELF (.elf)"),
            FileType::Orc => write!(f, "Binary (.bin)"),
        }
    }
}

pub fn load_sdrr_firmware<P: AsRef<Path>>(path: P) -> Result<SdrrInfo> {
    let firmware_data = fs::read(path)?;

    if firmware_data.len() >= 4 && &firmware_data[0..4] == b"\x7fELF" {
        load_from_elf(firmware_data)
    } else {
        load_from_binary(firmware_data)
    }
}

fn load_from_binary(firmware_data: Vec<u8>) -> Result<SdrrInfo> {
    if firmware_data.len() < SDRR_INFO_OFFSET + 48 {
        return Err(anyhow::anyhow!("Firmware file too small"));
    }

    // Extract SDRR info from the specified offset
    let sdrr_data = &firmware_data[SDRR_INFO_OFFSET..];

    SdrrInfo::from_firmware_bytes(
        FileType::Orc,
        sdrr_data,
        &firmware_data,
        STM32F4_FLASH_BASE,
        SDRR_INFO_OFFSET,
    )
    .map_err(|e| anyhow::anyhow!(e))
}

fn load_from_elf(firmware_data: Vec<u8>) -> Result<SdrrInfo> {
    let elf = Elf::parse(&firmware_data)?;

    // Find the sdrr_info symbol
    let sdrr_symbol = elf
        .syms
        .iter()
        .find(|sym| {
            if let Some(name) = elf.strtab.get_at(sym.st_name) {
                name == "sdrr_info"
            } else {
                false
            }
        })
        .ok_or_else(|| anyhow::anyhow!("sdrr_info symbol not found"))?;

    // Get the section containing the symbol
    let sdrr_section = &elf.section_headers[sdrr_symbol.st_shndx];

    // Calculate file offset of the symbol within its section
    let symbol_offset_in_section = sdrr_symbol.st_value - sdrr_section.sh_addr;
    let symbol_file_offset = sdrr_section.sh_offset + symbol_offset_in_section;

    // Extract symbol data
    let sdrr_data = &firmware_data
        [symbol_file_offset as usize..(symbol_file_offset + sdrr_symbol.st_size) as usize];

    // Find .rodata section
    let rodata_section = elf
        .section_headers
        .iter()
        .find(|sh| {
            if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
                name == ".rodata"
            } else {
                false
            }
        })
        .ok_or_else(|| anyhow::anyhow!("No .rodata section found"))?;

    // Create synthetic binary
    let synthetic_binary =
        create_synthetic_binary_from_symbol(&firmware_data, sdrr_data, rodata_section)?;

    // Parse using existing logic
    let sdrr_data = &synthetic_binary[SDRR_INFO_OFFSET..];
    SdrrInfo::from_firmware_bytes(
        FileType::Elf,
        sdrr_data,
        &synthetic_binary,
        STM32F4_FLASH_BASE,
        SDRR_INFO_OFFSET,
    )
    .map_err(|e| anyhow::anyhow!(e))
}

fn create_synthetic_binary_from_symbol(
    elf_data: &[u8],
    sdrr_data: &[u8],
    rodata_section: &goblin::elf::SectionHeader,
) -> Result<Vec<u8>> {
    let rodata_offset = (rodata_section.sh_addr - STM32F4_FLASH_BASE as u64) as usize;

    let total_size = std::cmp::max(
        SDRR_INFO_OFFSET + sdrr_data.len(),
        rodata_offset + rodata_section.sh_size as usize,
    );

    let mut synthetic = vec![0u8; total_size];

    // Place sdrr_info at expected offset
    synthetic[SDRR_INFO_OFFSET..SDRR_INFO_OFFSET + sdrr_data.len()].copy_from_slice(sdrr_data);

    // Place .rodata at its virtual address offset
    let rodata_raw = &elf_data[rodata_section.sh_offset as usize
        ..(rodata_section.sh_offset + rodata_section.sh_size) as usize];
    synthetic[rodata_offset..rodata_offset + rodata_raw.len()].copy_from_slice(rodata_raw);

    Ok(synthetic)
}
