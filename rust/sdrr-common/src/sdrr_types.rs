// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RomType {
    Rom2316,
    Rom2332,
    Rom2364,
    Rom23128,
}

impl RomType {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "2316" => Some(RomType::Rom2316),
            "2332" => Some(RomType::Rom2332),
            "2364" => Some(RomType::Rom2364),
            "23128" => Some(RomType::Rom23128),
            _ => None,
        }
    }

    pub fn num_addr_lines(&self) -> usize {
        match self {
            RomType::Rom2316 => 11,  // 2^11 = 2048 bytes
            RomType::Rom2332 => 12,  // 2^12 = 4096 bytes
            RomType::Rom2364 => 13,  // 2^13 = 8192 bytes
            RomType::Rom23128 => 14, // 2^14 = 16384 bytes
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            RomType::Rom2316 => 2048,   // 2KB
            RomType::Rom2332 => 4096,   // 4KB
            RomType::Rom2364 => 8192,   // 8KB
            RomType::Rom23128 => 16384, // 16KB
        }
    }

    pub fn cs_lines_count(&self) -> usize {
        match self {
            RomType::Rom2316 => 3,
            RomType::Rom2332 => 2,
            RomType::Rom2364 => 1,
            RomType::Rom23128 => 2,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RomType::Rom2316 => "2316",
            RomType::Rom2332 => "2332",
            RomType::Rom2364 => "2364",
            RomType::Rom23128 => "23128",
        }
    }

    pub fn c_enum(&self) -> &str {
        match self {
            RomType::Rom2316 => "ROM_TYPE_2316",
            RomType::Rom2332 => "ROM_TYPE_2332",
            RomType::Rom2364 => "ROM_TYPE_2364",
            RomType::Rom23128 => "ROM_TYPE_23128",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmFamily {
    F4,
}

impl StmFamily {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "f4" => Some(StmFamily::F4),
            _ => None,
        }
    }
}

impl fmt::Display for StmFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StmFamily::F4 => write!(f, "F4"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmProcessor {
    F401BC,
    F401DE,
    F405,
    F411,
    F446,
}

impl StmProcessor {
    pub fn vco_min_mhz(&self) -> u32 {
        match self {
            StmProcessor::F401BC => 192,
            StmProcessor::F401DE => 192,
            StmProcessor::F405 => 100,
            StmProcessor::F411 => 100,
            StmProcessor::F446 => 100,
        }
    }

    pub fn vco_max_mhz(&self, overclock: bool) -> u32 {
        if !overclock {
            432
        } else {
            // Theoretically allows up to 400MHz!
            800
        }
    }

    pub fn max_sysclk_mhz(&self) -> u32 {
        match self {
            StmProcessor::F401BC => 84,
            StmProcessor::F401DE => 84,
            StmProcessor::F405 => 168,
            StmProcessor::F411 => 100,
            StmProcessor::F446 => 180,
        }
    }

    /// Calculate PLL values for target frequency using HSI (16 MHz)
    /// Returns (PLLM, PLLN, PLLP, PLLQ) or None if frequency not achievable
    pub fn calculate_pll_hsi(
        &self,
        target_freq_mhz: u32,
        overclock: bool,
    ) -> Option<(u8, u16, u8, u8)> {
        // Validate target frequency is within limits
        if target_freq_mhz > self.max_sysclk_mhz() && !overclock {
            return None;
        }

        // HSI = 16 MHz, target VCO input = 2 MHz for best jitter
        const HSI_MHZ: u32 = 16;
        const PLLM: u8 = 8; // 16/8 = 2 MHz VCO input
        const VCO_IN_MHZ: u32 = HSI_MHZ / PLLM as u32;

        // Try PLLP values: 2, 4, 6, 8
        for pllp in [2u8, 4, 6, 8] {
            let vco_mhz = target_freq_mhz * pllp as u32;

            // Check VCO frequency is in valid range
            if vco_mhz >= self.vco_min_mhz() && vco_mhz <= self.vco_max_mhz(overclock) {
                let plln = vco_mhz / VCO_IN_MHZ;

                // Check PLLN is in valid range (50-432)
                if (50..=432).contains(&plln) {
                    // Calculate PLLQ for USB (48 MHz target)
                    let pllq_raw = (vco_mhz as f32 / 48.0).round() as u8;
                    let pllq = pllq_raw.clamp(2, 15);

                    return Some((PLLM, plln as u16, pllp, pllq));
                }
            }
        }

        None
    }

    /// Generate PLL #defines for target frequency
    pub fn generate_pll_defines(&self, target_freq_mhz: u32, overclock: bool) -> Option<String> {
        if let Some((m, n, p, q)) = self.calculate_pll_hsi(target_freq_mhz, overclock) {
            // Calculate intermediate values for comments
            let hsi_mhz = 16;
            let vco_input_mhz = hsi_mhz / m as u32;
            let fvco_mhz = vco_input_mhz * n as u32;
            let sysclk_mhz = fvco_mhz / p as u32;
            let usb_mhz = fvco_mhz / q as u32;

            // Convert PLL_P division factor to register encoding
            let pll_p_reg = match p {
                2 => "0b00",
                4 => "0b01",
                6 => "0b10",
                8 => "0b11",
                _ => unreachable!("Invalid PLL_P value: {}", p),
            };

            Some(format!(
                "//   HSI={}MHz\n//   VCO_input={}MHz\n//   fVCO={}MHz\n//   SYSCLK={}MHz\n//   USB={}MHz\n#define PLL_M    {}\n#define PLL_N    {}\n#define PLL_P    {}  // div {}\n#define PLL_Q    {}",
                hsi_mhz, vco_input_mhz, fvco_mhz, sysclk_mhz, usb_mhz, m, n, pll_p_reg, p, q
            ))
        } else {
            None
        }
    }

    /// Check if target frequency is achievable with HSI PLL configuration
    pub fn is_frequency_valid(&self, target_freq_mhz: u32, overclock: bool) -> bool {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => {
                // F4 family uses HSI PLL, check if target frequency is achievable
                self.calculate_pll_hsi(target_freq_mhz, overclock).is_some()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmVariant {
    F446RC, // STM32F446RC (6 or 7), 64-pins, 128KB SRAM, 256KB Flash
    F446RE, // STM32F446RE (6 or 7), 64-pins, 128KB SRAM, 512KB Flash
    F411RC, // STM32F411RC (6 or 7), 64-pins, 128KB SRAM, 256KB Flash
    F411RE, // STM32F411RE (6 or 7), 64-pins, 128KB SRAM, 512KB Flash
    F405RG, // STM32F405RE (6 or 7), 64-pins, 128KB SRAM, 1024KB Flash (+ 64KB CCM RAM)
    F401RE, // STM32F401RE (6 or 7), 64-pins, 96KB SRAM, 512KB Flash
    F401RB, // STM32F401RB (6 or 7), 64-pins, 64KB SRAM, 128KB Flash
    F401RC, // STM32F401RC (6 or 7), 64-pins, 96KB SRAM, 256KB Flash
}

impl StmVariant {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "f446rc" => Some(StmVariant::F446RC),
            "f446re" => Some(StmVariant::F446RE),
            "f411rc" => Some(StmVariant::F411RC),
            "f411re" => Some(StmVariant::F411RE),
            "f405rg" => Some(StmVariant::F405RG),
            "f401re" => Some(StmVariant::F401RE),
            "f401rb" => Some(StmVariant::F401RB),
            "f401rc" => Some(StmVariant::F401RC),
            _ => None,
        }
    }

    pub fn line_enum(&self) -> &str {
        match self {
            StmVariant::F446RC | StmVariant::F446RE => "F446",
            StmVariant::F411RC | StmVariant::F411RE => "F411",
            StmVariant::F405RG => "F405",
            StmVariant::F401RE => "F401DE",
            StmVariant::F401RB | StmVariant::F401RC => "F401BC",
        }
    }

    pub fn storage_enum(&self) -> &str {
        match self {
            StmVariant::F446RC => "STORAGE_C",
            StmVariant::F446RE => "STORAGE_E",
            StmVariant::F411RC => "STORAGE_C",
            StmVariant::F411RE => "STORAGE_E",
            StmVariant::F405RG => "STORAGE_G",
            StmVariant::F401RE => "STORAGE_E",
            StmVariant::F401RB => "STORAGE_B",
            StmVariant::F401RC => "STORAGE_C",
        }
    }

    fn flash_storage_bytes(&self) -> usize {
        self.flash_storage_kb() * 1024
    }

    pub fn flash_storage_kb(&self) -> usize {
        match self {
            StmVariant::F446RC => 256,
            StmVariant::F446RE => 512,
            StmVariant::F411RC => 256,
            StmVariant::F411RE => 512,
            StmVariant::F405RG => 1024,
            StmVariant::F401RB => 128,
            StmVariant::F401RC => 256,
            StmVariant::F401RE => 512,
        }
    }

    pub fn ram_kb(&self) -> usize {
        match self {
            StmVariant::F446RC | StmVariant::F446RE => 128,
            StmVariant::F411RC | StmVariant::F411RE => 128,
            StmVariant::F405RG => 128, // +64KB CCM RAM
            StmVariant::F401RB | StmVariant::F401RC => 64,
            StmVariant::F401RE => 96,
        }
    }

    pub fn supports_banked_roms(&self) -> bool {
        // 72 KB RAM as requires:
        // - 64KB for total of 4 16KB banked images
        // - 4KB for logging buffer
        // - 4KB for everything else
        //
        // 96KB flash as requires:
        // - 64KB for total of 1 set of 4x16KB banked images
        // - 32KB for firmware
        self.ram_kb() > 72 && self.flash_storage_kb() >= 96
    }

    pub fn supports_multi_rom_sets(&self) -> bool {
        // Same criteria as banked roms
        self.supports_banked_roms()
    }

    pub fn ccm_ram_kb(&self) -> Option<usize> {
        // F405 has 64KB of CCM RAM, others don't
        match self {
            StmVariant::F405RG => Some(64),
            _ => None,
        }
    }

    pub fn define_flash_size_bytes(&self) -> String {
        format!("#define STM_FLASH_SIZE {}", self.flash_storage_bytes())
    }

    pub fn define_flash_size_kb(&self) -> String {
        format!("#define STM_FLASH_SIZE_KB {}", self.flash_storage_kb())
    }

    pub fn define_ram_size_bytes(&self) -> String {
        format!("#define STM_RAM_SIZE {}", self.ram_kb() * 1024)
    }

    pub fn define_ram_size_kb(&self) -> String {
        format!("#define STM_RAM_SIZE_KB {}", self.ram_kb())
    }

    pub fn define_var_sub_fam(&self) -> &str {
        match self {
            StmVariant::F446RC | StmVariant::F446RE => "#define STM32F446      1",
            StmVariant::F411RC | StmVariant::F411RE => "#define STM32F411      1",
            StmVariant::F405RG => "#define STM32F405      1",
            StmVariant::F401RE => "#define STM32F401DE    1",
            StmVariant::F401RB | StmVariant::F401RC => "#define STM32F401BC    1",
        }
    }

    pub fn family(&self) -> StmFamily {
        match self {
            StmVariant::F446RC
            | StmVariant::F446RE
            | StmVariant::F411RC
            | StmVariant::F411RE
            | StmVariant::F405RG
            | StmVariant::F401RE
            | StmVariant::F401RB
            | StmVariant::F401RC => StmFamily::F4,
        }
    }

    pub fn processor(&self) -> StmProcessor {
        match self {
            StmVariant::F446RC | StmVariant::F446RE => StmProcessor::F446,
            StmVariant::F411RC | StmVariant::F411RE => StmProcessor::F411,
            StmVariant::F405RG => StmProcessor::F405,
            StmVariant::F401RE => StmProcessor::F401DE,
            StmVariant::F401RB | StmVariant::F401RC => StmProcessor::F401BC,
        }
    }

    pub fn define_var_fam(&self) -> &str {
        match self.family() {
            StmFamily::F4 => "#define STM32F4        1",
        }
    }

    pub fn define_var_str(&self) -> &str {
        match self {
            StmVariant::F446RC => "#define STM_VARIANT    \"F446RC\"",
            StmVariant::F446RE => "#define STM_VARIANT    \"F446RE\"",
            StmVariant::F411RC => "#define STM_VARIANT    \"F411RC\"",
            StmVariant::F411RE => "#define STM_VARIANT    \"F411RE\"",
            StmVariant::F405RG => "#define STM_VARIANT    \"F405RG\"",
            StmVariant::F401RE => "#define STM_VARIANT    \"F401RE\"",
            StmVariant::F401RB => "#define STM_VARIANT    \"F401RB\"",
            StmVariant::F401RC => "#define STM_VARIANT    \"F401RC\"",
        }
    }

    /// Generate PLL defines for target frequency (F4 variants only)
    pub fn generate_pll_defines(&self, target_freq_mhz: u32, overclock: bool) -> Option<String> {
        self.processor()
            .generate_pll_defines(target_freq_mhz, overclock)
    }

    /// Used to pass into sdrr Makefile as VARIANT
    pub fn makefile_var(&self) -> &str {
        match self {
            StmVariant::F446RC => "stm32f446rc",
            StmVariant::F446RE => "stm32f446re",
            StmVariant::F411RC => "stm32f411rc",
            StmVariant::F411RE => "stm32f411re",
            StmVariant::F405RG => "stm32f405rg",
            StmVariant::F401RE => "stm32f401re",
            StmVariant::F401RB => "stm32f401rb",
            StmVariant::F401RC => "stm32f401rc",
        }
    }

    /// Used to pass to probe-rs
    pub fn chip_id(&self) -> &str {
        match self {
            StmVariant::F446RC => "STM32F446RCTx",
            StmVariant::F446RE => "STM32F446RETx",
            StmVariant::F411RC => "STM32F411RCTx",
            StmVariant::F411RE => "STM32F411RETx",
            StmVariant::F405RG => "STM32F405RGTx",
            StmVariant::F401RE => "STM32F401RETx",
            StmVariant::F401RB => "STM32F401RBTx",
            StmVariant::F401RC => "STM32F401RCTx",
        }
    }

    /// Check if target frequency is valid for this variant
    pub fn is_frequency_valid(&self, target_freq_mhz: u32, overclock: bool) -> bool {
        self.processor()
            .is_frequency_valid(target_freq_mhz, overclock)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServeAlg {
    /// default
    Default,

    /// a
    TwoCsOneAddr,

    /// b
    AddrOnCs,
}

impl ServeAlg {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" => Some(ServeAlg::Default),
            "a" | "two_cs_one_addr" => Some(ServeAlg::TwoCsOneAddr),
            "b" => Some(ServeAlg::AddrOnCs),
            _ => None,
        }
    }

    pub fn c_value(&self) -> &str {
        match self {
            ServeAlg::Default => "SERVE_ADDR_ON_CS",
            ServeAlg::TwoCsOneAddr => "SERVE_TWO_CS_ONE_ADDR",
            ServeAlg::AddrOnCs => "SERVE_ADDR_ON_CS",
        }
    }

    pub fn c_value_multi_rom_set(&self) -> &str {
        "SERVE_ADDR_ON_ANY_CS"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsLogic {
    ActiveLow,
    ActiveHigh,

    /// Used for 2332/2316 ROMs, when a CS line isn't used because it's always
    /// tied active.
    Ignore,
}

impl CsLogic {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "0" => Some(CsLogic::ActiveLow),
            "1" => Some(CsLogic::ActiveHigh),
            "ignore" => Some(CsLogic::Ignore),
            _ => None,
        }
    }
}
