// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use crate::rom_types::{CsLogic, RomType, ServeAlg, StmVariant};
use crate::preprocessor::{RomImage, RomSet};
use crate::hardware::HwConfig;
use std::path::PathBuf;
use std::collections::BTreeMap;

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
    pub hw: HwConfig,
    pub freq: u32,
    pub status_led: bool,
    pub overclock: bool,
    pub bootloader: bool,
    pub preload_to_ram: bool,
    pub auto_yes: bool,
    pub serve_alg: ServeAlg,
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
    pub set: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RomInSet {
    pub config: RomConfig,
    pub image: RomImage,
    pub original_index: usize,
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
        // Check CS1 isn't ignore
        if self.cs1 == CsLogic::Ignore {
            return Err(
                "CS1 cannot be set to 'ignore' - it must be active high or low".to_string(),
            );
        }

        match match rom_type {
            RomType::Rom2364 => {
                // 2364 requires only CS1 (1 CS line)
                if self.cs2.is_some() || self.cs3.is_some() {
                    Err(())
                } else {
                    Ok(())
                }
            }
            RomType::Rom2332 => {
                // 2332 requires CS1 and CS2 (2 CS lines)
                if self.cs3.is_some() {
                    return Err(format!("ROM type {} does not support CS3", rom_type.name()));
                }
                if self.cs2.is_none() { Err(()) } else { Ok(()) }
            }
            RomType::Rom2316 => {
                // 2316 requires CS1, CS2, and CS3 (3 CS lines)
                if self.cs2.is_none() || self.cs3.is_none() {
                    Err(())
                } else {
                    Ok(())
                }
            }
            RomType::Rom23128 => {
                unreachable!("23128 not yet supported");
            }
        } {
            Ok(()) => Ok(()),
            Err(()) => {
                return Err(format!(
                    "ROM type {} requires {} CS line(s)",
                    rom_type.name(),
                    rom_type.cs_lines_count()
                ));
            }
        }
    }
}

impl Config {
    pub fn validate(&mut self) -> Result<(), String> {
        // Validate at least one ROM
        if self.roms.is_empty() {
            return Err("At least one ROM image must be provided".to_string());
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

        // Validate processor against family
        if self.stm_variant.family() != self.hw.stm.family {
            return Err(format!(
                "STM32 variant {} does not match hardware family {}",
                self.stm_variant.makefile_var(),
                self.hw.stm.family
            ));
        }

        // Validate and set frequency
        match self.stm_variant.processor() {
            _ => {
                if !self
                    .stm_variant
                    .is_frequency_valid(self.freq, self.overclock)
                {
                    return Err(format!(
                        "Frequency {}MHz is not valid for variant {}. Valid range: 16-{}MHz",
                        self.freq,
                        self.stm_variant.makefile_var(),
                        self.stm_variant.processor().max_sysclk_mhz()
                    ));
                }
            }
        }

        // Validate ROM sets (basic validation that doesn't need ROM images)
        let mut sets: Vec<usize> = self.roms.iter()
            .filter_map(|rom| rom.set)
            .collect();
        
        if !sets.is_empty() {
            // Check if all ROMs have sets specified
            let roms_with_sets = self.roms.iter().filter(|rom| rom.set.is_some()).count();
            if roms_with_sets != self.roms.len() {
                return Err("When using sets, all ROMs must specify a set number".to_string());
            }
            
            // Sort and check sequential from 0
            sets.sort();
            sets.dedup();
            
            for (i, &set_num) in sets.iter().enumerate() {
                if set_num != i {
                    return Err(format!("ROM sets must be numbered sequentially starting from 0. Missing set {}", i));
                }
            }
        }

        Ok(())
    }

    pub fn create_rom_sets(&self, rom_images: &[RomImage]) -> Result<Vec<RomSet>, String> {
        // Collect all sets
        let sets: Vec<usize> = self.roms.iter()
            .filter_map(|rom| rom.set)
            .collect();
        
        if sets.is_empty() {
            // No sets specified - backward compatibility mode: each ROM gets its own set
            let rom_sets: Vec<RomSet> = self.roms.iter().zip(rom_images.iter()).enumerate()
                .map(|(ii, (rom_config, rom_image))| {
                    RomSet {
                        id: ii,
                        roms: vec![RomInSet {
                            config: rom_config.clone(),
                            image: rom_image.clone(),
                            original_index: ii,
                        }],
                    }
                })
                .collect();
            return Ok(rom_sets);
        }

        if !self.hw.supports_multi_rom_sets() {
            return Err("Multiple ROMs per set is only supported on hardware revision F".to_string());
        }

        // Create ROM sets map
        let mut rom_sets_map = BTreeMap::new();
        
        for (original_index, (rom_config, rom_image)) in self.roms.iter().zip(rom_images.iter()).enumerate() {
            let set_id = rom_config.set.unwrap(); // We know all ROMs have sets at this point
            let roms_in_set = rom_sets_map.entry(set_id).or_insert_with(Vec::new);
            

            // CS lines: 10 (standard), 14 (X1), 15 (X2) 
            // Note: X1 and X2 pins only available on hardware revision F
            let index = roms_in_set.len();
            if index >= 3 {
                return Err(format!("Set {} has more than the maximum 3 ROMs", set_id));
            }
            
            roms_in_set.push(RomInSet {
                config: rom_config.clone(),
                image: rom_image.clone(),
                original_index,
            });
        }
        
        // Convert to final ROM sets vector
        let rom_sets: Vec<RomSet> = rom_sets_map.into_iter()
            .map(|(id, roms)| RomSet { id, roms })
            .collect();
            
        Ok(rom_sets)
    }
}
