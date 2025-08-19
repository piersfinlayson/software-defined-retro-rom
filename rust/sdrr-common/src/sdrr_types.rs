// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

use std::fmt;

use crate::hardware::Port;

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
pub enum McuFamily {
    Stm32F4,
    Rp2350,
}

impl McuFamily {
    const MAX_STM_PIN_NUM: u8 = 15;
    const MAX_STM_DATA_PIN_NUM: u8 = 7;
    const MAX_RP2350_PIN_NUM: u8 = 29;
    const MAX_RP2350_ADDR_CS_PIN_NUM: u8 = 15;  // First half-word
    const MAX_RP2350_DATA_PIN_NUM: u8 = 23;     // 3rd byte
    
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "f4" => Some(McuFamily::Stm32F4),
            "rp2350" => Some(McuFamily::Rp2350),
            _ => None,
        }
    }

    pub fn valid_pin_num(&self, pin: u8) -> bool {
        match self {
            McuFamily::Stm32F4 => pin <= Self::MAX_STM_PIN_NUM,
            McuFamily::Rp2350 => pin <= Self::MAX_RP2350_PIN_NUM,
        }
    }

    pub fn max_valid_addr_pin(&self) -> u8 {
        match self {
            McuFamily::Stm32F4 => Self::MAX_STM_PIN_NUM - 2, // Top two reserved for X1/X2
            McuFamily::Rp2350 => Self::MAX_RP2350_ADDR_CS_PIN_NUM,  // Any
        }
    }

    pub fn max_valid_addr_cs_pin(&self) -> u8 {
        match self {
            McuFamily::Stm32F4 => Self::MAX_STM_PIN_NUM,
            McuFamily::Rp2350 => Self::MAX_RP2350_ADDR_CS_PIN_NUM,
        }
    }

    pub fn max_valid_data_pin(&self) -> u8 {
        match self {
            McuFamily::Stm32F4 => Self::MAX_STM_DATA_PIN_NUM,
            McuFamily::Rp2350 => Self::MAX_RP2350_DATA_PIN_NUM,
        }
    }

    pub fn allowed_data_port(&self) -> Port {
        match self {
            McuFamily::Stm32F4 => Port::A,
            McuFamily::Rp2350 => Port::Zero,
        }
    }

    pub fn allowed_addr_port(&self) -> Port {
        match self {
            McuFamily::Stm32F4 => Port::C,
            McuFamily::Rp2350 => Port::Zero,
        }
    }

    pub fn allowed_cs_port(&self) -> Port {
        match self {
            McuFamily::Stm32F4 => Port::C,
            McuFamily::Rp2350 => Port::Zero,
        }
    }

    pub fn allowed_sel_port(&self) -> Port {
        match self {
            McuFamily::Stm32F4 => Port::B,
            McuFamily::Rp2350 => Port::Zero,
        }
    }

    pub fn valid_x1_pins(&self) -> Vec<u8> {
        match self {
            McuFamily::Stm32F4 => vec![14],
            McuFamily::Rp2350 => vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15],
        }
    }

    pub fn valid_x2_pins(&self) -> Vec<u8> {
        match self {
            McuFamily::Stm32F4 => vec![15],
            McuFamily::Rp2350 => vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15],
        }
    }
}

impl fmt::Display for McuFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McuFamily::Stm32F4 => write!(f, "F4"),
            McuFamily::Rp2350 => write!(f, "RP2350"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McuProcessor {
    F401BC,
    F401DE,
    F405,
    F411,
    F446,
    Rp2350,
}

impl McuProcessor {
    pub fn vco_min_mhz(&self) -> u32 {
        match self {
            McuProcessor::F401BC => 192,
            McuProcessor::F401DE => 192,
            McuProcessor::F405 => 100,
            McuProcessor::F411 => 100,
            McuProcessor::F446 => 100,
            McuProcessor::Rp2350 => 400,
        }
    }

    pub fn vco_max_mhz(&self, overclock: bool) -> u32 {
        if !overclock {
            match self {
                McuProcessor::Rp2350 => 1600,
                McuProcessor::F401BC
                | McuProcessor::F401DE
                | McuProcessor::F405
                | McuProcessor::F411
                | McuProcessor::F446 => 432,
            }
        } else {
            match self {
                McuProcessor::Rp2350 => 1600,
                McuProcessor::F401BC
                | McuProcessor::F401DE
                | McuProcessor::F405
                | McuProcessor::F411
                | McuProcessor::F446 => 800,
            }
        }
    }

    pub fn max_sysclk_mhz(&self) -> u32 {
        match self {
            McuProcessor::F401BC => 84,
            McuProcessor::F401DE => 84,
            McuProcessor::F405 => 168,
            McuProcessor::F411 => 100,
            McuProcessor::F446 => 180,
            McuProcessor::Rp2350 => 150,
        }
    }

    /// Calculate PLL values for target frequency using HSI (16 MHz)
    /// Returns (PLLM, PLLN, PLLP, PLLQ) or None if frequency not achievable
    fn calculate_stm32_pll_hsi(
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

    /// Calculate RP2350 PLL values for 12MHz XOSC input
    /// Returns (REFDIV, FBDIV, POSTDIV1, POSTDIV2) or None if not achievable
    fn calculate_rp2350_pll_12mhz(
        &self,
        target_freq_mhz: u32,
        overclock: bool,
    ) -> Option<(u8, u16, u8, u8)> {
        // Validate target frequency
        if target_freq_mhz > self.max_sysclk_mhz() && !overclock {
            return None;
        }
        
        const XOSC_MHZ: u32 = 12;
        const REFDIV: u8 = 1;  // Fixed for 12MHz
        
        // Try POSTDIV combinations (prefer higher PD1:PD2 ratios)
        for pd2 in 1..=7u8 {
            for pd1 in 1..=7u8 {
                let vco_mhz = target_freq_mhz * pd1 as u32 * pd2 as u32;
                
                if vco_mhz >= self.vco_min_mhz() && vco_mhz <= self.vco_max_mhz(overclock) {
                    let fbdiv = vco_mhz / XOSC_MHZ;
                    if (16..=320).contains(&fbdiv) && (vco_mhz % XOSC_MHZ == 0) {
                        return Some((REFDIV, fbdiv as u16, pd1, pd2));
                    }
                }
            }
        }
        None
    }

    /// Generate PLL #defines for target frequency
    fn generate_stm32_pll_defines(&self, target_freq_mhz: u32, overclock: bool) -> Option<String> {
        if let Some((m, n, p, q)) = self.calculate_stm32_pll_hsi(target_freq_mhz, overclock) {
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

    // Unlike the Pico SDK's vcocalc.py we are interested in power savings over decrease in jitter.
    fn generate_rp2350_pll_defines(&self, target_freq_mhz: u32, overclock: bool) -> Option<String> {
        if let Some((refdiv, fbdiv, postdiv1, postdiv2)) = self.calculate_rp2350_pll_12mhz(target_freq_mhz, overclock) {
            // Calculate intermediate values for comments
            const CLK_REF_MHZ: u32 = 12;
            let vco_input_mhz = CLK_REF_MHZ / refdiv as u32;
            let fvco_mhz = vco_input_mhz * fbdiv as u32;
            let sysclk_mhz = fvco_mhz / (postdiv1 as u32 * postdiv2 as u32);

            Some(format!(
                "//   CLK_REF={}MHz\n//   VCO_input={}MHz\n//   fVCO={}MHz\n//   SYSCLK={}MHz\n#define PLL_SYS_REFDIV    {}\n#define PLL_SYS_FBDIV     {}\n#define PLL_SYS_POSTDIV1  {}\n#define PLL_SYS_POSTDIV2  {}",
                CLK_REF_MHZ, vco_input_mhz, fvco_mhz, sysclk_mhz, refdiv, fbdiv, postdiv1, postdiv2
            ))
        } else {
            None
        }
    }

    pub fn generate_pll_defines(&self, target_freq_mhz: u32, overclock: bool) -> Option<String> {
        match self {
            McuProcessor::Rp2350 => self.generate_rp2350_pll_defines(target_freq_mhz, overclock),
            _ => self.generate_stm32_pll_defines(target_freq_mhz, overclock), // Rename existing function
        }
    }

    fn calculate_pll_hsi(
        &self,
        target_freq_mhz: u32,
        overclock: bool,
    ) -> Option<(u8, u16, u8, u8)> {
        match self {
            McuProcessor::Rp2350 => self.calculate_rp2350_pll_12mhz(target_freq_mhz, overclock),
            _ => self.calculate_stm32_pll_hsi(target_freq_mhz, overclock),
        }
    }

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
pub enum McuVariant {
    F446RC, // STM32F446RC (6 or 7), 64-pins, 128KB SRAM, 256KB Flash
    F446RE, // STM32F446RE (6 or 7), 64-pins, 128KB SRAM, 512KB Flash
    F411RC, // STM32F411RC (6 or 7), 64-pins, 128KB SRAM, 256KB Flash
    F411RE, // STM32F411RE (6 or 7), 64-pins, 128KB SRAM, 512KB Flash
    F405RG, // STM32F405RE (6 or 7), 64-pins, 128KB SRAM, 1024KB Flash (+ 64KB CCM RAM)
    F401RE, // STM32F401RE (6 or 7), 64-pins, 96KB SRAM, 512KB Flash
    F401RB, // STM32F401RB (6 or 7), 64-pins, 64KB SRAM, 128KB Flash
    F401RC, // STM32F401RC (6 or 7), 64-pins, 96KB SRAM, 256KB Flash
    Rp2350, // RP2350A, 60-pin, 2MB flash
}

impl McuVariant {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "f446rc" => Some(McuVariant::F446RC),
            "f446re" => Some(McuVariant::F446RE),
            "f411rc" => Some(McuVariant::F411RC),
            "f411re" => Some(McuVariant::F411RE),
            "f405rg" => Some(McuVariant::F405RG),
            "f401re" => Some(McuVariant::F401RE),
            "f401rb" => Some(McuVariant::F401RB),
            "f401rc" => Some(McuVariant::F401RC),
            "rp2350" => Some(McuVariant::Rp2350),
            _ => None,
        }
    }

    pub fn line_enum(&self) -> &str {
        match self {
            McuVariant::F446RC | McuVariant::F446RE => "F446",
            McuVariant::F411RC | McuVariant::F411RE => "F411",
            McuVariant::F405RG => "F405",
            McuVariant::F401RE => "F401DE",
            McuVariant::F401RB | McuVariant::F401RC => "F401BC",
            McuVariant::Rp2350 => "RP2350_LINE",
        }
    }

    pub fn storage_enum(&self) -> &str {
        match self {
            McuVariant::F446RC => "STORAGE_C",
            McuVariant::F446RE => "STORAGE_E",
            McuVariant::F411RC => "STORAGE_C",
            McuVariant::F411RE => "STORAGE_E",
            McuVariant::F405RG => "STORAGE_G",
            McuVariant::F401RE => "STORAGE_E",
            McuVariant::F401RB => "STORAGE_B",
            McuVariant::F401RC => "STORAGE_C",
            McuVariant::Rp2350 => "STORAGE_2MB",
        }
    }

    fn flash_storage_bytes(&self) -> usize {
        self.flash_storage_kb() * 1024
    }

    pub fn flash_storage_kb(&self) -> usize {
        match self {
            McuVariant::F446RC => 256,
            McuVariant::F446RE => 512,
            McuVariant::F411RC => 256,
            McuVariant::F411RE => 512,
            McuVariant::F405RG => 1024,
            McuVariant::F401RB => 128,
            McuVariant::F401RC => 256,
            McuVariant::F401RE => 512,
            McuVariant::Rp2350 => 2048,
        }
    }

    pub fn ram_kb(&self) -> usize {
        match self {
            McuVariant::F446RC | McuVariant::F446RE => 128,
            McuVariant::F411RC | McuVariant::F411RE => 128,
            McuVariant::F405RG => 128, // +64KB CCM RAM
            McuVariant::F401RB | McuVariant::F401RC => 64,
            McuVariant::F401RE => 96,
            McuVariant::Rp2350 => 520,
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
            McuVariant::F405RG => Some(64),
            _ => None,
        }
    }

    pub fn define_flash_size_bytes(&self) -> String {
        format!("#define MCU_FLASH_SIZE {}", self.flash_storage_bytes())
    }

    pub fn define_flash_size_kb(&self) -> String {
        format!("#define MCU_FLASH_SIZE_KB {}", self.flash_storage_kb())
    }

    pub fn define_ram_size_bytes(&self) -> String {
        format!("#define MCU_RAM_SIZE {}", self.ram_kb() * 1024)
    }

    pub fn define_ram_size_kb(&self) -> String {
        format!("#define MCU_RAM_SIZE_KB {}", self.ram_kb())
    }

    pub fn define_var_sub_fam(&self) -> &str {
        match self {
            McuVariant::F446RC | McuVariant::F446RE => "#define STM32F446      1",
            McuVariant::F411RC | McuVariant::F411RE => "#define STM32F411      1",
            McuVariant::F405RG => "#define STM32F405      1",
            McuVariant::F401RE => "#define STM32F401DE    1",
            McuVariant::F401RB | McuVariant::F401RC => "#define STM32F401BC    1",
            McuVariant::Rp2350 => "#define RP2350A        1",
        }
    }

    pub fn family(&self) -> McuFamily {
        match self {
            McuVariant::F446RC
            | McuVariant::F446RE
            | McuVariant::F411RC
            | McuVariant::F411RE
            | McuVariant::F405RG
            | McuVariant::F401RE
            | McuVariant::F401RB
            | McuVariant::F401RC => McuFamily::Stm32F4,
            McuVariant::Rp2350 => McuFamily::Rp2350,
        }
    }

    pub fn processor(&self) -> McuProcessor {
        match self {
            McuVariant::F446RC | McuVariant::F446RE => McuProcessor::F446,
            McuVariant::F411RC | McuVariant::F411RE => McuProcessor::F411,
            McuVariant::F405RG => McuProcessor::F405,
            McuVariant::F401RE => McuProcessor::F401DE,
            McuVariant::F401RB | McuVariant::F401RC => McuProcessor::F401BC,
            McuVariant::Rp2350 => McuProcessor::Rp2350,
        }
    }

    pub fn define_var_fam(&self) -> &str {
        match self.family() {
            McuFamily::Stm32F4 => "#define STM32F4        1",
            McuFamily::Rp2350 => "#define RP235X         1",
        }
    }

    pub fn define_var_str(&self) -> &str {
        match self {
            McuVariant::F446RC => "#define MCU_VARIANT    \"F446RC\"",
            McuVariant::F446RE => "#define MCU_VARIANT    \"F446RE\"",
            McuVariant::F411RC => "#define MCU_VARIANT    \"F411RC\"",
            McuVariant::F411RE => "#define MCU_VARIANT    \"F411RE\"",
            McuVariant::F405RG => "#define MCU_VARIANT    \"F405RG\"",
            McuVariant::F401RE => "#define MCU_VARIANT    \"F401RE\"",
            McuVariant::F401RB => "#define MCU_VARIANT    \"F401RB\"",
            McuVariant::F401RC => "#define MCU_VARIANT    \"F401RC\"",
            McuVariant::Rp2350 => "#define MCU_VARIANT    \"RP2350\"",
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
            McuVariant::F446RC => "stm32f446rc",
            McuVariant::F446RE => "stm32f446re",
            McuVariant::F411RC => "stm32f411rc",
            McuVariant::F411RE => "stm32f411re",
            McuVariant::F405RG => "stm32f405rg",
            McuVariant::F401RE => "stm32f401re",
            McuVariant::F401RB => "stm32f401rb",
            McuVariant::F401RC => "stm32f401rc",
            McuVariant::Rp2350 => "rp2350",
        }
    }

    /// Used to pass to probe-rs
    pub fn chip_id(&self) -> &str {
        match self {
            McuVariant::F446RC => "STM32F446RCTx",
            McuVariant::F446RE => "STM32F446RETx",
            McuVariant::F411RC => "STM32F411RCTx",
            McuVariant::F411RE => "STM32F411RETx",
            McuVariant::F405RG => "STM32F405RGTx",
            McuVariant::F401RE => "STM32F401RETx",
            McuVariant::F401RB => "STM32F401RBTx",
            McuVariant::F401RC => "STM32F401RCTx",
            McuVariant::Rp2350 => "RP235X",
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
