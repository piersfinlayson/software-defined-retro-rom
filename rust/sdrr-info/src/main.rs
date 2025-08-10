// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-info
//!
//! This tool extracts the core properties, configuration options and ROM
//! image information from an SDRR firmware binary or ELF file.
//!
//! It supports firmware version from v0.2.0 onwards.  v0.1.0 firmware did not
//! contain the relevant magic bytes or properties in a format that could be
//! easily extracted.
//!
//! It works by:
//! - Loading the provided file
//! - Detecting whether it's an ELF file (otherwise it assumes a binary file)
//! - If an ELF file, looks for the .sddr_info and .ro_data sections and builds
//!   a quick and dirty binary file from them.
//! - If it's not an ELF file, checks for the magic bytes at the known
//!   location (start of the sdrr_info structure, expects to be located at
//!   0x200 from the start of the binary).
//! - Then performs common processing on the binary file or "fake" ELF binary,
//!   starting with the sdrr_info struct (which contains the core options) and
//!   following and enumerating the ROM sets and images.

// Max version supported by sdrr-info
pub const SDRR_VERSION_MAJOR: u16 = 0;
pub const SDRR_VERSION_MINOR: u16 = 2;
pub const SDRR_VERSION_PATCH: u16 = 1;

// Modules
mod args;
mod load;
mod utils;

// External crates
use anyhow::Result;
use chrono::{DateTime, Local};
use core::fmt;
use std::fs::metadata;
use std::io::Write;
use std::path::Path;

use args::{Args, Command, parse_args};
use load::load_sdrr_firmware;
use sdrr_fw_parser::{Parser, readers::MemoryReader};
use sdrr_fw_parser::{SdrrAddress, SdrrCsSet, SdrrInfo, SdrrServe, SdrrStmPort, StmLine};
use utils::add_commas;

// SDRR info structure offset in firmware binary
pub const SDRR_INFO_OFFSET: usize = 0x200;

// STM32F4 flash base address
pub const STM32F4_FLASH_BASE: u32 = 0x08000000;

// Supported file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    // A .elf file
    Elf,

    // A .bin file
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

struct FirmwareData {
    file_type: FileType,
    file_size: usize,
    parser: Parser<MemoryReader>,
    info: SdrrInfo,
}

pub fn print_header() {
    println!("Software Defined Retro ROM - Firmware Information");
    println!("-------------------------------------------------");
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            print_header();
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let firmware_path = &args.firmware;
    let mut fw_data = match load_sdrr_firmware(firmware_path).await {
        Ok(info) => info,
        Err(e) => {
            print_header();
            eprintln!("Error loading firmware");
            eprintln!(
                "Did you supply an SDRR v{}.{}.{} or later .elf or .bin file?",
                SDRR_VERSION_MAJOR, SDRR_VERSION_MINOR, SDRR_VERSION_PATCH
            );
            eprintln!("Detailed error: {}", e);
            std::process::exit(1);
        }
    };

    // Only output a header if output-binary argument not set
    if let Some(binary) = args.output_binary {
        if !binary {
            print_header();
        }
    }

    match args.command {
        Command::Info => print_sdrr_info(&fw_data, &args),
        Command::LookupRaw => match args.range {
            None => lookup_raw(&mut fw_data, &args).await,
            Some(_) => lookup_raw_range(&mut fw_data, &args).await,
        },
        Command::Lookup => lookup(&mut fw_data, &args).await,
    }

    Ok(())
}

fn print_sdrr_info(fw_data: &FirmwareData, args: &Args) {
    let info = &fw_data.info;

    print_header();
    println!();

    println!("Core Firmware Properties");
    println!("------------------------");
    println!(
        "File name:     {}",
        Path::new(&args.firmware)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| "".into())
    );
    let modified_str = metadata(&args.firmware)
        .and_then(|data| data.modified())
        .map(|modified| {
            let datetime: DateTime<Local> = modified.into();
            datetime.format("%b %e %Y %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| "error".to_string());
    println!("File modified: {}", modified_str);
    println!("File type:     {}", fw_data.file_type);
    println!(
        "File size:     {} bytes ({}KB)",
        add_commas(fw_data.file_size as u64),
        fw_data.file_size.div_ceil(1024)
    );
    println!(
        "Version:       {}.{}.{} (build {})",
        info.major_version, info.minor_version, info.patch_version, info.build_number
    );

    println!(
        "Build Date:    {}",
        info.build_date.as_deref().unwrap_or("unknown")
    );

    println!(
        "Git commit:    {}",
        std::str::from_utf8(&info.commit).unwrap_or("<error>")
    );

    println!(
        "Hardware:      {}",
        info.hw_rev.as_deref().unwrap_or("unknown")
    );

    if (info.stm_line == StmLine::F401BC) || (info.stm_line == StmLine::F401DE) {
        println!(
            "STM32:         F401R{} ({}KB flash, {}KB RAM)",
            info.stm_storage.package_code(),
            info.stm_storage.kb(),
            info.stm_line.ram_kb()
        );
    } else {
        println!(
            "STM32:         {:?}R{} ({}KB flash, {}KB RAM)",
            info.stm_line,
            info.stm_storage.package_code(),
            info.stm_storage.kb(),
            info.stm_line.ram_kb()
        );
    }
    println!(
        "Frequency:     {} MHz (Overclocking: {})",
        info.freq, info.overclock
    );
    println!();
    println!("Configurable Options");
    println!("--------------------");

    let rom_pins = if let Some(pins) = info.pins.as_ref() {
        format!("{} pin ROM", pins.rom_pins)
    } else {
        "unknown".to_string()
    };
    println!("ROM emulation:    {}", rom_pins);
    let preload = if info.preload_image_to_ram {
        "RAM"
    } else {
        "false"
    };
    println!("Serve image from: {}", preload);
    let bootloader = if info.bootloader_capable {
        "true (close all image select jumpers to activate)"
    } else {
        "false"
    };
    println!("SWD enabled:      {}", info.swd_enabled);
    println!("Boot logging:     {}", info.boot_logging_enabled);
    println!("Status LED:       {}", info.status_led_enabled);
    println!("Count ROM access: {}", info.count_rom_access);
    println!("STM bootloader:   {}", bootloader);
    let mco = if info.mco_enabled {
        "true (exposed via test pad)"
    } else {
        "false"
    };
    println!("MCO enabled:      {}", mco);
    println!(
        "Boot config:      0x{:2X}{:2X}{:2X}{:2X} - Reserved, should be 0xFFFFFFFF",
        info.boot_config[0], info.boot_config[1], info.boot_config[2], info.boot_config[3]
    );
    println!();

    if args.detail {
        println!("Pin Configuration");
        println!("-----------------");

        let pins = &info.pins;
        if let Some(pins) = pins {
            println!();
            println!("Data pin mapping:");
            for (ii, &pin) in pins.data.iter().enumerate() {
                if pin != 0xFF {
                    println!(
                        "  D{}: {}P{}{}",
                        ii,
                        if ii < 10 { " " } else { "" },
                        pins.data_port,
                        pin
                    );
                }
            }
            println!();
            println!("Address pin mapping:");
            for (ii, &pin) in pins.addr.iter().enumerate() {
                if pin != 0xFF {
                    println!(
                        "  A{}: {}P{}{}",
                        ii,
                        if ii < 10 { " " } else { "" },
                        pins.addr_port,
                        pin
                    );
                }
            }
            println!();
            println!("Chip select pins:");
            if pins.cs1_2364 != 0xFF {
                println!("  2364 CS1: P{}{}", pins.cs_port, pins.cs1_2364);
            }
            if pins.cs1_2332 != 0xFF {
                println!("  2332 CS1: P{}{}", pins.cs_port, pins.cs1_2332);
            }
            if pins.cs2_2332 != 0xFF {
                println!("  2332 CS2: P{}{}", pins.cs_port, pins.cs2_2332);
            }
            if pins.cs1_2316 != 0xFF {
                println!("  2316 CS1: P{}{}", pins.cs_port, pins.cs1_2316);
            }
            if pins.cs2_2316 != 0xFF {
                println!("  2316 CS2: P{}{}", pins.cs_port, pins.cs2_2316);
            }
            if pins.cs3_2316 != 0xFF {
                println!("  2316 CS3: P{}{}", pins.cs_port, pins.cs3_2316);
            }
            if pins.ce_23128 != 0xFF {
                println!("  23128 CE: P{}{}", pins.cs_port, pins.ce_23128);
            }
            if pins.oe_23128 != 0xFF {
                println!("  23128 OE: P{}{}", pins.cs_port, pins.oe_23128);
            }
            if pins.x1 != 0xFF {
                println!("  Multi X1: P{}{}", pins.cs_port, pins.x1);
            }
            if pins.x2 != 0xFF {
                println!("  Multi X2: P{}{}", pins.cs_port, pins.x2);
            }
            println!();
            println!("Image select pins:");
            if pins.sel0 != 0xFF {
                println!("  SEL0: P{}{}", pins.sel_port, pins.sel0);
            }
            if pins.sel1 != 0xFF {
                println!("  SEL1: P{}{}", pins.sel_port, pins.sel1);
            }
            if pins.sel2 != 0xFF {
                println!("  SEL2: P{}{}", pins.sel_port, pins.sel2);
            }
            if pins.sel3 != 0xFF {
                println!("  SEL3: P{}{}", pins.sel_port, pins.sel3);
            }
            println!();
            println!("Status LED pin:");
            if pins.status_port == SdrrStmPort::None {
                println!("  Pin: None");
            } else {
                println!("  Pin: P{}{}", pins.status_port, pins.status);
            }
            println!();
        } else {
            println!("No pin configuration available");
            return;
        }
    }

    println!("ROMs Summary:");
    println!("-------------");
    println!("Total sets: {}", info.rom_set_count);
    // Count up total number of ROM images across all sets
    let total_roms: usize = info.rom_sets.iter().map(|set| set.roms.len()).sum();
    println!("Total ROMs: {}", total_roms);

    if args.detail {
        println!();
        println!("ROM Details:");
        println!("--------------");

        for (ii, rom_set) in info.rom_sets.iter().enumerate() {
            if ii > 0 {
                println!("-----------");
            }
            println!("ROM Set: {}", ii);
            let set_type = if rom_set.serve == SdrrServe::AddrOnAnyCs {
                "Multi-ROM socket"
            } else if rom_set.rom_count > 1 {
                "Dynamic bank switching"
            } else {
                "Single ROM image"
            };
            println!("  Set type:      {}", set_type);
            println!("  Size:          {} bytes", rom_set.size);
            println!("  ROM Count:     {}", rom_set.rom_count);
            println!("  Algorithm:     {}", rom_set.serve);
            println!("  Multi-ROM CS1: {}", rom_set.multi_rom_cs1_state);

            for (jj, rom) in rom_set.roms.iter().enumerate() {
                println!("  ROM: {}", jj);
                println!("    Type:        {}", rom.rom_type);
                println!(
                    "    Name:        {}",
                    rom.filename.as_deref().unwrap_or("<not present>")
                );
                println!(
                    "    CS States:   {}/{}/{}",
                    rom.cs1_state, rom.cs2_state, rom.cs3_state
                );
            }
        }
    }
}

async fn lookup_byte_at_address(
    fw_data: &mut FirmwareData,
    detail: bool,
    set: u8,
    addr: SdrrAddress,
    output_mangled_byte: bool,
) -> Result<(), String> {
    let info = &mut fw_data.info;
    let parser = &mut fw_data.parser;

    // Get the size of this rom set
    let byte = if output_mangled_byte {
        info.read_rom_byte_raw(parser, set, addr).await
    } else {
        info.read_rom_byte_demangled(parser, set, addr).await
    }?;

    // Get ROM names
    let roms: Vec<String> = info.rom_sets[set as usize]
        .roms
        .iter()
        .map(|rom| {
            rom.filename
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string())
        })
        .collect();
    let rom_name = roms.join(", ");

    if detail {
        println!("Byte lookup ROM set {} ({})", set, rom_name);
        if let SdrrAddress::Logical(addr) = addr {
            // We can unwrap the mangled address, because read_rom_byte() above
            // has successfully mangled it
            println!("Mangled address 0x{:04X}", addr.mangle(info).unwrap());
        }
    }

    if output_mangled_byte {
        println!("{addr:#}: 0x{byte:02X} (mangled byte)")
    } else {
        println!("{addr:#}: 0x{byte:02X} (demangled byte)")
    }

    Ok(())
}

async fn lookup_raw(fw_data: &mut FirmwareData, args: &Args) {
    println!("Lookup Byte Using Raw (mangled) Address");
    println!("---------------------------------------");

    // Ensure we have the arguments
    let set = args.set.expect("Internal error: set number is required");
    let addr = args.addr.expect("Internal error: address is required");
    let output_mangled_byte = args
        .output_mangled
        .expect("Internal error: output_mangled is required");

    if let Err(e) = lookup_byte_at_address(
        fw_data,
        args.detail,
        set,
        SdrrAddress::from_raw(addr),
        output_mangled_byte,
    )
    .await
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn lookup_raw_range(fw_data: &mut FirmwareData, args: &Args) {
    let info = &mut fw_data.info;
    let parser = &mut fw_data.parser;

    // Ensure we have the arguments
    let detail = args.detail;
    let set = args.set.expect("Internal error: set number is required");
    let output_mangled = args
        .output_mangled
        .expect("Internal error: output_mangled is required");
    let output_binary = args
        .output_binary
        .expect("Internal error: output_binary is required");
    let (start_addr, end_addr) = args
        .range
        .expect("Internal error: address range is required");

    // Validate address range
    let rom_size = info.rom_sets[set as usize].size;
    eprintln!("ROM size: 0x{:04X}", rom_size);
    if start_addr > end_addr || start_addr >= rom_size || end_addr >= rom_size {
        eprintln!(
            "Error: Invalid address range: 0x{:04X} to 0x{:04X} for ROM set size 0x{:04X}",
            start_addr, end_addr, rom_size
        );
        std::process::exit(1);
    }
    let roms: Vec<String> = info.rom_sets[set as usize]
        .roms
        .iter()
        .map(|rom| {
            rom.filename
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string())
        })
        .collect();
    let rom_name = roms.join(", ");

    if output_binary {
        let mut binary_data = Vec::new();
        for addr in start_addr..=end_addr {
            let byte = info
                .read_rom_byte_raw(parser, set, SdrrAddress::from_raw(addr))
                .await
                .expect("Failed to read byte");

            let output_byte = if output_mangled {
                byte
            } else {
                info.demangle_byte(byte)
                    .expect("Pin configuration not available")
            };

            binary_data.push(output_byte);
        }
        std::io::stdout()
            .write_all(&binary_data)
            .expect("Failed to write binary data to stdout");
    } else {
        // Hex dump output
        if detail {
            println!("Byte lookup ROM set {} ({})", set, rom_name);
            println!(
                "Raw address range 0x{:04X} to 0x{:04X}:",
                start_addr, end_addr
            );
        }

        for addr in start_addr..=end_addr {
            let log_addr = SdrrAddress::from_raw(addr);
            let byte = info
                .read_rom_byte_raw(parser, set, log_addr)
                .await
                .expect("Failed to read byte");

            let output_byte = if output_mangled {
                byte
            } else {
                info.demangle_byte(byte)
                    .expect("Pin configuration not available")
            };

            let byte_pos = (addr - start_addr) as usize;

            // Print address at start of each line
            if byte_pos % 16 == 0 {
                print!("{:04X}: ", addr);
            }

            // Print the byte
            print!("{:02X}", output_byte);

            // Add spacing
            if (byte_pos + 1) % 16 == 0 {
                // Newline every 16 bytes
                println!();
            } else if (byte_pos + 1) % 4 == 0 {
                // Bigger space every 4 bytes
                print!("  ");
            } else {
                // Regular space between bytes
                print!(" ");
            }
        }

        // Ensure we end with a newline if we didn't just print one
        let total_bytes = (end_addr - start_addr + 1) as usize;
        if total_bytes % 16 != 0 {
            println!();
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn lookup_range(
    fw_data: &mut FirmwareData,
    detail: bool,
    set: u8,
    start_addr: u32,
    end_addr: u32,
    cs_set: &SdrrCsSet,
    output_mangled: bool,
    output_binary: bool,
) -> Result<(), String> {
    let info = &mut fw_data.info;
    let parser = &mut fw_data.parser;

    let roms: Vec<String> = info.rom_sets[set as usize]
        .roms
        .iter()
        .map(|rom| {
            rom.filename
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string())
        })
        .collect();
    let rom_name = roms.join(", ");

    // Validate address range
    let rom_type = info.rom_sets[set as usize].roms[0].rom_type;
    let max_addr = rom_type.max_addr();
    if start_addr > end_addr || start_addr > max_addr || end_addr > max_addr {
        return Err(format!(
            "Invalid address range: 0x{:04X} to 0x{:04X} for ROM type {}",
            start_addr, end_addr, rom_type
        ));
    }
    if cs_set.cs2().is_some() && !rom_type.supports_cs2() {
        return Err(format!("ROM type {} does not support CS2 line", rom_type));
    }
    if cs_set.cs3().is_some() && !rom_type.supports_cs3() {
        return Err(format!("ROM type {} does not support CS3 line", rom_type));
    }
    if (cs_set.x1().is_some() || cs_set.x2().is_some())
        && info.rom_sets[set as usize].roms.len() < 2
    {
        return Err("Multi-ROM X1/X2 lines can only be used with multi-ROM sets".to_string());
    }

    if output_binary {
        // Collect bytes for binary output
        let mut binary_data = Vec::new();

        for addr in start_addr..=end_addr {
            let log_addr = SdrrAddress::from_logical(addr, cs_set);
            let byte = info.read_rom_byte_raw(parser, set, log_addr).await?;

            let output_byte = if output_mangled {
                byte
            } else {
                info.demangle_byte(byte)?
            };

            binary_data.push(output_byte);
        }

        // Write binary data to stdout
        std::io::stdout()
            .write_all(&binary_data)
            .map_err(|e| format!("Failed to write binary data to stdout: {}", e))?;
    } else {
        // Hex dump output
        if detail {
            println!("Byte lookup ROM set {} ({})", set, rom_name);
            println!("Address range 0x{:04X} to 0x{:04X}:", start_addr, end_addr);
        }

        for addr in start_addr..=end_addr {
            let log_addr = SdrrAddress::from_logical(addr, cs_set);
            let byte = info.read_rom_byte_raw(parser, set, log_addr).await?;

            let output_byte = if output_mangled {
                byte
            } else {
                info.demangle_byte(byte)?
            };

            let byte_pos = (addr - start_addr) as usize;

            // Print address at start of each line
            if byte_pos % 16 == 0 {
                print!("{:04X}: ", addr);
            }

            // Print the byte
            print!("{:02X}", output_byte);

            // Add spacing
            if (byte_pos + 1) % 16 == 0 {
                // Newline every 16 bytes
                println!();
            } else if (byte_pos + 1) % 4 == 0 {
                // Bigger space every 4 bytes
                print!("  ");
            } else {
                // Regular space between bytes
                print!(" ");
            }
        }

        // Ensure we end with a newline if we didn't just print one
        let total_bytes = (end_addr - start_addr + 1) as usize;
        if total_bytes % 16 != 0 {
            println!();
        }
    }

    Ok(())
}

async fn lookup(fw_data: &mut FirmwareData, args: &Args) {
    let binary = args.output_binary.unwrap_or(false);
    if !binary {
        println!("Lookup Byte Using Real (non-mangled) Address");
        println!("--------------------------------------------");
    }

    // Ensure we have the arguments
    let set = args.set.expect("Internal error: set number is required");
    let output_mangled = args
        .output_mangled
        .expect("Internal error: output_mangled is required");
    let _output_binary = args
        .output_binary
        .expect("Internal error: output_binary is required");
    let cs1 = args.cs1.expect("Internal error: cs1 is required");
    let cs2 = args.cs2;
    let cs3 = args.cs3;
    let x1 = args.x1;
    let x2 = args.x2;
    let cs_set = SdrrCsSet::new(cs1, cs2, cs3, x1, x2);

    if let Some((start_addr, end_addr)) = args.range {
        // Range lookup
        let output_binary = args.output_binary.unwrap_or(false);
        if let Err(e) = lookup_range(
            fw_data,
            args.detail,
            set,
            start_addr,
            end_addr,
            &cs_set,
            output_mangled,
            output_binary,
        )
        .await
        {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        // Single address lookup
        let addr = args.addr.expect("Internal error: address is required");
        let addr = SdrrAddress::from_logical(addr, &cs_set);

        if let Err(e) =
            lookup_byte_at_address(fw_data, args.detail, set, addr, output_mangled).await
        {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
