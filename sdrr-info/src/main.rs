/// sdrr-info
///
/// This tool extracts the core properties, configuration options and ROM
/// image information from an SDRR firmware binary or ELF file.
///
/// It supports firmware version from v0.1.1 onwards.  v0.1.0 firmware did not
/// contain the relevant magic bytes or properties in a format that could be
/// easily extracted.
///
/// It works by:
/// - Loading the provided file
/// - Detecting whether it's an ELF file (otherwise it assumes a binary file)
/// - If an ELF file, looks for the .sddr_info and .ro_data sections and builds
///   a quick and dirty binary file from them.
/// - If it's not an ELF file, checks for the magic bytes at the known
///   location (start of the sdrr_info structure, expects to be located at
///   0x200 from the start of the binary).
/// - Then performs common processing on the binary file or "fake" ELF binary,
///   starting with the sdrr_info struct (which contains the core options) and
///   then following and enumerating the ROM sets and images.

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

// Versions supported by sdrr-info
pub const SDRR_VERSION_MAJOR: u16 = 0;
pub const SDRR_VERSION_MINOR: u16 = 1;
pub const SDRR_VERSION_PATCH: u16 = 1;

// Modules
mod symbols;
mod load;
mod args;
mod utils;

// External crates
use anyhow::Result;
use std::io::Write;
use std::path::Path;
use std::fs::metadata;
use chrono::{DateTime, Local};

use symbols::SdrrInfo;
use load::load_sdrr_firmware;
use args::{Args, Command, parse_args};
use utils::add_commas;

// SDRR info structure offset in firmware binary
pub const SDRR_INFO_OFFSET: usize = 0x200;

// STM32F4 flash base address
pub const STM32F4_FLASH_BASE: u32 = 0x08000000;

pub fn print_header() {
    println!("Software Defined Retro ROM - Firmware Information");
    println!("=================================================");
}

// Example usage function
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            print_header();
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let firmware_path = &args.firmware;
    let info = match load_sdrr_firmware(firmware_path) {
        Ok(info) => info,
        Err(e) => {
            print_header();
            eprintln!("Error loading firmware");
            eprintln!("Did you supply an SDRR v0.1.1 or later .elf or .bin file?");
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
        Command::Info => print_sdrr_info(&info, &args),
        Command::LookupRaw => lookup_raw(&info, &args),
        Command::Lookup => lookup(&info, &args),
    }

    Ok(())
}

fn print_sdrr_info(info: &SdrrInfo, args: &Args) {
    println!();
    println!("Core Firmware Properties");
    println!("------------------------");
    println!("File name:     {}", 
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
    println!("File type:     {}", info.file_type);
    println!("File size:     {} bytes ({}KB)", add_commas(info.file_size as u64), (info.file_size + 1023) / 1024);
    println!(
        "Version:       {}.{}.{} (build {})",
        info.major_version, info.minor_version, info.patch_version, info.build_number
    );
    println!("Build Date:    {}", info.build_date);
    println!(
        "Git commit:    {}",
        std::str::from_utf8(&info.commit).unwrap_or("<error>")
    );
    println!("Hardware:      {}", info.hw_rev);
    println!(
        "STM32:         {:?}R{} ({}KB flash, {}KB RAM)",
        info.stm_line,
        info.stm_storage.package_code(),
        info.stm_storage.kb(),
        info.stm_line.ram_kb()
    );
    println!(
        "Frequency:     {} MHz (Overclocking: {})",
        info.freq, info.overclock
    );
    println!();
    println!("Configurable Options");
    println!("--------------------");
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
    println!("STM bootloader:   {}", bootloader);
    let mco = if info.mco_enabled {
        "true (exposed via test pad)"
    } else {
        "false"
    };
    println!("MCO enabled:      {}", mco);
    println!("Boot config:      {}, {}, {}, {} - Reserved, should be 255",
        info.boot_config[0], info.boot_config[1], info.boot_config[2], info.boot_config[3]);
    println!();

    if args.detail {
        let pins = &info.pins;
        println!("Pin Configuration");
        println!("-----------------");
        println!("Data port:    {}", pins.data_port);
        println!("Address port: {}", pins.addr_port);
        println!("CS port:      {}", pins.cs_port);
        println!("Select port:  {}", pins.sel_port);
        
        println!();
        println!("Data pin mapping - port {}:", pins.data_port);
        for (ii, &pin) in pins.data.iter().enumerate() {
            if pin != 0xFF {
                println!("  D{}: {}Port pin {}", ii, if ii < 10 { " " } else { "" }, pin);
            }
        }

        println!();
        println!("Address pin mapping - port {}:", pins.addr_port);
        for (ii, &pin) in pins.addr.iter().enumerate() {
            if pin != 0xFF {
                println!("  A{}: {}Port pin {}", ii, if ii < 10 { " " } else { "" }, pin);
            }
        }
        
        println!();
        println!("Chip select pins - port {}:", pins.cs_port);
        if pins.cs1_2364 != 0xFF { println!("  2364 CS1: Port pin {}", pins.cs1_2364); }
        if pins.cs1_2332 != 0xFF { println!("  2332 CS1: Port pin {}", pins.cs1_2332); }
        if pins.cs2_2332 != 0xFF { println!("  2332 CS2: Port pin {}", pins.cs2_2332); }
        if pins.cs1_2316 != 0xFF { println!("  2316 CS1: Port pin {}", pins.cs1_2316); }
        if pins.cs2_2316 != 0xFF { println!("  2316 CS2: Port pin {}", pins.cs2_2316); }
        if pins.cs3_2316 != 0xFF { println!("  2316 CS3: Port pin {}", pins.cs3_2316); }
        if pins.ce_23128 != 0xFF { println!("  23128 CE: Port pin {}", pins.ce_23128); }
        if pins.oe_23128 != 0xFF { println!("  23128 OE: Port pin {}", pins.oe_23128); }
        if pins.x1 != 0xFF { println!("  Multi X1: Port pin {}", pins.x1); }
        if pins.x2 != 0xFF { println!("  Multi X2: Port pin {}", pins.x2); }

        println!();
        println!("Image select pins - port {}:", pins.sel_port);
        if pins.sel0 != 0xFF { println!("  SEL0: Pin {}", pins.sel0); }
        if pins.sel1 != 0xFF { println!("  SEL1: Pin {}", pins.sel1); }
        if pins.sel2 != 0xFF { println!("  SEL2: Pin {}", pins.sel2); }
        if pins.sel3 != 0xFF { println!("  SEL3: Pin {}", pins.sel3); }

        println!();
    }

    println!("ROMs Summary:");
    println!("-------------");
    println!("Total sets: {}", info.rom_set_count);
    // Count up total number of ROM images across all sets
    let total_roms: usize = info.rom_sets.iter().map(|set| set
        .roms.len()).sum();
    println!("Total ROMs: {}", total_roms);

    if args.detail {
        println!();
        println!("ROM Details: {}", info.rom_set_count);
        println!("--------------");

        for (ii, rom_set) in info.rom_sets.iter().enumerate() {
            if ii > 0 {
                println!("-----------");
            }
            println!("ROM Set: {}", ii);
            println!("  Size: {} bytes", rom_set.size);
            println!("  ROM Count:     {}", rom_set.rom_count);
            println!("  Algorithm:     {}", rom_set.serve);
            println!("  Multi-ROM CS1: {}", rom_set.multi_rom_cs1_state);

            for (jj, rom) in rom_set.roms.iter().enumerate() {
                println!("  ROM: {}", jj);
                println!("    Type:      {}", rom.rom_type);
                println!(
                    "    Name:      {}",
                    rom.filename.as_deref().unwrap_or("<not present>")
                );
                println!(
                    "    CS States: {}/{}/{}",
                    rom.cs1_state, rom.cs2_state, rom.cs3_state
                );
            }
        }
    }
}

fn lookup_byte_at_address(
    info: &SdrrInfo,
    detail: bool,
    set: u8,
    lookup_addr: u32,
    original_addr: u32,
    output_mangled: bool,
    addr_description: &str,
) -> Result<(), String> {
    // Get the image
    let image = info
        .get_rom_set_image(set)
        .ok_or_else(|| format!("No ROM set found for set number {}", set))?;

    // Lookup the address in the image
    let byte = image[lookup_addr as usize];

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
        if lookup_addr != original_addr {
            println!("Mangled address 0x{:04X}", lookup_addr);
        }
    }

    if output_mangled {
        println!(
            "{} 0x{:04X}: 0x{:02X} (mangled byte)",
            addr_description, original_addr, byte
        );
    } else {
        let demangled_byte = info.demangle_byte(byte);
        println!(
            "{} 0x{:04X}: 0x{:02X} (demangled byte)",
            addr_description, original_addr, demangled_byte
        );
    }

    Ok(())
}

fn lookup_raw(info: &SdrrInfo, args: &Args) {
    println!("Lookup Byte Using Raw (mangled) Address");
    println!("---------------------------------------");

    // Ensure we have the arguments
    let set = args.set.expect("Internal error: set number is required");
    let addr = args.addr.expect("Internal error: address is required");
    let output_mangled = args
        .output_mangled
        .expect("Internal error: output_mangled is required");

    if let Err(e) = lookup_byte_at_address(info, args.detail, set, addr, addr, output_mangled, "Mangled address")
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn lookup_range(
    info: &SdrrInfo,
    detail: bool,
    set: u8,
    start_addr: u32,
    end_addr: u32,
    output_mangled: bool,
    output_binary: bool,
    cs1: bool,
    cs2: Option<bool>,
    cs3: Option<bool>,
    x1: Option<bool>,
    x2: Option<bool>,
) -> Result<(), String> {
    let image = info
        .get_rom_set_image(set)
        .ok_or_else(|| format!("No ROM set found for set number {}", set))?;

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
        // Collect bytes for binary output
        let mut binary_data = Vec::new();

        for addr in start_addr..=end_addr {
            let lookup_addr = info.mangle_address(addr, cs1, cs2, cs3, x1, x2);
            let byte = image[lookup_addr as usize];

            let output_byte = if output_mangled {
                byte
            } else {
                info.demangle_byte(byte)
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
            let lookup_addr = info.mangle_address(addr, cs1, cs2, cs3, x1, x2);
            let byte = image[lookup_addr as usize];

            let output_byte = if output_mangled {
                byte
            } else {
                info.demangle_byte(byte)
            };

            let byte_pos = (addr - start_addr) as usize;

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

fn lookup(info: &SdrrInfo, args: &Args) {
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

    if let Some((start_addr, end_addr)) = args.range {
        // Range lookup
        let output_binary = args.output_binary.unwrap_or(false);
        if let Err(e) = lookup_range(
            info,
            args.detail,
            set,
            start_addr,
            end_addr,
            output_mangled,
            output_binary,
            cs1,
            cs2,
            cs3,
            x1,
            x2,
        ) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        // Single address lookup
        let addr = args.addr.expect("Internal error: address is required");
        let lookup_addr = info.mangle_address(addr, cs1, cs2, cs3, x1, x2);

        if let Err(e) = lookup_byte_at_address(
            info,
            args.detail,
            set,
            lookup_addr,
            addr,
            output_mangled,
            "Actual address",
        ) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
