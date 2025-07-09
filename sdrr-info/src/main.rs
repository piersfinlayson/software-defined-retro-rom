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

// SDRR info structure offset in firmware binary
pub const SDRR_INFO_OFFSET: usize = 0x200;

// STM32F4 flash base address
pub const STM32F4_FLASH_BASE: u32 = 0x08000000;

// Example usage function
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <firmware.elf|firmware.bin>", args[0]);
        std::process::exit(1);
    }

    println!("Software Defined Retro ROM - Firmware Information");
    println!("=================================================");

    let firmware_path = &args[1];
    match load_sdrr_firmware(firmware_path) {
        Ok(info) => {
            print_sdrr_info(&info);
        }
        Err(e) => {
            println!("Error loading firmware");
            println!("Did you supply an SDRR v0.1.1 or later .elf or .bin file?");
            println!("Detailed error: {}", e);
        }
    }

    Ok(())
}

fn print_sdrr_info(info: &SdrrInfo) {
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
