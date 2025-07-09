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

use anyhow::Result;

mod symbols;
use symbols::SdrrInfo;
mod load;
use load::load_sdrr_firmware;
mod args;
use args::{parse_args, Command, Args};

// SDRR info structure offset in firmware binary
pub const SDRR_INFO_OFFSET: usize = 0x200;

// STM32F4 flash base address
pub const STM32F4_FLASH_BASE: u32 = 0x08000000;

// Example usage function
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Software Defined Retro ROM - Firmware Information");
    println!("=================================================");
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let firmware_path = &args.firmware;
    let info = match load_sdrr_firmware(firmware_path) {
        Ok(info) => {
            info
        }
        Err(e) => {
            eprintln!("Error loading firmware");
            println!("Did you supply an SDRR v0.1.1 or later .elf or .bin file?");
            println!("Detailed error: {}", e);
            std::process::exit(1);
        }
    };

    match args.command {
        Command::Info => print_sdrr_info(&info, &args),
        Command::LookupRaw => lookup_raw(&info, &args),
        Command::Lookup => lookup(&info, &args),
    }

    Ok(())
}

fn print_sdrr_info(info: &SdrrInfo, _args: &Args) {
    println!();
    println!("Core Firmware Properties");
    println!("------------------------");
    println!("File type:     {}", info.file_type);
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
    println!();
    println!("ROM Sets: {}", info.rom_set_count);
    println!("-----------");

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
            println!(
                "    CS Lines:  {}/{}/{}",
                rom.cs1_line, rom.cs2_line, rom.cs3_line
            );
        }
    }
}

fn lookup_byte_at_address(
    info: &SdrrInfo, 
    set: u8, 
    lookup_addr: u32, 
    original_addr: u32,
    output_mangled: bool,
    addr_description: &str
) -> Result<(), String> {
    // Get the image
    let image = info.get_rom_set_image(set)
        .ok_or_else(|| format!("No ROM set found for set number {}", set))?;

    // Lookup the address in the image
    let byte = image[lookup_addr as usize];

    // Get ROM names
    let roms: Vec<String> = info
        .rom_sets[set as usize]
        .roms
        .iter()
        .map(|rom| rom.filename.clone().unwrap_or_else(|| "<unknown>".to_string()))
        .collect();
    let rom_name = roms.join(", ");
    
    println!("Byte lookup ROM set {} ({})", set, rom_name);
    
    if lookup_addr != original_addr {
        println!("Mangled address 0x{:04X}", lookup_addr);
    }
    
    if output_mangled {
        println!("{} 0x{:04X}: 0x{:02X} (mangled byte)", addr_description, original_addr, byte);
    } else {
        let demangled_byte = info.demangle_byte(byte);
        println!("{} 0x{:04X}: 0x{:02X} (demangled byte)", addr_description, original_addr, demangled_byte);
    }
    
    Ok(())
}

fn lookup_raw(info: &SdrrInfo, args: &Args) {
    println!("Lookup Byte Using Raw (mangled) Address");
    println!("---------------------------------------");

    // Ensure we have the arguments
    let set = args.set.expect("Internal error: set number is required");
    let addr = args.addr.expect("Internal error: address is required");
    let output_mangled = args.output_mangled.expect("Internal error: output_mangled is required");

    if let Err(e) = lookup_byte_at_address(info, set, addr, addr, output_mangled, "Mangled address") {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn lookup_range(
    info: &SdrrInfo,
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
    let image = info.get_rom_set_image(set)
        .ok_or_else(|| format!("No ROM set found for set number {}", set))?;

    let roms: Vec<String> = info
        .rom_sets[set as usize]
        .roms
        .iter()
        .map(|rom| rom.filename.clone().unwrap_or_else(|| "<unknown>".to_string()))
        .collect();
    let rom_name = roms.join(", ");
    
    println!("Byte lookup ROM set {} ({})", set, rom_name);
    println!("Address range 0x{:04X} to 0x{:04X}:", start_addr, end_addr);
    
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
        
        // Write binary data to file
        let filename = "/tmp/rom.bin";
        std::fs::write(filename, &binary_data)
            .map_err(|e| format!("Failed to write binary file {}: {}", filename, e))?;
        
        println!("Binary data written to {}", filename);
    } else {
        // Hex dump output
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
    println!("Lookup Byte Using Real (non-mangled) Address");
    println!("--------------------------------------------");

    // Ensure we have the arguments
    let set = args.set.expect("Internal error: set number is required");
    let output_mangled = args.output_mangled.expect("Internal error: output_mangled is required");
    let _output_binary = args.output_binary.expect("Internal error: output_binary is required");
    let cs1 = args.cs1.expect("Internal error: cs1 is required");
    let cs2 = args.cs2;
    let cs3 = args.cs3;
    let x1 = args.x1;
    let x2 = args.x2;

    if let Some((start_addr, end_addr)) = args.range {
        // Range lookup
        let output_binary = args.output_binary.unwrap_or(false);
        if let Err(e) = lookup_range(info, set, start_addr, end_addr, output_mangled, output_binary, cs1, cs2, cs3, x1, x2) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        // Single address lookup
        let addr = args.addr.expect("Internal error: address is required");
        let lookup_addr = info.mangle_address(addr, cs1, cs2, cs3, x1, x2);

        if let Err(e) = lookup_byte_at_address(info, set, lookup_addr, addr, output_mangled, "Actual address") {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}