// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use crate::preprocessor::{RomImage, RomSet};
use sdrr_common::HwConfig;
use sdrr_common::{CsLogic, RomType, ServeAlg, StmVariant};
use std::collections::BTreeMap;
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
    pub main_loop_one_shot: bool,
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
    pub bank: Option<usize>,
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
            Err(()) => Err(format!(
                "ROM type {} requires {} CS line(s)",
                rom_type.name(),
                rom_type.cs_lines_count()
            )),
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
        #[allow(clippy::match_single_binding)]
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
        let mut sets: Vec<usize> = self.roms.iter().filter_map(|rom| rom.set).collect();

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
                    return Err(format!(
                        "ROM sets must be numbered sequentially starting from 0. Missing set {}",
                        i
                    ));
                }
            }

            // Enhanced set validation for banking and multi-ROM modes
            for &set_id in &sets {
                let roms_in_set: Vec<_> = self
                    .roms
                    .iter()
                    .filter(|rom| rom.set == Some(set_id))
                    .collect();

                // Check if this set uses banking
                let banked_roms: Vec<_> = roms_in_set
                    .iter()
                    .filter(|rom| rom.bank.is_some())
                    .collect();

                let is_banked_set = !banked_roms.is_empty();

                if is_banked_set {
                    // Banking mode validation

                    // Check hardware variant supports banked sets
                    if !self.hw.supports_banked_roms() {
                        return Err(
                            "Bank switched sets of ROMs are only supported on hardware revision F onwards".to_string(),
                        );
                    }

                    // Check STM variant supports banked sets
                    if !self.stm_variant.supports_banked_roms() {
                        return Err(format!(
                            "Set {}: banked ROMs are not supported on STM32 variant {} due to lack of RAM and/or flash",
                            set_id,
                            self.stm_variant.makefile_var()
                        ));
                    }

                    // All ROMs in set must have bank specified
                    if banked_roms.len() != roms_in_set.len() {
                        return Err(format!(
                            "Set {}: when using banks, all ROMs in the set must specify a bank number",
                            set_id
                        ));
                    }

                    // Max 4 ROMs for banked sets
                    if roms_in_set.len() > 4 {
                        return Err(format!(
                            "Set {}: banked sets can contain maximum 4 ROMs, found {}",
                            set_id,
                            roms_in_set.len()
                        ));
                    }

                    // Banks must be sequential from 0
                    let mut banks: Vec<usize> =
                        roms_in_set.iter().map(|rom| rom.bank.unwrap()).collect();
                    banks.sort();
                    banks.dedup();

                    for (i, &bank_num) in banks.iter().enumerate() {
                        if bank_num != i {
                            return Err(format!(
                                "Set {}: bank numbers must be sequential starting from 0. Missing bank {}",
                                set_id, i
                            ));
                        }
                    }

                    // All ROMs must have same type
                    let first_rom_type = &roms_in_set[0].rom_type;
                    for rom in &roms_in_set[1..] {
                        if rom.rom_type != *first_rom_type {
                            return Err(format!(
                                "Set {}: all ROMs in a banked set must have the same type. Found {} and {}",
                                set_id,
                                first_rom_type.name(),
                                rom.rom_type.name()
                            ));
                        }
                    }

                    // All ROMs must have same CS configuration
                    let first_cs_config = &roms_in_set[0].cs_config;
                    for rom in &roms_in_set[1..] {
                        if rom.cs_config.cs1 != first_cs_config.cs1
                            || rom.cs_config.cs2 != first_cs_config.cs2
                            || rom.cs_config.cs3 != first_cs_config.cs3
                        {
                            return Err(format!(
                                "Set {}: all ROMs in a banked set must have the same CS configuration",
                                set_id
                            ));
                        }
                    }
                } else {
                    // Multi-ROM mode validation

                    // Check hardware variant supports multi-rom sets
                    if !self.hw.supports_multi_rom_sets() {
                        return Err(
                            "Multi-ROM sets of ROMs are only supported on hardware revision F onwards".to_string(),
                        );
                    }

                    // Check this STM variant supports multi-ROM sets
                    if roms_in_set.len() > 1 {
                        #[allow(clippy::collapsible_if)]
                        if !self.stm_variant.supports_multi_rom_sets() {
                            return Err(format!(
                                "Set {}: multi-set ROMs are not supported on STM32 variant {} due to lack of RAM and/or flash",
                                set_id,
                                self.stm_variant.makefile_var()
                            ));
                        }
                    }

                    // Ensure no ROMs have bank specified
                    for rom in &roms_in_set {
                        if rom.bank.is_some() {
                            return Err(format!(
                                "Set {}: mixed banking modes not allowed - either all or no ROMs in set must specify bank",
                                set_id
                            ));
                        }
                    }

                    // Max 3 ROMs for multi-ROM sets
                    if roms_in_set.len() > 3 {
                        return Err(format!(
                            "Set {}: multi-ROM sets can contain maximum 3 ROMs, found {}",
                            set_id,
                            roms_in_set.len()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn create_rom_sets(&self, rom_images: &[RomImage]) -> Result<Vec<RomSet>, String> {
        let sets: Vec<usize> = self.roms.iter().filter_map(|rom| rom.set).collect();

        if sets.is_empty() {
            let rom_sets: Vec<RomSet> = self
                .roms
                .iter()
                .zip(rom_images.iter())
                .enumerate()
                .map(|(ii, (rom_config, rom_image))| RomSet {
                    id: ii,
                    roms: vec![RomInSet {
                        config: rom_config.clone(),
                        image: rom_image.clone(),
                        original_index: ii,
                    }],
                    is_banked: false,
                })
                .collect();
            return Ok(rom_sets);
        }

        let mut unique_sets: Vec<usize> = sets.clone();
        unique_sets.sort();
        unique_sets.dedup();

        let mut rom_sets_map = BTreeMap::new();

        for &set_id in &unique_sets {
            let roms_in_set: Vec<_> = self
                .roms
                .iter()
                .zip(rom_images.iter())
                .enumerate()
                .filter(|(_, (rom_config, _))| rom_config.set == Some(set_id))
                .collect();

            let is_banked = roms_in_set
                .iter()
                .any(|(_, (rom_config, _))| rom_config.bank.is_some());

            let mut rom_set_entries = Vec::new();

            if is_banked {
                let mut banked_roms: Vec<_> = roms_in_set.into_iter().collect();
                banked_roms.sort_by_key(|(_, (rom_config, _))| rom_config.bank.unwrap());

                for (original_index, (rom_config, rom_image)) in banked_roms {
                    rom_set_entries.push(RomInSet {
                        config: rom_config.clone(),
                        image: rom_image.clone(),
                        original_index,
                    });
                }
            } else {
                for (original_index, (rom_config, rom_image)) in roms_in_set {
                    rom_set_entries.push(RomInSet {
                        config: rom_config.clone(),
                        image: rom_image.clone(),
                        original_index,
                    });
                }
            }

            rom_sets_map.insert(
                set_id,
                RomSet {
                    id: set_id,
                    roms: rom_set_entries,
                    is_banked,
                },
            );
        }

        let rom_sets: Vec<RomSet> = unique_sets
            .into_iter()
            .map(|set_id| rom_sets_map.remove(&set_id).unwrap())
            .collect();

        Ok(rom_sets)
    }
}
