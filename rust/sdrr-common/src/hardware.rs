#![allow(dead_code)]
use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Deserializer};
/// Handles loading hardware configuration files and creating objects
/// for use by sddr-gen/sdrr-info.
// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::sdrr_types::{RomType, McuFamily};

/// Top level directory searched for hardware configuration files.
pub const HW_CONFIG_DIRS: [&str; 2] = ["sdrr-hw-config", "../sdrr-hw-config"];

/// Subdirectories within the hardware configuration directory also searched
/// for hardware configuration files.
pub const HW_CONFIG_SUB_DIRS: [&str; 2] = ["user", "third-party"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Port {
    None,
    Zero, // RP2350
    A,
    B,
    C,
    D,
}

impl Port {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "0" => Some(Port::Zero),
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
            Port::Zero => write!(f, "PORT_0"),
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
pub struct McuPorts {
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
pub struct McuPins {
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
    pub x_jumper_pull: u8,
    pub sel: Vec<u8>,
    pub sel_jumper_pull: u8,
    pub status: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mcu {
    #[serde(deserialize_with = "deserialize_mcu_family")]
    pub family: McuFamily,
    pub ports: McuPorts,
    pub pins: McuPins,
}

fn deserialize_mcu_family<'de, D>(deserializer: D) -> Result<McuFamily, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    McuFamily::try_from_str(&s)
        .ok_or_else(|| serde::de::Error::custom(format!("Invalid STM family: {}", s)))
}

fn deserialize_rom_map<'de, D>(deserializer: D) -> Result<HashMap<RomType, u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map: HashMap<String, u8> = HashMap::deserialize(deserializer)?;
    let mut rom_map = HashMap::new();

    for (key, value) in string_map {
        match RomType::try_from_str(&key) {
            Some(rom_type) => {
                rom_map.insert(rom_type, value);
            }
            None => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid ROM type: {}",
                    key
                )));
            }
        }
    }

    Ok(rom_map)
}

/// Represents the hardware configuration for a particular SDRR hardware
/// config - see /sdrr-hw-config/README.md for details.
#[derive(Debug, Clone, Deserialize)]
pub struct HwConfig {
    #[serde(skip)]
    pub name: String,
    pub description: String,
    pub rom: Rom,
    pub mcu: Mcu,
    #[serde(skip)]
    phys_pin_to_addr_map: Vec<Option<usize>>,
    #[serde(skip)]
    phys_pin_to_data_map: [usize; 8],
}

impl HwConfig {
    pub fn new(json: &str, name: &str) -> Result<Self> {
        let mut config: HwConfig = serde_json::from_str(json)?;
        config.name = normalize_name(name);
        validate_config(&config.name, &config)?;

        // Create pin maps for quick access
        let num_phys_addr_pins = if config.rom.pins.quantity == 24 {
            config.mcu.family.max_valid_addr_pin()+1 // 24-pin ROMs have maximum 13 address lines + 1 CS
        } else if config.rom.pins.quantity == 28 {
            16 // 28 pin ROMs have 14 address lines + 2 CS
        } else {
            bail!(
                "Unsupported ROM type {}, expected 24 or 28-pin ROM",
                config.rom.pins.quantity
            );
        };

        // Create address pin map.
        // config.mcu.pins.addr is indexed by address line (Ax).  We need the
        // index phys_pin_to_addr_map to be indexed by physical pin (PCy).
        // Any pin that's unused (values 16-255) are set to None.
        config.phys_pin_to_addr_map = Vec::new();
        config.phys_pin_to_addr_map.resize_with(num_phys_addr_pins as usize, || None);
        for (addr_line, &phys_pin) in config.mcu.pins.addr.iter().enumerate() {
            if phys_pin <= config.mcu.family.max_valid_addr_pin() {
                config.phys_pin_to_addr_map[phys_pin as usize] = Some(addr_line);
            }
        }

        // Do the same for data lines
        config.phys_pin_to_data_map = [0; 8];
        for (data_line, &phys_pin) in config.mcu.pins.data.iter().enumerate() {
            if phys_pin <= config.mcu.family.max_valid_data_pin() {
                // We modulo 8 the physical pin as on eSTM32F4 we are limited
                // to pins 0-1 for the data lines, and on RP2350, we pretend
                // we're on pins 0-7, as we write the 8-bit value to the GPIO
                // output register, causing it to replicate across all 4 bytes,
                // meaning it gets applied to the correct pins anyway.
                config.phys_pin_to_data_map[phys_pin as usize % 8] = data_line;
            } else {
                bail!("Missing data pin {} in config {}", phys_pin, config.name);
            }
        }

        Ok(config)
    }

    pub fn get_phys_pin_to_addr_map(&self, num_addr_lines: usize) -> Vec<Option<usize>> {
        let mut map = self.phys_pin_to_addr_map.clone();
        for pin in &mut map {
            if let Some(addr) = pin {
                if *addr >= num_addr_lines {
                    *pin = None;
                }
            }
        }
        map
    }

    pub fn get_phys_pin_to_data_map(&self) -> [usize; 8] {
        self.phys_pin_to_data_map
    }

    pub fn port_data(&self) -> Port {
        self.mcu.ports.data_port
    }

    pub fn port_addr(&self) -> Port {
        self.mcu.ports.addr_port
    }

    pub fn port_cs(&self) -> Port {
        self.mcu.ports.cs_port
    }

    pub fn port_sel(&self) -> Port {
        self.mcu.ports.sel_port
    }

    pub fn port_status(&self) -> Port {
        self.mcu.ports.status_port
    }

    pub fn pin_status(&self) -> u8 {
        self.mcu.pins.status
    }

    pub fn pin_cs1(&self, rom_type: &RomType) -> u8 {
        self.mcu.pins.cs1.get(rom_type).copied().unwrap_or(255)
    }

    pub fn pin_cs2(&self, rom_type: &RomType) -> u8 {
        self.mcu.pins.cs2.get(rom_type).copied().unwrap_or(255)
    }

    pub fn pin_cs3(&self, rom_type: &RomType) -> u8 {
        self.mcu.pins.cs3.get(rom_type).copied().unwrap_or(255)
    }

    pub fn pin_x1(&self) -> u8 {
        self.mcu.pins.x1.unwrap_or(255)
    }

    pub fn pin_x2(&self) -> u8 {
        self.mcu.pins.x2.unwrap_or(255)
    }

    pub fn pin_ce(&self, rom_type: &RomType) -> u8 {
        self.mcu.pins.ce.get(rom_type).copied().unwrap_or(255)
    }

    pub fn pin_oe(&self, rom_type: &RomType) -> u8 {
        self.mcu.pins.oe.get(rom_type).copied().unwrap_or(255)
    }

    pub fn pin_sel(&self, sel: usize) -> u8 {
        self.mcu.pins.sel.get(sel).copied().unwrap_or(255)
    }

    pub fn sel_jumper_pull(&self) -> u8 {
        self.mcu.pins.sel_jumper_pull
    }

    pub fn x_jumper_pull(&self) -> u8 {
        self.mcu.pins.x_jumper_pull
    }

    pub fn cs_pin_for_rom_in_set(&self, rom_type: &RomType, set_index: usize) -> u8 {
        match set_index {
            0 => self.pin_cs1(rom_type),
            1 => self.pin_x1(),
            2 => self.pin_x2(),
            _ => 255, // No more CS pins available
        }
    }

    pub fn supports_banked_roms(&self) -> bool {
        self.supports_multi_rom_sets()
    }

    pub fn supports_multi_rom_sets(&self) -> bool {
        // Requires both pins X1 and X2
        if let Some(x1) = self.mcu.pins.x1 {
            if let Some(x2) = self.mcu.pins.x2 {
                if x1 < 255 && x2 < 255 {
                    assert!(x1 <= 15 && x2 <= 15, "X1 and X2 pins must be less than 15");
                    return true;
                }
            }
        }
        false
    }
}

fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace("_", "-")
}

fn validate_pin_number(mcu: &Mcu, pin: u8, pin_name: &str, config_name: &str) -> Result<()> {

    if !mcu.family.valid_pin_num(pin) && pin != 255 {
        bail!(
            "{}: invalid pin number {} for {}, must be valid or 255 if pin not exposed",
            config_name,
            pin,
            pin_name,
        );
    }
    Ok(())
}

fn validate_rom_types(
    mcu: &Mcu,
    rom_map: &HashMap<RomType, u8>,
    pin_type: &str,
    config_name: &str,
) -> Result<()> {
    for (rom_type, &pin) in rom_map {
        validate_pin_number(mcu, pin, &format!("{}[{:?}]", pin_type, rom_type), config_name)?;
    }
    Ok(())
}

fn validate_pin_array(mcu: &Mcu, pins: &[u8], pin_type: &str, config_name: &str, max_pins: u8) -> Result<()> {
    let mut seen = HashSet::new();
    let mut num_pins = 0;
    for &pin in pins {
        validate_pin_number(mcu, pin, pin_type, config_name)?;
        if !seen.insert(pin) {
            bail!(
                "{}: duplicate pin {} in {} array",
                config_name,
                pin,
                pin_type
            );
        }
        num_pins += 1;
    }
    if num_pins > max_pins as usize {
        bail!(
            "{}: too many pins in {} array, maximum is {}",
            config_name,
            pin_type,
            max_pins
        );
    }
    Ok(())
}
/// min_valid - minimum number of valid pins expected in the array.
/// valid_value - maximum valid pin value
fn validate_pin_values(
    pins: &[u8],
    pin_type: &str,
    config_name: &str,
    min_valid: usize,
    valid_value: u8,
) -> Result<()> {
    for (ii, &pin) in pins.iter().enumerate() {
        if ii >= min_valid {
            break;
        }
        if pin > valid_value {
            bail!(
                "{}: invalid pin value {} in {} array, must be 0-{}",
                config_name,
                pin,
                pin_type,
                valid_value
            );
        }
    }
    Ok(())
}

fn validate_config(name: &str, config: &HwConfig) -> Result<()> {
    // Check data pins are exactly 8
    if config.mcu.pins.data.len() != 8 {
        bail!(
            "{}: data pins must be exactly 8, found {}",
            name,
            config.mcu.pins.data.len()
        );
    }

    // Validate pins consistent within pin arrays
    validate_pin_array(&config.mcu, &config.mcu.pins.data, "data", name, 8)?;
    validate_pin_array(&config.mcu, &config.mcu.pins.addr, "addr", name, 16)?;
    validate_pin_array(&config.mcu, &config.mcu.pins.sel, "sel", name, 7)?;

    // Validate values in pin arrays are within valid ranges, with minimum
    // numbers
    validate_pin_values(&config.mcu.pins.data, "data", name, 8, config.mcu.family.max_valid_data_pin())?;
    match config.rom.pins.quantity {
        24 => {
            // For 24-pin ROMs, we expect address pins A0-12 to be <= 13
            // Because 14/15 used for X1/X2 and require larger RAM image
            validate_pin_values(&config.mcu.pins.addr, "addr", name, 13, config.mcu.family.max_valid_addr_pin())?
        }
        28 => {
            // For 28-pin ROMs, need 14 address lines, and 14/15 can be
            // used for an address line - CE/OE can be anywhere in the port
            validate_pin_values(&config.mcu.pins.addr, "addr", name, 14, 15)?
        }
        _ => bail!(
            "{}: unsupported ROM type {}, expected 24 or 28-pin ROM",
            name,
            config.rom.pins.quantity
        ),
    }

    // Validate ROM type mappings
    validate_rom_types(&config.mcu, &config.mcu.pins.cs1, "cs1", name)?;
    validate_rom_types(&config.mcu, &config.mcu.pins.cs2, "cs2", name)?;
    validate_rom_types(&config.mcu, &config.mcu.pins.cs3, "cs3", name)?;
    validate_rom_types(&config.mcu, &config.mcu.pins.ce, "ce", name)?;
    validate_rom_types(&config.mcu, &config.mcu.pins.oe, "oe", name)?;

    // Validate ports
    if config.mcu.ports.data_port != config.mcu.family.allowed_data_port() {
        bail!(
            "{}: data port must be {:?}, found {:?}",
            name,
            config.mcu.family.allowed_data_port(),
            config.mcu.ports.data_port
        );
    }
    if config.mcu.ports.addr_port != config.mcu.family.allowed_addr_port() {
        bail!(
            "{}: address port must be {:?}, found {:?}",
            name,
            config.mcu.family.allowed_addr_port(),
            config.mcu.ports.addr_port
        );
    }
    if config.mcu.ports.cs_port != config.mcu.family.allowed_cs_port() {
        bail!(
            "{}: CS port must be {:?}, found {:?}",
            name,
            config.mcu.family.allowed_cs_port(),
            config.mcu.ports.cs_port
        );
    }
    if config.mcu.ports.sel_port != config.mcu.family.allowed_sel_port() {
        bail!(
            "{}: SEL port must be {:?}, found {:?}",
            name,
            config.mcu.family.allowed_sel_port(),
            config.mcu.ports.sel_port
        );
    }

    // Validate optional pins
    if let Some(pin) = config.mcu.pins.x1 {
        validate_pin_number(&config.mcu, pin, "x1", name)?;
    }
    if let Some(pin) = config.mcu.pins.x2 {
        validate_pin_number(&config.mcu, pin, "x2", name)?;
    }

    // Group pins by port for conflict checking
    let mut port_pins: HashMap<Port, Vec<(&str, u8)>> = HashMap::new();

    // Add data pins
    for &pin in &config.mcu.pins.data {
        port_pins
            .entry(config.mcu.ports.data_port)
            .or_default()
            .push(("data", pin));
    }

    // Add address pins
    for &pin in &config.mcu.pins.addr {
        port_pins
            .entry(config.mcu.ports.addr_port)
            .or_default()
            .push(("addr", pin));
    }

    // Add sel pins
    for &pin in &config.mcu.pins.sel {
        port_pins
            .entry(config.mcu.ports.sel_port)
            .or_default()
            .push(("sel", pin));
    }

    // Add CS pins
    for &pin in config.mcu.pins.cs1.values() {
        port_pins
            .entry(config.mcu.ports.cs_port)
            .or_default()
            .push(("cs1", pin));
    }
    for &pin in config.mcu.pins.cs2.values() {
        port_pins
            .entry(config.mcu.ports.cs_port)
            .or_default()
            .push(("cs2", pin));
    }
    for &pin in config.mcu.pins.cs3.values() {
        port_pins
            .entry(config.mcu.ports.cs_port)
            .or_default()
            .push(("cs3", pin));
    }

    // Add optional pins
    if let Some(pin) = config.mcu.pins.x1 {
        port_pins
            .entry(config.mcu.ports.cs_port) // Assuming x1/x2 are on cs_port
            .or_default()
            .push(("x1", pin));
    }
    if let Some(pin) = config.mcu.pins.x2 {
        port_pins
            .entry(config.mcu.ports.cs_port)
            .or_default()
            .push(("x2", pin));
    }

    for &pin in config.mcu.pins.ce.values() {
        port_pins
            .entry(config.mcu.ports.cs_port)
            .or_default()
            .push(("ce", pin));
    }
    for &pin in config.mcu.pins.oe.values() {
        port_pins
            .entry(config.mcu.ports.cs_port)
            .or_default()
            .push(("oe", pin));
    }
    let pin = config.mcu.pins.status;
    port_pins
        .entry(config.mcu.ports.status_port)
        .or_default()
        .push(("status", pin));
    if config.mcu.pins.sel_jumper_pull > 1 {
        bail!("Invalid sel_jumper_pull value - set to 0 for jumper pulling sel pins down to GND, 1 for jumper pulling sel pins up.")
    }

    // Validate X1/X2 pins are fixed at 14/15 if provided
    if let Some(x1_pin) = config.mcu.pins.x1 {
        let valid_pins = config.mcu.family.valid_x1_pins();
        if !valid_pins.contains(&x1_pin) {
            bail!("{}: X1 pin must be within {:?}, found {}", name, valid_pins, x1_pin);
        }
    }
    if let Some(x2_pin) = config.mcu.pins.x2 {
        let valid_pins = config.mcu.family.valid_x2_pins();
        if !valid_pins.contains(&x2_pin) {
            bail!("{}: X2 pin must be within {:?}, found {}", name, valid_pins, x2_pin);
        }
    }

    // Both X1 and X2 must be provided together for multi-ROM support
    if config.mcu.pins.x1.is_some() != config.mcu.pins.x2.is_some() {
        bail!(
            "{}: X1 and X2 pins must both be provided or both omitted",
            name
        );
    }

    // Check for conflicts within each port
    for (port, pins) in port_pins {
        let mut used_pins: HashMap<u8, Vec<&str>> = HashMap::new();

        for (pin_type, pin_num) in pins {
            used_pins.entry(pin_num).or_default().push(pin_type);
        }

        for (pin_num, pin_types) in used_pins {
            if pin_types.len() > 1 {
                // Check if this is an allowed overlap
                let cs_types: HashSet<&str> =
                    ["cs1", "cs2", "cs3", "ce", "oe"].into_iter().collect();
                let has_cs = pin_types.iter().any(|t| cs_types.contains(t));
                let all_cs_or_addr = pin_types
                    .iter()
                    .all(|t| cs_types.contains(t) || *t == "addr");

                if !(has_cs && all_cs_or_addr) {
                    bail!(
                        "{}: pin {} on port {:?} used by multiple incompatible functions: {:?}",
                        name,
                        pin_num,
                        port,
                        pin_types
                    );
                }
            }
        }
    }

    Ok(())
}

fn get_config_dirs() -> Result<Vec<PathBuf>> {
    // Find first existing root directory
    let root = HW_CONFIG_DIRS
        .iter()
        .map(Path::new)
        .find(|path| path.exists())
        .ok_or_else(|| {
            anyhow!(
                "No hardware configuration directories found. Searched: {:?}",
                HW_CONFIG_DIRS
            )
        })?;

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
                let filename = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

                let normalized = normalize_name(filename);
                if normalized != filename {
                    bail!(
                        "Invalid hardware revision name '{}', must be lower-case with dashes, not underscores",
                        path.display()
                    );
                }

                // Check for duplicates
                if let Some(first_path) = seen_names.get(&normalized) {
                    bail!(
                        "Duplicate hardware revision '{}' found in {} and {}",
                        filename,
                        first_path.display(),
                        path.display()
                    );
                }
                seen_names.insert(normalized.clone(), path.clone());

                // Parse JSON to get description
                let content = fs::read_to_string(&path)?;
                let config = HwConfig::new(&content, &normalized).with_context(|| {
                    format!("Failed to parse hardware config: {}", path.display())
                })?;

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
                let config = HwConfig::new(&content, &normalized).with_context(|| {
                    format!(
                        "Failed to parse hardware config '{}'",
                        config_path.display()
                    )
                })?;

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
