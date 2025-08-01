// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

//! sdrr-gen - Handles command-line arguments for the SDRR generator
//!
//! Generates a configuration based on user input, including hardware
//! revision, STM32 variant, ROM configurations, and various options for
//! logging, preloading image to RAM, and more.

use clap::Parser;
use std::path::PathBuf;

use sdrr_common::args::{parse_hw_rev, parse_serve_alg, parse_stm_variant};
use sdrr_common::{CsLogic, HwConfig, RomType, ServeAlg, StmVariant};

use crate::config::{Config, CsConfig, RomConfig, SizeHandling};
use crate::file::{FileSource, check_output_dir, source_image_file};

#[derive(Parser, Debug)]
#[clap(
    name = "sdrr-gen",
    about = "Software Defined Retro ROM configuration generator",
    version
)]
pub struct Args {
    /// ROM configuration (file=path,type=2364,cs1=0)
    #[clap(long, alias = "rom-config", required_unless_present = "list_hw_revs")]
    rom: Vec<String>,

    /// STM32 variant (f446rc, f446re, f411rc, f411re, f405rg, f401re, f401rb, f401rc)
    #[clap(long, required_unless_present = "list_hw_revs", value_parser = parse_stm_variant)]
    stm: Option<StmVariant>,

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

    /// Enable main loop one shot
    #[clap(long, requires = "main_loop_logging")]
    main_loop_one_shot: bool,

    /// Enable debug logging
    #[clap(long, requires = "boot_logging")]
    debug_logging: bool,

    /// Output directory
    #[clap(
        long,
        alias = "output",
        alias = "output-dir",
        default_value = "./output"
    )]
    output_dir: PathBuf,

    /// Overwrite existing files
    #[clap(long)]
    overwrite: bool,

    /// Use internal oscillator (default)
    #[clap(long)]
    hsi: bool,

    /// Use external oscillator
    #[clap(long, conflicts_with = "hsi")]
    hse: bool,

    /// Hardware revision (use --list-hw-revs for options)
    #[clap(long, alias= "hw-rev", value_parser = parse_hw_rev, required_unless_present = "list_hw_revs")]
    hw: Option<HwConfig>,

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
    #[clap(long, value_parser = parse_serve_alg)]
    serve_alg: Option<ServeAlg>,

    /// List available hardware revisions
    #[clap(long, default_value = "false")]
    list_hw_revs: bool,

    /// Use to enable SDRR's ROM access count functionality.
    #[clap(long, default_value = "false")]
    count_rom_access: bool,
}

impl Args {
    /// Returns the list hw revs flags
    pub fn list_hw_revs(&self) -> bool {
        self.list_hw_revs
    }

    /// Returns the ROM configuration
    fn rom_config(&self) -> &Vec<String> {
        &self.rom
    }

    /// Returns the STM variant desired
    fn stm_variant(&self) -> Option<StmVariant> {
        self.stm
    }

    /// Returns whether SWD should be enabled
    fn swd(&self) -> bool {
        self.swd
    }

    /// Returns whether MCO should be enabled
    fn mco(&self) -> bool {
        self.mco
    }

    /// Returns whether MCO2 should be enabled
    fn mco2(&self) -> bool {
        self.mco2
    }

    /// Returns whether boot logging should be enabled
    fn boot_logging(&self) -> bool {
        self.boot_logging
    }

    /// Returns whether main loop logging should be enabled
    fn main_loop_logging(&self) -> bool {
        self.main_loop_logging
    }

    /// Returns whether main loop one shot should be enabled
    fn main_loop_one_shot(&self) -> bool {
        self.main_loop_one_shot
    }

    /// Returns whether debug logging should be enabled
    fn debug_logging(&self) -> bool {
        self.debug_logging
    }

    /// Returns whether the output directory can be overwritten
    fn can_overwrite(&self) -> bool {
        self.overwrite
    }

    /// Returns whether HSE should be used
    fn hse(&self) -> bool {
        self.hse
    }

    /// Returns the hardware configuration
    fn hw_config(&self) -> HwConfig {
        self.hw
            .clone()
            .expect("Internal error - hardware configrureation must be set")
    }

    /// Returns whether the status LED should be supported
    fn status_led(&self) -> bool {
        self.status_led
    }

    /// Returns whether overclocking should be supported
    fn overclock(&self) -> bool {
        self.overclock
    }

    /// Returns whether bootloader mode should be supported
    fn bootloader(&self) -> bool {
        self.bootloader
    }

    /// Returns whether preloading should be supported
    fn preload_to_ram(&self) -> bool {
        !self.disable_preload_to_ram
    }

    /// Returns whether the user wants to auto answer yes to  licence terms
    fn auto_yes(&self) -> bool {
        self.yes
    }

    /// Returns the serve algorithm to use
    fn serve_alg(&self) -> ServeAlg {
        self.serve_alg.unwrap_or(ServeAlg::Default)
    }

    /// Returns the STM variant and the desired frequency in MHz.  The
    /// frequency is either user-specified, or, if unspecified, the maximum
    /// frequency for the STM32 variant.
    fn stm_and_freq(&self) -> Result<(StmVariant, u32), String> {
        let stm_variant = self
            .stm_variant()
            .ok_or("STM32 variant must be specified")?;
        let freq = self
            .freq
            .unwrap_or_else(|| stm_variant.processor().max_sysclk_mhz());
        Ok((stm_variant, freq))
    }

    /// Parse the ROM configuration arguments into a vector of `RomConfig`.
    pub fn parse_rom_args(&self) -> Result<Vec<RomConfig>, String> {
        let mut roms = Vec::new();

        for (rom_num, rom_config_str) in self.rom_config().iter().enumerate() {
            let rom_config = self.parse_rom_arg(rom_num, rom_config_str).map_err(|e| {
                format!("ROM #{rom_num} configuration error: {e} (config: {rom_config_str})")
            })?;
            roms.push(rom_config);
        }

        Ok(roms)
    }

    /// Returns the entire sdrr-gen configuration object.
    pub fn gen_config(&self) -> Result<Config, String> {
        // Check whether output directory exists and if so, if user is happy to
        // overwrite its content.
        check_output_dir(&self.output_dir, self.can_overwrite()).map_err(|e| e.to_string())?;

        // Parse the ROM arguments
        let roms = self.parse_rom_args()?;

        // Get the STM variant and frequency
        let (stm_variant, freq) = self.stm_and_freq()?;

        // Return the config object
        Ok(Config {
            roms,
            stm_variant,
            output_dir: self.output_dir.clone(),
            swd: self.swd(),
            count_rom_access: self.count_rom_access,
            mco: self.mco(),
            mco2: self.mco2(),
            boot_logging: self.boot_logging(),
            main_loop_logging: self.main_loop_logging(),
            main_loop_one_shot: self.main_loop_one_shot(),
            debug_logging: self.debug_logging(),
            overwrite: self.can_overwrite(),
            hse: self.hse(),
            hw: self.hw_config(),
            freq,
            status_led: self.status_led(),
            overclock: self.overclock(),
            bootloader: self.bootloader(),
            preload_to_ram: self.preload_to_ram(),
            auto_yes: self.auto_yes(),
            serve_alg: self.serve_alg(),
        })
    }

    fn parse_rom_arg(&self, rom_num: usize, s: &str) -> Result<RomConfig, String> {
        let mut original_file_source = None;
        let mut extract = None;
        let mut licence = None;
        let mut rom_type = None;
        let mut cs1 = None;
        let mut cs2 = None;
        let mut cs3 = None;
        let mut size_handling = SizeHandling::None;
        let mut set = None;
        let mut bank = None;

        for pair in s.split(',') {
            let parts: Vec<&str> = pair.split('=').collect();

            match parts[0] {
                "set" => {
                    if parts.len() != 2 {
                        return Err(
                            "Invalid 'set' parameter format - must include set number".to_string()
                        );
                    }
                    let set_num: usize = parts[1]
                        .parse()
                        .map_err(|_| format!("Invalid set number: {}", parts[1]))?;
                    set = Some(set_num);
                }
                "bank" => {
                    if parts.len() != 2 {
                        return Err("Invalid 'bank' parameter format - must include bank number"
                            .to_string());
                    }
                    let bank_num: usize = parts[1]
                        .parse()
                        .map_err(|_| format!("Invalid bank number: {}", parts[1]))?;
                    bank = Some(bank_num);
                }
                "file" => {
                    let original_source = parts[1].to_string();
                    original_file_source =
                        if parts[1].starts_with("http://") || parts[1].starts_with("https://") {
                            // Will be handled after we know if extract is specified
                            Some(FileSource::Url(original_source))
                        } else {
                            Some(FileSource::Local(PathBuf::from(parts[1])))
                        };
                }
                "extract" => {
                    if parts.len() != 2 {
                        return Err("Invalid 'extract' parameter format - must include filename"
                            .to_string());
                    }
                    if extract.is_some() {
                        return Err("extract specified multiple times".to_string());
                    }
                    extract = Some(parts[1].to_string());
                }
                "licence" => {
                    if parts.len() != 2 {
                        return Err(
                            "Invalid 'licence' parameter format - must include URL".to_string()
                        );
                    }
                    if licence.is_some() {
                        return Err("licence specified multiple times".to_string());
                    }
                    licence = Some(parts[1].to_string());
                }
                "type" => {
                    if parts.len() != 2 {
                        return Err(
                            "Invalid 'type' parameter format - must include type".to_string()
                        );
                    }
                    rom_type = RomType::try_from_str(parts[1])
                        .map(Some)
                        .ok_or_else(|| format!("Invalid ROM type: {}", parts[1]))?
                }
                "cs1" => {
                    if parts.len() != 2 {
                        return Err(
                            "Invalid 'cs1' parameter format - must include cs1 value".to_string()
                        );
                    }
                    if cs1.is_some() {
                        return Err("cs1 specified multiple times".to_string());
                    }
                    cs1 = CsLogic::try_from_str(parts[1]).map(Some).ok_or_else(|| {
                        format!("Invalid cs1 value: {} (use 0, 1, or ignore)", parts[1])
                    })?
                }
                "cs2" => {
                    if parts.len() != 2 {
                        return Err(
                            "Invalid 'cs2' parameter format - must include cs2 value".to_string()
                        );
                    }
                    if cs2.is_some() {
                        return Err("cs2 specified multiple times".to_string());
                    }
                    cs2 = CsLogic::try_from_str(parts[1]).map(Some).ok_or_else(|| {
                        format!("Invalid cs2 value: {} (use 0, 1, or ignore)", parts[1])
                    })?
                }
                "cs3" => {
                    if parts.len() != 2 {
                        return Err(
                            "Invalid 'cs3' parameter format - must include cs3 value".to_string()
                        );
                    }
                    if cs3.is_some() {
                        return Err("cs3 specified multiple times".to_string());
                    }
                    cs3 = CsLogic::try_from_str(parts[1]).map(Some).ok_or_else(|| {
                        format!("Invalid cs3 value: {} (use 0, 1, or ignore)", parts[1])
                    })?
                }
                "dup" => {
                    if parts.len() != 1 {
                        return Err(
                            "Invalid 'dup' parameter format - doesn't take a value".to_string()
                        );
                    }
                    if !matches!(size_handling, SizeHandling::None) {
                        return Err("Cannot specify both 'dup' and 'pad'".to_string());
                    }
                    size_handling = SizeHandling::Duplicate;
                }
                "pad" => {
                    if parts.len() != 1 {
                        return Err(
                            "Invalid 'pad' parameter format - doesn't take a value".to_string()
                        );
                    }
                    if !matches!(size_handling, SizeHandling::None) {
                        return Err("Cannot specify both 'dup' and 'pad'".to_string());
                    }
                    size_handling = SizeHandling::Pad;
                }
                _ => return Err(format!("Unknown key: {}", parts[0])),
            }
        }

        // Update the file source if extract is specified
        let source = original_file_source.ok_or("Missing 'file' parameter".to_string())?;
        let source = if let Some(extract) = extract.as_ref() {
            match source {
                FileSource::Local(_) => {
                    return Err("Zip extraction from local file not supported".to_string());
                }
                FileSource::Url(url) => FileSource::UrlZip(url, extract.to_string()),
                FileSource::UrlZip(_, _) => {
                    unreachable!("Internal error - Have UrlZip before we should")
                }
            }
        } else {
            source
        };

        let file = source_image_file(rom_num, &source, &self.output_dir)?;

        let rom_type = rom_type.ok_or("Missing 'type' parameter")?;
        let cs1 = cs1.ok_or("Missing 'cs1' parameter")?;

        Ok(RomConfig {
            file,
            original_source: source.to_string(),
            extract,
            licence,
            rom_type,
            cs_config: CsConfig::new(cs1, cs2, cs3),
            size_handling,
            set,
            bank,
        })
    }
}
