// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

mod config;
mod generator;
mod preprocessor;
mod rom_types;

use crate::config::{Config, CsConfig, SizeHandling};
use crate::generator::generate_files;
use crate::rom_types::{CsLogic, RomType, StmVariant, ServeAlg};
use anyhow::{Context, Result};
use clap::Parser;
use preprocessor::RomImage;
use rom_types::HwRev;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempPath};
use urlencoding::decode;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[clap(
    name = "sdrr-gen",
    about = "Software Defined Retro ROM configuration generator",
    version
)]
struct Args {
    /// ROM configuration (file=path,type=2364,cs1=0)
    #[clap(long, required = true)]
    rom: Vec<String>,

    /// STM32 variant (f446rc, f446re, f411rc, f411re, f405rg, f401re, f401rb, f401rc)
    #[clap(long, value_parser = parse_stm_variant)]
    stm: StmVariant,

    /// Enable SWD
    #[clap(long)]
    swd: bool,

    /// Enable MCO
    #[clap(long)]
    mco: bool,

    /// Enable MCO2 (STM32F4xx only)
    #[clap(long, requires = "mco")]
    mco2: bool,

    /// Enable boot logging
    #[clap(long, requires = "swd")]
    boot_logging: bool,

    /// Enable main loop logging
    #[clap(long, requires = "boot_logging")]
    main_loop_logging: bool,

    /// Enable debug logging
    #[clap(long, requires = "boot_logging")]
    debug_logging: bool,

    /// Output directory
    #[clap(long, default_value = "./output")]
    output: PathBuf,

    /// Overwrite existing files
    #[clap(long)]
    overwrite: bool,

    /// Use internal oscillator (default)
    #[clap(long)]
    hsi: bool,

    /// Use external oscillator
    #[clap(long, conflicts_with = "hsi")]
    hse: bool,

    /// Hardware revision (a, b, c, d, e, f)
    #[clap(long, value_parser = parse_hw_rev)]
    hw_rev: Option<HwRev>,

    /// Target frequency in MHz (default: max for the variant)
    #[clap(long)]
    freq: Option<u32>,

    /// Support the status LED
    #[clap(long)]
    status_led: bool,

    /// Support overclocking the processor (STM32F4xx only)
    #[clap(long)]
    overclock: bool,

    /// Support entering bootloader mode when all select jumpers as closed
    #[clap(long)]
    bootloader: bool,

    /// Disable preloading of ROMs to RAM
    #[clap(long)]
    disable_preload_to_ram: bool,

    /// Automatically answer [y]es to questions
    #[clap(long, short = 'y')]
    yes: bool,

    /// Byte serving algorithm to choose (default, a = 2 CS 1 Addr, b = Addr on CS)
    #[clap(long, value_parser = perse_serve_alg)]
    serve_alg: Option<ServeAlg>,
}

fn parse_stm_variant(s: &str) -> Result<StmVariant, String> {
    StmVariant::from_str(s)
        .ok_or_else(|| format!("Invalid STM32 variant: {}. Valid values are: f446rc, f446re, f411rc, f411re, f405rg, f401re, f401rb, f401rc", s))
}

fn parse_hw_rev(s: &str) -> Result<HwRev, String> {
    HwRev::from_str(s).ok_or_else(|| {
        format!(
            "Invalid hardware revision: {}. Valid values are: a, b, c, d, e, f",
            s
        )
    })
}

fn perse_serve_alg(s: &str) -> Result<ServeAlg, String> {
    ServeAlg::from_str(s).ok_or_else(|| {
        format!(
            "Invalid serve algorithm: {}. Valid values are: default, a (2 CS 1 Addr), b (Addr on CS)",
            s
        )
    })
}

fn download_url_to_temp(url: &str) -> Result<(PathBuf, TempPath), String> {
    println!("Downloading {}", url);

    let response =
        reqwest::blocking::get(url).map_err(|e| format!("Failed to download {}: {}", url, e))?;

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let mut temp_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;

    temp_file
        .write_all(&bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let temp_path = temp_file.into_temp_path();
    let path = temp_path.to_path_buf();

    Ok((path, temp_path))
}

fn download_and_extract_zip(url: &str, extract_file: &str) -> Result<(PathBuf, TempPath), String> {
    // URL decode the extract filename to handle spaces and special characters
    let decoded_extract_file = decode(extract_file).map_err(|e| {
        format!(
            "Failed to URL decode extract filename '{}': {}",
            extract_file, e
        )
    })?;

    println!(
        "Downloading and extracting {} from {}",
        decoded_extract_file, url
    );

    let response =
        reqwest::blocking::get(url).map_err(|e| format!("Failed to download {}: {}", url, e))?;

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip archive: {}", e))?;

    // First, collect all filenames and check if our target exists
    let mut file_names = Vec::new();
    let mut target_exists = false;

    for i in 0..archive.len() {
        if let Ok(f) = archive.by_index(i) {
            let name = f.name().to_string();
            if name == decoded_extract_file {
                target_exists = true;
            }
            file_names.push(name);
        }
    }

    if !target_exists {
        println!(
            "Failed to find '{}' in zip. Archive contents:",
            decoded_extract_file
        );
        for name in &file_names {
            println!("  '{}'", name);
        }
        return Err(format!("Failed to find {} in zip", decoded_extract_file));
    }

    // Now we know the file exists, extract it
    let mut file = archive
        .by_name(&decoded_extract_file)
        .map_err(|e| format!("Failed to extract {}: {}", decoded_extract_file, e))?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| format!("Failed to read {} from zip: {}", extract_file, e))?;

    let mut temp_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;

    temp_file
        .write_all(&contents)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let temp_path = temp_file.into_temp_path();
    let path = temp_path.to_path_buf();

    Ok((path, temp_path))
}

fn parse_rom_config(s: &str) -> Result<(config::RomConfig, Vec<TempPath>), String> {
    let mut temp_handles = Vec::new();
    let mut file = None;
    let mut original_file_source = None;
    let mut extract = None;
    let mut licence = None;
    let mut rom_type = None;
    let mut cs1 = None;
    let mut cs2 = None;
    let mut cs3 = None;
    let mut size_handling = SizeHandling::None;
    let mut set = None;

    for pair in s.split(',') {
        let parts: Vec<&str> = pair.split('=').collect();

        match parts[0] {
            "set" => {
                if parts.len() != 2 {
                    return Err("Invalid 'set' parameter format - must include set number".to_string());
                }
                let set_num: usize = parts[1].parse()
                    .map_err(|_| format!("Invalid set number: {}", parts[1]))?;
                set = Some(set_num);
            }
            "file" => {
                let original_source = parts[1].to_string();
                if parts[1].starts_with("http://") || parts[1].starts_with("https://") {
                    // Will be handled after we know if extract is specified
                    original_file_source = Some(original_source);
                } else {
                    file = Some(PathBuf::from(parts[1]));
                    original_file_source = Some(original_source);
                }
            }
            "extract" => {
                if parts.len() != 2 {
                    return Err(
                        "Invalid 'extract' parameter format - must include filename".to_string()
                    );
                }
                if extract.is_some() {
                    return Err("extract specified multiple times".to_string());
                }
                extract = Some(parts[1].to_string());
            }
            "licence" => {
                if parts.len() != 2 {
                    return Err("Invalid 'licence' parameter format - must include URL".to_string());
                }
                if licence.is_some() {
                    return Err("licence specified multiple times".to_string());
                }
                licence = Some(parts[1].to_string());
            }
            "type" => {
                if parts.len() != 2 {
                    return Err("Invalid 'type' parameter format - must include type".to_string());
                }
                rom_type = RomType::from_str(parts[1])
                    .map(Some)
                    .ok_or_else(|| format!("Invalid ROM type: {}", parts[1]))?
            }
            "cs1" => {
                if parts.len() != 2 {
                    return Err("Invalid 'cs1' parameter format - must include cs1 value".to_string());
                }
                if cs1.is_some() {
                    return Err("cs1 specified multiple times".to_string());
                }
                cs1 = CsLogic::from_str(parts[1])
                    .map(Some)
                    .ok_or_else(|| format!("Invalid cs1 value: {} (use 0, 1, or ignore)", parts[1]))?
            }
            "cs2" => {
                if parts.len() != 2 {
                    return Err("Invalid 'cs2' parameter format - must include cs2 value".to_string());
                }
                if cs2.is_some() {
                    return Err("cs2 specified multiple times".to_string());
                }
                cs2 = CsLogic::from_str(parts[1])
                    .map(Some)
                    .ok_or_else(|| format!("Invalid cs2 value: {} (use 0, 1, or ignore)", parts[1]))?
            }
            "cs3" => {
                if parts.len() != 2 {
                    return Err("Invalid 'cs3' parameter format - must include cs3 value".to_string());
                }
                if cs3.is_some() {
                    return Err("cs3 specified multiple times".to_string());
                }
                cs3 = CsLogic::from_str(parts[1])
                    .map(Some)
                    .ok_or_else(|| format!("Invalid cs3 value: {} (use 0, 1, or ignore)", parts[1]))?
            }
            "dup" => {
                if parts.len() != 1 {
                    return Err("Invalid 'dup' parameter format - doesn't take a value".to_string());
                }
                if !matches!(size_handling, SizeHandling::None) {
                    return Err("Cannot specify both 'dup' and 'pad'".to_string());
                }
                size_handling = SizeHandling::Duplicate;
            }
            "pad" => {
                if parts.len() != 1 {
                    return Err("Invalid 'pad' parameter format - doesn't take a value".to_string());
                }
                if !matches!(size_handling, SizeHandling::None) {
                    return Err("Cannot specify both 'dup' and 'pad'".to_string());
                }
                size_handling = SizeHandling::Pad;
            }
            _ => return Err(format!("Unknown key: {}", parts[0])),
        }
    }

    // Handle URL downloading with optional zip extraction
    if let Some(ref source) = original_file_source {
        if source.starts_with("http://") || source.starts_with("https://") {
            if let Some(ref extract_file) = extract {
                let (path, temp_handle) = download_and_extract_zip(source, extract_file)?;
                file = Some(path);
                temp_handles.push(temp_handle);
            } else {
                let (path, temp_handle) = download_url_to_temp(source)?;
                file = Some(path);
                temp_handles.push(temp_handle);
            }
        }
    }

    let file = file.ok_or("Missing 'file' parameter")?; // Add this line
    let rom_type = rom_type.ok_or("Missing 'type' parameter")?;
    let cs1 = cs1.ok_or("Missing 'cs1' parameter")?;

    Ok((
        config::RomConfig {
            file, // Now this is PathBuf, not Option<PathBuf>
            original_source: original_file_source.unwrap(),
            extract,
            licence,
            rom_type,
            cs_config: CsConfig::new(cs1, cs2, cs3),
            size_handling,
            set,
        },
        temp_handles,
    ))
}

fn confirm_licences(config: &Config) -> Result<()> {
    let licensed_roms: Vec<_> = config
        .roms
        .iter()
        .filter(|rom| rom.licence.is_some())
        .collect();

    if licensed_roms.is_empty() {
        return Ok(());
    }

    println!("Some ROM images require licence acceptance:");
    println!();

    for rom in &licensed_roms {
        let source = rom.extract.as_ref().unwrap_or(&rom.original_source);
        let filename = source.split('/').last().unwrap_or(source);

        println!("ROM: {}", filename);
        println!("Licence: {}", rom.licence.as_ref().unwrap());
        println!();
    }

    print!("Do you accept the licence terms for the above ROM(s)? (y/n): ");
    io::stdout().flush().unwrap();

    if config.auto_yes {
        println!("Automatically accepting licence terms due to --yes flag.");
        return Ok(());
    }
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read user input: {}", e))?;

    let response = input.trim().to_lowercase();
    if response != "y" && response != "yes" {
        return Err(anyhow::anyhow!("licence terms not accepted. Aborting."));
    }

    println!("Licence terms accepted. Continuing...");
    println!();
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse ROM configurations
    let mut all_temp_handles = Vec::new();
    let mut roms = Vec::new();
    for (i, rom_config_str) in args.rom.iter().enumerate() {
        let (rom_config, mut temp_handles) = parse_rom_config(rom_config_str).map_err(|e| {
            anyhow::anyhow!(
                "ROM #{} configuration error: {} (config: {})",
                i + 1,
                e,
                rom_config_str
            )
        })?;
        roms.push(rom_config);
        all_temp_handles.append(&mut temp_handles);
    }

    // Set the frequency based on the STM32 variant or user input
    let freq = args
        .freq
        .unwrap_or_else(|| args.stm.processor().max_sysclk_mhz());

    // Create configuration
    let mut config = Config {
        roms,
        stm_variant: args.stm,
        output_dir: args.output,
        swd: args.swd,
        mco: args.mco,
        mco2: args.mco2,
        boot_logging: args.boot_logging,
        main_loop_logging: args.main_loop_logging,
        debug_logging: args.debug_logging,
        overwrite: args.overwrite,
        hse: args.hse,
        hw_rev: args.hw_rev,
        freq,
        status_led: args.status_led,
        overclock: args.overclock,
        bootloader: args.bootloader,
        preload_to_ram: !args.disable_preload_to_ram,
        auto_yes: args.yes,
        serve_alg: args.serve_alg.unwrap_or(ServeAlg::Default),
    };

    // Validate it
    config
        .validate()
        .map_err(|e| anyhow::anyhow!("Configuration error: {}", e))?;

    // Validate output directory
    if !config.overwrite && config.output_dir.exists() {
        for file_name in &["roms.h", "roms.c", "sdrr_config.h", "linker.ld"] {
            let file_path = config.output_dir.join(file_name);
            if file_path.exists() {
                return Err(anyhow::anyhow!(
                    "Output file '{}' already exists. Use --overwrite to overwrite.",
                    file_path.display()
                ));
            }
        }
    }

    // Check and confirm licences before proceeding
    confirm_licences(&config)?;

    // Load ROM files
    let mut rom_images = Vec::new();
    for (_, rom_config) in config.roms.iter().enumerate() {
        let rom_image = RomImage::load_from_file(
            &rom_config.file,
            &rom_config.rom_type,
            &rom_config.size_handling,
        )
        .with_context(|| {
            format!(
                "Failed to process ROM image: {}",
                rom_config.original_source
            )
        })?;
        rom_images.push(rom_image);
    }

    // Create ROM sets (no additional validation needed)
    let rom_sets = config
        .create_rom_sets(&rom_images)
        .map_err(|e| anyhow::anyhow!("ROM set creation error: {}", e))?;

    println!("Successfully loaded {} ROM file(s) in {} set(s)", rom_images.len(), rom_sets.len());

    // Generate output files
    generate_files(&config, &rom_sets).with_context(|| "Failed to generate output files")?;

    println!(
        "Successfully transformed ROM images and generated output files in {}",
        config.output_dir.display()
    );

    Ok(())
}
