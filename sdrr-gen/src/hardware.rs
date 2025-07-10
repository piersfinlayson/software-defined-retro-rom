#![allow(dead_code)]
/// Handles loading hardware configuration files and creating objects
/// for use by sddr-gen.

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Deserializer};
use anyhow::{anyhow, Result, bail, Context};

use crate::rom_types::{RomType, StmFamily};

// Maximum pin number on an STM32 port
const MAX_STM_PIN_NUM: u8 = 15;

/// Top level directory searched for hardware configuration files.
pub const HW_CONFIG_DIRS: [&str; 2] = ["sdrr-hw-config", "../sdrr-hw-config"];

/// Subdirectories within the hardware configuration directory also searched
/// for hardware configuration files.
pub const HW_CONFIG_SUB_DIRS: [&str; 2] = ["user", "third-party"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Port {
    None,
    A,
    B,
    C,
    D,
}

impl Port {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "A" => Some(Port::A),
            "B" => Some(Port::B),
            "C" => Some(Port::C),
            "D" => Some(Port::D),
            "NONE" => Some(Port::None),
            _ => None,
        }
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Port::None => write!(f, "PORT_NONE"),
            Port::A => write!(f, "PORT_A"),
            Port::B => write!(f, "PORT_B"),
            Port::C => write!(f, "PORT_C"),
            Port::D => write!(f, "PORT_D"),
        }
    }
}

impl<'de> Deserialize<'de> for Port {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Port::from_str(&s).ok_or_else(|| {
            serde::de::Error::custom(format!("Invalid port: {}, must be None, A, B, C, or D", s))
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct StmPorts {
    pub data_port: Port,
    pub addr_port: Port,
    pub cs_port: Port,
    pub sel_port: Port,
    pub status_port: Port,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RomPins {
    pub quantity: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rom {
    pub pins: RomPins,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StmPins {
    pub data: Vec<u8>,
    pub addr: Vec<u8>,
    #[serde(default, deserialize_with = "deserialize_rom_map")]
    pub cs1: HashMap<RomType, u8>,
    #[serde(default, deserialize_with = "deserialize_rom_map")]
    pub cs2: HashMap<RomType, u8>,
    #[serde(default, deserialize_with = "deserialize_rom_map")]
    pub cs3: HashMap<RomType, u8>,
    pub x1: Option<u8>,
    pub x2: Option<u8>,
    #[serde(default, deserialize_with = "deserialize_rom_map")]
    pub ce: HashMap<RomType, u8>,
    #[serde(default, deserialize_with = "deserialize_rom_map")]
    pub oe: HashMap<RomType, u8>,
    pub sel: Vec<u8>,
    pub status: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Stm {
    #[serde(deserialize_with = "deserialize_stm_family")]
    pub family: StmFamily,
    pub ports: StmPorts,
    pub pins: StmPins,
}

fn deserialize_stm_family<'de, D>(deserializer: D) -> Result<StmFamily, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    StmFamily::from_str(&s).ok_or_else(|| {
        serde::de::Error::custom(format!("Invalid STM family: {}", s))
    })
}

fn deserialize_rom_map<'de, D>(deserializer: D) -> Result<HashMap<RomType, u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map: HashMap<String, u8> = HashMap::deserialize(deserializer)?;
    let mut rom_map = HashMap::new();
    
    for (key, value) in string_map {
        match RomType::from_str(&key) {
            Some(rom_type) => {
                rom_map.insert(rom_type, value);
            },
            None => {
                return Err(serde::de::Error::custom(format!("Invalid ROM type: {}", key)));
            }
        }
    }
    
    Ok(rom_map)
}

#[derive(Debug, Clone, Deserialize)]
pub struct HwConfig {
    #[serde(skip)]
    pub name: String,
    pub description: String,
    pub rom: Rom,
    pub stm: Stm,
}

impl HwConfig {
    pub fn port_data(&self) -> Port {
        self.stm.ports.data_port
    }

    pub fn port_addr(&self) -> Port {
        self.stm.ports.addr_port
    }

    pub fn port_cs(&self) -> Port {
        self.stm.ports.cs_port
    }

    pub fn port_sel(&self) -> Port {
        self.stm.ports.sel_port
    }

    pub fn port_status(&self) -> Port {
        self.stm.ports.status_port
    }

    pub fn pin_status(&self) -> u8 {
        self.stm.pins.status
    }

    pub fn pin_cs1(&self, rom_type: &RomType) -> u8 {
        self.stm.pins.cs1.get(&rom_type).copied().unwrap_or(255)
    }

    pub fn pin_cs2(&self, rom_type: &RomType) -> u8 {
        self.stm.pins.cs2.get(&rom_type).copied().unwrap_or(255)
    }

    pub fn pin_cs3(&self, rom_type: &RomType) -> u8 {
        self.stm.pins.cs3.get(&rom_type).copied().unwrap_or(255)
    }

    pub fn pin_x1(&self) -> u8 {
        self.stm.pins.x1.unwrap_or(255)
    }

    pub fn pin_x2(&self) -> u8 {
        self.stm.pins.x2.unwrap_or(255)
    }

    pub fn pin_ce(&self, rom_type: &RomType) -> u8 {
        self.stm.pins.ce.get(&rom_type).copied().unwrap_or(255)
    }

    pub fn pin_oe(&self, rom_type: &RomType) -> u8 {
        self.stm.pins.oe.get(&rom_type).copied().unwrap_or(255)
    }

    pub fn pin_sel(&self, sel: usize) -> u8 {
        self.stm.pins.sel.get(sel).copied().unwrap_or(255)
    }

    pub fn cs_pin_for_rom_in_set(&self, rom_type: &RomType, set_index: usize) -> u8 {
        match set_index {
            0 => self.pin_cs1(rom_type),
            1 => self.pin_x1(),
            2 => self.pin_x2(),
            _ => 255, // No more CS pins available
        }
    }

    pub fn supports_multi_rom_sets(&self) -> bool {
        // Requires both pins X1 and X2
        if let Some(x1) = self.stm.pins.x1 {
            if let Some(x2) = self.stm.pins.x2 {
                if x1 < 255 && x2 < 255 {
                    assert!(x1 < 15 && x2 < 15, "X1 and X2 pins must be less than 15");
                    return true
                }
            }
        }
        false
    }
}

fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace("_", "-")
}

fn validate_pin_number(pin: u8, pin_name: &str, config_name: &str) -> Result<()> {
    if pin > MAX_STM_PIN_NUM && pin != 255 {
        bail!("{}: invalid pin number {} for {}, must be 0-{} or 255",
              config_name, pin, pin_name, MAX_STM_PIN_NUM);
    }
    Ok(())
}

fn validate_rom_types(rom_map: &HashMap<RomType, u8>, pin_type: &str, config_name: &str) -> Result<()> {
    for (rom_type, &pin) in rom_map {
        validate_pin_number(pin, &format!("{}[{:?}]", pin_type, rom_type), config_name)?;
    }
    Ok(())
}

fn validate_pin_array(pins: &[u8], pin_type: &str, config_name: &str, max_pins: u8) -> Result<()> {
    let mut seen = HashSet::new();
    let mut num_pins = 0;
    for &pin in pins {
        validate_pin_number(pin, pin_type, config_name)?;
        if !seen.insert(pin) {
            bail!("{}: duplicate pin {} in {} array", config_name, pin, pin_type);
        }
        num_pins += 1;
    }
    if num_pins > max_pins as usize {
        bail!("{}: too many pins in {} array, maximum is {}", config_name, pin_type, max_pins);
    }
    Ok(())
}

fn validate_pin_values(pins: &[u8], pin_type: &str, config_name: &str, min_valid: usize, valid_value: u8) -> Result<()> {
    let mut ii = 0;
    for &pin in pins {
        if ii >= min_valid {
            break;
        }
        if pin > valid_value {
            bail!("{}: invalid pin value {} in {} array, must be 0-{}",
                  config_name, pin, pin_type, valid_value);
        }
        ii += 1
    }
    Ok(())
}

fn validate_config(name: &str, config: &HwConfig) -> Result<()> {
    // Only support 24 pins ROMS, currently
    if config.rom.pins.quantity != 24 {
        bail!("{}: ROM pins quantity must currently be 24, found {}", name, config.rom.pins.quantity);
    }
    
    // Check data pins are exactly 8
    if config.stm.pins.data.len() != 8 {
        bail!("{}: data pins must be exactly 8, found {}", name, config.stm.pins.data.len());
    }
    
    // Validate pins consistent within pin arrays
    validate_pin_array(&config.stm.pins.data, "data", name, 8)?;
    validate_pin_array(&config.stm.pins.addr, "addr", name, 16)?;
    validate_pin_array(&config.stm.pins.sel, "sel", name, 4)?;

    // Validate values in pin arrays are within valid ranges, with minimum
    // numbers
    validate_pin_values(&config.stm.pins.data, "data", name, 8, 7)?;
    if config.rom.pins.quantity == 24 {
        // For 24-pin ROMs, we expect address pins A0-12 to be <= 13
        // Because 14/15 used for X1/X2 and require larger RAM image
        validate_pin_values(&config.stm.pins.addr, "addr", name, 13, 13)?;
    } else {
        bail!("{}: unsupported ROM type {}, expected 24-pin ROM", name, config.rom.pins.quantity);
    }
    
    // Validate ROM type mappings
    validate_rom_types(&config.stm.pins.cs1, "cs1", name)?;
    validate_rom_types(&config.stm.pins.cs2, "cs2", name)?;
    validate_rom_types(&config.stm.pins.cs3, "cs3", name)?;
    validate_rom_types(&config.stm.pins.ce, "ce", name)?;
    validate_rom_types(&config.stm.pins.oe, "oe", name)?;

    // Validate ports
    if config.stm.ports.data_port != Port::A {
        bail!("{}: data port must be A, found {:?}", name, config.stm.ports.data_port);
    }
    if config.stm.ports.addr_port != Port::C {
        bail!("{}: address port must be C, found {:?}", name, config.stm.ports.addr_port);
    }
    if config.stm.ports.cs_port != Port::C {
        bail!("{}: CS port must be C, found {:?}", name, config.stm.ports.cs_port);
    }
    if config.stm.ports.sel_port != Port::B {
        bail!("{}: SEL port must be B, found {:?}", name, config.stm.ports.sel_port);
    }
    
    // Validate optional pins
    if let Some(pin) = config.stm.pins.x1 {
        validate_pin_number(pin, "x1", name)?;
    }
    if let Some(pin) = config.stm.pins.x2 {
        validate_pin_number(pin, "x2", name)?;
    }

    // Group pins by port for conflict checking
    let mut port_pins: HashMap<Port, Vec<(&str, u8)>> = HashMap::new();
    
    // Add data pins
    for &pin in &config.stm.pins.data {
        port_pins.entry(config.stm.ports.data_port)
                 .or_default()
                 .push(("data", pin));
    }
    
    // Add address pins
    for &pin in &config.stm.pins.addr {
        port_pins.entry(config.stm.ports.addr_port)
                 .or_default()
                 .push(("addr", pin));
    }
    
    // Add sel pins
    for &pin in &config.stm.pins.sel {
        port_pins.entry(config.stm.ports.sel_port)
                 .or_default()
                 .push(("sel", pin));
    }
    
    // Add CS pins
    for &pin in config.stm.pins.cs1.values() {
        port_pins.entry(config.stm.ports.cs_port)
                 .or_default()
                 .push(("cs1", pin));
    }
    for &pin in config.stm.pins.cs2.values() {
        port_pins.entry(config.stm.ports.cs_port)
                 .or_default()
                 .push(("cs2", pin));
    }
    for &pin in config.stm.pins.cs3.values() {
        port_pins.entry(config.stm.ports.cs_port)
                 .or_default()
                 .push(("cs3", pin));
    }
    
    // Add optional pins
    if let Some(pin) = config.stm.pins.x1 {
        port_pins.entry(config.stm.ports.cs_port)  // Assuming x1/x2 are on cs_port
                 .or_default()
                 .push(("x1", pin));
    }
    if let Some(pin) = config.stm.pins.x2 {
        port_pins.entry(config.stm.ports.cs_port)
                 .or_default()
                 .push(("x2", pin));
    }
    
    for &pin in config.stm.pins.ce.values() {
        port_pins.entry(config.stm.ports.cs_port)
                 .or_default()
                 .push(("ce", pin));
    }
    for &pin in config.stm.pins.oe.values() {
        port_pins.entry(config.stm.ports.cs_port)
                 .or_default()
                 .push(("oe", pin));
    }
    let pin = config.stm.pins.status;
    port_pins.entry(config.stm.ports.status_port)
                .or_default()
                .push(("status", pin));

    // Check for conflicts within each port
    for (port, pins) in port_pins {
        let mut used_pins: HashMap<u8, Vec<&str>> = HashMap::new();
        
        for (pin_type, pin_num) in pins {
            used_pins.entry(pin_num).or_default().push(pin_type);
        }
        
        for (pin_num, pin_types) in used_pins {
            if pin_types.len() > 1 {
                // Check if this is an allowed overlap
                let cs_types: HashSet<&str> = ["cs1", "cs2", "cs3", "ce", "oe"].into_iter().collect();
                let has_cs = pin_types.iter().any(|t| cs_types.contains(t));
                let all_cs_or_addr = pin_types.iter().all(|t| cs_types.contains(t) || *t == "addr");
                
                if !(has_cs && all_cs_or_addr) {
                    bail!("{}: pin {} on port {:?} used by multiple incompatible functions: {:?}", 
                          name, pin_num, port, pin_types);
                }
            }
        }
    }
    
    Ok(())
}

fn get_config_dirs() -> Result<Vec<PathBuf>> {
    // Find first existing root directory
    let root = HW_CONFIG_DIRS.iter()
        .map(|dir| Path::new(dir))
        .find(|path| path.exists())
        .ok_or_else(|| anyhow!("No hardware configuration directories found. Searched: {:?}", HW_CONFIG_DIRS))?;

    // Build list starting with root, then existing subdirs
    let mut dirs = vec![root.to_path_buf()];
    
    for subdir in HW_CONFIG_SUB_DIRS.iter() {
        let subdir_path = root.join(subdir);
        if subdir_path.exists() {
            dirs.push(subdir_path);
        } else {
            println!("Config subdirectory not found: {}", subdir_path.display());
        }
    }

    Ok(dirs)
}

pub fn list_available_configs() -> Result<Vec<(String, String)>> {
    let config_dirs = get_config_dirs()?;
    
    let mut configs = Vec::new();
    let mut seen_names: HashMap<String, PathBuf> = HashMap::new();

    for config_dir in config_dirs {
        for entry in fs::read_dir(config_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path.file_stem()
                                .and_then(|s| s.to_str())
                                .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
                
                let normalized = normalize_name(filename);
                if normalized != filename {
                    bail!("Invalid hardware revision name '{}', must be lower-case with dashes, not underscores", path.display());
                }
                
                // Check for duplicates
                if let Some(first_path) = seen_names.get(&normalized) {
                    bail!("Duplicate hardware revision '{}' found in {} and {}", 
                        filename, first_path.display(), path.display());
                }
                seen_names.insert(normalized.clone(), path.clone());
                
                // Parse JSON to get description
                let content = fs::read_to_string(&path)?;
                let mut config: HwConfig = serde_json::from_str(&content)?;
                config.name = normalized.clone();
                            
                configs.push((filename.to_string(), config.description));
            }
        }
    }
    
    if configs.is_empty() {
        bail!("No valid hardware configurations found");
    }
    
    configs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(configs)
}

pub fn get_hw_config(name: &str) -> Result<HwConfig> {
    // We enumerate the configurations, both to parse them and check there's
    // no duplicates.  We don't actually output the list here though.
    // If there's a problem the error will propagate up.
    list_available_configs()?;

    // Now load the config we've been asked for.
    let normalized = normalize_name(name);
    let config_dirs = get_config_dirs()?;
    
    for config_dir in config_dirs {
        let config_path = config_dir.join(format!("{}.json", normalized));
        
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                let mut config: HwConfig = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON in: {}", config_path.display()))?;

                config.name = normalized.clone();
                validate_config(&normalized, &config)?;
                
                return Ok(config);
            }
            Err(_) => continue, // Try next directory
        }
    }
    
    bail!("Hardware config '{}' not found", normalized);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("23-D"), "23-d");
        assert_eq!(normalize_name("23_D"), "23-d");
        assert_eq!(normalize_name("28_A"), "28-a");
        assert_eq!(normalize_name("28-A"), "28-a");
    }
}