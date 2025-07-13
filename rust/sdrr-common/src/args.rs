// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use crate::hardware::{get_hw_config, HwConfig};
use crate::sdrr_types::{StmVariant, ServeAlg};

pub fn parse_stm_variant(s: &str) -> Result<StmVariant, String> {
    StmVariant::from_str(s)
        .ok_or_else(|| format!("Invalid STM32 variant: {}. Valid values are: f446rc, f446re, f411rc, f411re, f405rg, f401re, f401rb, f401rc", s))
}

pub fn parse_hw_rev(hw_rev: &str) -> Result<HwConfig, String> {
    // Special case d, e and f for backwards compatibility
    let hw_rev = match hw_rev {
        "d" => "24-d",
        "e" => "24-e",
        "f" => "24-f",
        _ => hw_rev,
    };

    let hw_config = get_hw_config(hw_rev)
        .map_err(|e| format!("Failed to get hardware config: {} - use --list-hw-revs for options", e))?;

    if hw_config.rom.pins.quantity != 24 {
        return Err(format!(
            "{}: ROM pins quantity must currently be 24, found {}",
            hw_rev, hw_config.rom.pins.quantity
        ));
    }
    
    Ok(hw_config)
}

pub fn parse_serve_alg(s: &str) -> Result<ServeAlg, String> {
    ServeAlg::from_str(s).ok_or_else(|| {
        format!(
            "Invalid serve algorithm: {}. Valid values are: default, a (2 CS 1 Addr), b (Addr on CS)",
            s
        )
    })
}

