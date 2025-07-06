use crate::rom_types::{CsLogic, HwRev, RomType, StmFamily, StmVariant, StmProcessor};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub roms: Vec<RomConfig>,
    pub stm_variant: StmVariant,
    pub output_dir: PathBuf,
    pub swd: bool,
    pub mco: bool,
    pub mco2: bool,
    pub boot_logging: bool,
    pub main_loop_logging: bool,
    pub debug_logging: bool,
    pub overwrite: bool,
    pub hse: bool,
    pub hw_rev: Option<HwRev>,
    pub freq: u32,
    pub status_led: bool,
    pub overclock: bool,
    pub bootloader: bool,
    pub preload_to_ram: bool,
    pub auto_yes: bool,
}

#[derive(Debug, Clone)]
pub enum SizeHandling {
    None,
    Duplicate,
    Pad,
}

#[derive(Debug, Clone)]
pub struct RomConfig {
    pub file: PathBuf,
    pub original_source: String,
    pub extract: Option<String>,
    pub licence: Option<String>,
    pub rom_type: RomType,
    pub cs_config: CsConfig,
    pub size_handling: SizeHandling,
}

#[derive(Debug, Clone)]
pub struct CsConfig {
    pub cs1: CsLogic,
    pub cs2: Option<CsLogic>,
    pub cs3: Option<CsLogic>,
}

impl CsConfig {
    pub fn new(cs1: CsLogic, cs2: Option<CsLogic>, cs3: Option<CsLogic>) -> Self {
        Self { cs1, cs2, cs3 }
    }

    pub fn validate(&self, rom_type: &RomType) -> Result<(), String> {
        let required_cs_lines = rom_type.cs_lines_count();

        match rom_type {
            RomType::Rom2364 => {
                // 2364 requires only CS1 (1 CS line)
                if self.cs2.is_some() || self.cs3.is_some() {
                    return Err(format!(
                        "ROM type {} only supports {} CS line(s)",
                        rom_type.name(),
                        required_cs_lines
                    ));
                }
            }
            RomType::Rom2332 => {
                // 2332 requires CS1 and CS2 (2 CS lines)
                if self.cs2.is_none() {
                    return Err(format!(
                        "ROM type {} requires {} CS line(s)",
                        rom_type.name(),
                        required_cs_lines
                    ));
                }
                if self.cs3.is_some() {
                    return Err(format!("ROM type {} does not support CS3", rom_type.name()));
                }
            }
            RomType::Rom2316 => {
                // 2316 requires CS1, CS2, and CS3 (3 CS lines)
                if self.cs2.is_none() || self.cs3.is_none() {
                    return Err(format!(
                        "ROM type {} requires {} CS line(s)",
                        rom_type.name(),
                        required_cs_lines
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Config {
    pub fn validate(&mut self) -> Result<(), String> {
        // Validate at least one ROM
        if self.roms.is_empty() {
            return Err("At least one ROM must be provided".to_string());
        }

        // Validate each ROM configuration
        for rom in &self.roms {
            rom.cs_config
                .validate(&rom.rom_type)
                .inspect_err(|_| println!("Failed to process ROM {}", rom.file.display()))?;
        }

        // Validate output directory
        if !self.overwrite && self.output_dir.exists() {
            for file_name in &["roms.h", "roms.c", "config.h", "sdrr_config.h"] {
                let file_path = self.output_dir.join(file_name);
                if file_path.exists() {
                    return Err(format!(
                        "Output file '{}' already exists. Use --overwrite to overwrite.",
                        file_path.display()
                    ));
                }
            }
        }

        if let Some(hw_rev) = self.hw_rev {
            match hw_rev {
                HwRev::A | HwRev::B | HwRev::C => {
                    if self.stm_variant != StmVariant::F103RB
                        && self.stm_variant != StmVariant::F103R8
                    {
                        return Err(
                            "Hardware revision A, B, and C only support STM32F103".to_string()
                        );
                    }
                }
                HwRev::D | HwRev::E | HwRev::F => {
                    if self.stm_variant.family() != StmFamily::F4 {
                        return Err("Hardware revision D, E, and F only supports STM32F4 family"
                            .to_string());
                    }
                }
            }
        } else {
            match self.stm_variant.family() {
                StmFamily::F1 => {
                    self.hw_rev = Some(HwRev::A);
                }
                StmFamily::F4 => {
                    self.hw_rev = Some(HwRev::D);
                }
            }
        }

        // Validate and set frequency
        match self.stm_variant.processor() {
            StmProcessor::F103 => {
                if !self.stm_variant.is_frequency_valid(self.freq, self.overclock) {
                    return Err(format!(
                        "Frequency {}MHz is not valid for STM32F103. Valid range 8-64MHz",
                        self.freq
                    ));
                }
            }
            _ => {
                if !self
                    .stm_variant
                    .is_frequency_valid(self.freq, self.overclock)
                {
                    return Err(format!(
                        "Frequency {}MHz is not valid for variant {}. Valid range: 16-{}MHz",
                        self.freq,
                        self.stm_variant.makefile_var(),
                        self.stm_variant
                            .processor()
                            .max_sysclk_mhz()
                    ));
                }

            }
        }

        Ok(())
    }
}
