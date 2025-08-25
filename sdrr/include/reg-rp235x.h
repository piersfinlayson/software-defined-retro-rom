// RP235X registers header file

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef REG_RP235X_H
#define REG_RP235X_H

#if !defined(RP235X)
#error "RP235X family not defined"
#endif // RP235X

// Using Winbond W25Q16JV 2MB (16MBit)
#define MAX_FLASH_CLOCK_FREQ_MHZ 133
#define FLASH_SIZE_KB 2048
#if FLASH_SIZE_KB != MCU_FLASH_SIZE_KB
#error "Flash size mismatch"
#endif

#define RP2350_RAM_SIZE_KB 520
#if MCU_RAM_SIZE_KB != RP2350_RAM_SIZE_KB
#error "RAM size mismatch"
#endif

// Register base addresses
#define FLASH_BASE      0x10000000
#define XIP_CACHE_BASE  0x18000000
#define SYSINFO_BASE    0x40000000
#define CLOCKS_BASE     0x40010000
#define RESETS_BASE     0x40020000
#define IO_BANK0_BASE   0x40028000
#define PADS_BANK0_BASE 0x40038000
#define XOSC_BASE       0x40048000  
#define PLL_SYS_BASE    0x40050000
#define PLL_USB_BASE    0x40058000
#define ADC_BASE        0x400a0000
#define XIP_CTRL_BASE   0x400c8000
#define XIP_QMI_BASE    0x400d0000
#define POWMAN_BASE     0x40100000
#define OTP_BASE        0x40120000
#define SIO_BASE        0xD0000000
#define SCB_BASE        0xE000ED00

// SysInfo Registers
#define SYSINFO_CHIP_ID         (*((volatile uint32_t *)(SYSINFO_BASE + 0x00)))
#define SYSINFO_PACKAGE_SEL     (*((volatile uint32_t *)(SYSINFO_BASE + 0x04)))
#define SYSINFO_GITREF_RP2350   (*((volatile uint32_t *)(SYSINFO_BASE + 0x14)))

// Clock registers
#define CLOCK_CLK_GPOUT0_CTRL   (*((volatile uint32_t *)(CLOCKS_BASE + 0x00)))
#define CLOCK_CLK_GPOUT0_DIV    (*((volatile uint32_t *)(CLOCKS_BASE + 0x04)))
#define CLOCK_CLK_GPOUT0_SEL    (*((volatile uint32_t *)(CLOCKS_BASE + 0x08)))
#define CLOCK_REF_CTRL          (*((volatile uint32_t *)(CLOCKS_BASE + 0x30)))
#define CLOCK_REF_SELECTED      (*((volatile uint32_t *)(CLOCKS_BASE + 0x38)))
#define CLOCK_SYS_CTRL          (*((volatile uint32_t *)(CLOCKS_BASE + 0x3C)))
#define CLOCK_SYS_SELECTED      (*((volatile uint32_t *)(CLOCKS_BASE + 0x44)))
#define CLOCK_ADC_CTRL          (*((volatile uint32_t *)(CLOCKS_BASE + 0x6C)))

#define CLOCK_REF_SRC_XOSC      0x02
#define CLOCK_REF_SRC_SEL_MASK  0b1111
#define CLOCK_REF_SRC_SEL_XOSC  (1 << 2)

#define CLOCK_SYS_SRC_AUX           (1 << 0)
#define CLOCK_SYS_AUXSRC_PLL_SYS    (0x0 << 5)

#define CLOCK_ADC_ENABLE    (1 << 11)
#define CLOCK_ADC_ENABLED   (1 << 28)

// Reset registers
#define RESET_RESET     (*((volatile uint32_t *)(RESETS_BASE + 0x00)))
#define RESET_WDSEL     (*((volatile uint32_t *)(RESETS_BASE + 0x04)))
#define RESET_DONE      (*((volatile uint32_t *)(RESETS_BASE + 0x08)))

#define RESET_ADC           (1 << 0)
#define RESET_IOBANK0       (1 << 6)
#define RESET_PADS_BANK0    (1 << 9)
#define RESET_JTAG          (1 << 8)
#define RESET_PLL_SYS       (1 << 14)
#define RESET_PLL_USB       (1 << 15)
#define RESET_SYSINFO       (1 << 21)

// GPIO registers
#define GPIO_STATUS_OFFSET  0x000
#define GPIO_CTRL_OFFSET    0x004
#define GPIO_SPACING        0x008

#define GPIO_STATUS_INFROMPAD_BIT  17

#define GPIO_STATUS(pin)    (*(volatile uint32_t*)(IO_BANK0_BASE + GPIO_STATUS_OFFSET + pin*GPIO_SPACING))
#define GPIO_CTRL(pin)      (*(volatile uint32_t*)(IO_BANK0_BASE + GPIO_CTRL_OFFSET + pin*GPIO_SPACING))
#define GPIO_READ(pin)      ((GPIO_STATUS(pin) >> GPIO_STATUS_INFROMPAD_BIT) & 1)

#define GPIO_CTRL_FUNC_SIO      0x05
#define GPIO_CTRL_RESET         GPIO_CTRL_FUNC_SIO

// Pads registers
#define PAD_OFFSET_START    0x004
#define PAD_SPACING         0x004
#define GPIO_PAD(pin)       (*(volatile uint32_t*)(PADS_BANK0_BASE + PAD_OFFSET_START + pin*PAD_SPACING))  

// Pad control bits
#define PAD_SLEW_FAST_BIT   0
#define PAD_PDE_BIT         2
#define PAD_PUE_BIT         3
#define PAD_DRIVE_BIT       4
#define PAD_IE_BIT          6
#define PAD_OD_BIT          7
#define PAD_ISO             8
#define PAD_DRIVE_MASK      0x3
#define PAD_DRIVE_2MA       0x0
#define PAD_DRIVE_4MA       0x1
#define PAD_DRIVE_8MA       0x2
#define PAD_DRIVE_12MA      0x3
#define PAD_DRIVE(X)        ((X & PAD_DRIVE_MASK) << PAD_DRIVE_BIT)
#define PAD_SLEW_FAST       (1 << PAD_SLEW_FAST_BIT)
#define PAD_INPUT           (1 << PAD_IE_BIT)
#define PAD_OUTPUT_DISABLE  (1 << PAD_OD_BIT)
#define PAD_PU              (1 << PAD_PUE_BIT)
#define PAD_PD              (1 << PAD_PDE_BIT)
#define PAD_INPUT_PD        ((1 << PAD_PDE_BIT) | (1 << PAD_IE_BIT))
#define PAD_INPUT_PU        ((1 << PAD_PUE_BIT) | (1 << PAD_IE_BIT))

// Crystal Oscillator Registers
#define XOSC_CTRL       (*((volatile uint32_t *)(XOSC_BASE + 0x00)))
#define XOSC_STATUS     (*((volatile uint32_t *)(XOSC_BASE + 0x04)))
#define XOSC_DORMANT    (*((volatile uint32_t *)(XOSC_BASE + 0x08)))
#define XOSC_STARTUP    (*((volatile uint32_t *)(XOSC_BASE + 0x0C)))
#define XOSC_COUNT      (*((volatile uint32_t *)(XOSC_BASE + 0x10)))

// XOSC Values - See datasheet S8.2
#define XOSC_STARTUP_DELAY_1MS  47
#define XOSC_ENABLE         (0xfab << 12)
#define XOSC_RANGE_1_15MHz  0xaa0
#define XOSC_STATUS_STABLE  (1 << 31)

// PLL Registers
#define PLL_SYS_CS          (*((volatile uint32_t *)(PLL_SYS_BASE + 0x00)))
#define PLL_SYS_PWR         (*((volatile uint32_t *)(PLL_SYS_BASE + 0x04)))
#define PLL_SYS_FBDIV_INT   (*((volatile uint32_t *)(PLL_SYS_BASE + 0x08)))
#define PLL_SYS_PRIM        (*((volatile uint32_t *)(PLL_SYS_BASE + 0x0C)))
#define PLL_SYS_INTR        (*((volatile uint32_t *)(PLL_SYS_BASE + 0x10)))
#define PLL_SYS_INTE        (*((volatile uint32_t *)(PLL_SYS_BASE + 0x14)))
#define PLL_SYS_INTF        (*((volatile uint32_t *)(PLL_SYS_BASE + 0x18)))
#define PLL_SYS_INTS        (*((volatile uint32_t *)(PLL_SYS_BASE + 0x1C)))

// PLL Control/Status bits
#define PLL_CS_LOCK         (1 << 31)
#define PLL_CS_BYPASS       (1 << 8)
#define PLL_CS_REFDIV_MASK  0b111111
#define PLL_CS_REFDIV(X)    (((X) & PLL_CS_REFDIV_MASK) << PLL_CS_REFDIV_SHIFT)
#define PLL_CS_REFDIV_SHIFT 0

// PLL Power bits  
#define PLL_PWR_PD          (1 << 0)    // Power down
#define PLL_PWR_DSMPD       (1 << 2)    // DSM power down  
#define PLL_PWR_POSTDIVPD   (1 << 3)    // Post divider power down
#define PLL_PWR_VCOPD       (1 << 5)    // VCO power down

// PLL Post divider bits
#define PLL_PRIM_POSTDIV1(X)    (((X) & PLL_PRIM_POSTDIV_MASK) << 16)
#define PLL_PRIM_POSTDIV2(X)    (((X) & PLL_PRIM_POSTDIV_MASK) << 12)
#define PLL_PRIM_POSTDIV_MASK   0x7

// USB PLL Registers
#define PLL_USB_CS          (*((volatile uint32_t *)(PLL_USB_BASE + 0x00)))
#define PLL_USB_PWR         (*((volatile uint32_t *)(PLL_USB_BASE + 0x04)))
#define PLL_USB_FBDIV_INT   (*((volatile uint32_t *)(PLL_USB_BASE + 0x08)))
#define PLL_USB_PRIM        (*((volatile uint32_t *)(PLL_USB_BASE + 0x0C)))


// ADC Registers
#define ADC_CS              (*((volatile uint32_t *)(ADC_BASE + 0x00)))
#define ADC_RESULT          (*((volatile uint32_t *)(ADC_BASE + 0x04)))

#define ADC_RESULT_MASK       0xFFF

#define ADC_CS_AINSEL_MASK   0b1111
#define ADC_CS_AINSEL_SHIFT  12
#define ADC_CS_AINSEL(X)     (((X) & ADC_CS_AINSEL_MASK) << ADC_CS_AINSEL_SHIFT)
#define ADC_CS_TS           4
#define ADC_CS_READY        (1 << 8)
#define ADC_CS_START_ONCE   (1 << 2)
#define ADC_CS_TS_EN        (1 << 1)
#define ADC_CS_EN           (1 << 0)

// XIP_CTRL Registers
#define XIP_CTRL_CTRL       (*((volatile uint32_t *)(XIP_CTRL_BASE + 0x00)))
#define XIP_CTRL_STATUS     (*((volatile uint32_t *)(XIP_CTRL_BASE + 0x08)))
#define XIP_CTRL_CTR_HIT    (*((volatile uint32_t *)(XIP_CTRL_BASE + 0x0C)))
#define XIP_CTRL_CTR_ACC    (*((volatile uint32_t *)(XIP_CTRL_BASE + 0x10)))

// XIP_QMI Registers
#define XIP_QMI_M0_TIMING   (*((volatile uint32_t *)(XIP_QMI_BASE + 0x0C)))

#define XIP_QMI_M0_CLKDIV_MASK   0xFF
#define XIP_QMI_M0_CLKDIV_SHIFT  0

// Power Manager Registers
#define POWMAN_VREG_CTRL    (*((volatile uint32_t *)(POWMAN_BASE + 0x04)))
#define POWMAN_VREG_STATUS  (*((volatile uint32_t *)(POWMAN_BASE + 0x08)))
#define POWMAN_VREG         (*((volatile uint32_t *)(POWMAN_BASE + 0x0C)))

#define POWMAN_PASSWORD     (0x5AFE << 16)

#define POWMAN_VREG_CTRL_UNLOCK (1 << 13)
#define POWMAN_VREG_CTRL_DISABLE_VOLTAGE_LIMIT (1 << 8)
#define HT_TH_100       0x0
#define HT_TH_105       0x1
#define HT_TH_110       0x2
#define HT_TH_115       0x3
#define HT_TH_120       0x4
#define HT_TH_125       0x5
#define HT_TH_135       0x6
#define HT_TH_150       0x7
#define HT_TH_SHIFT     4
#define HT_TH_MASK      0b111
#define POWMAN_VREG_CTRL_HT_TH(X)   (((X) & HT_TH_MASK) << HT_TH_SHIFT)

#define VREG_MASK       0b11111
#define VREG_SHIFT      4
#define VREG_1_10V      0b01011
#define VREG_1_15V      0b01100
#define VREG_1_20V      0b01101
#define VREG_1_25V      0b01110
#define VREG_1_30V      0b01111
#define VREG_1_35V      0b10000
#define VREG_1_40V      0b10001
#define VREG_1_50V      0b10010
#define VREG_1_60V      0b10011
#define POWMAN_VREG_VOLTAGE(X)   (((X) & VREG_MASK) << VREG_SHIFT)
#define POWMAN_VREG_UPDATE (1 << 15)

// SIO Registers
#define SIO_CPUID           (*((volatile uint32_t *)(SIO_BASE + 0x00)))
#define SIO_GPIO_IN         (*((volatile uint32_t *)(SIO_BASE + 0x04)))
#define SIO_GPIO_OUT        (*((volatile uint32_t *)(SIO_BASE + 0x10)))
#define SIO_GPIO_OUT_SET    (*((volatile uint32_t *)(SIO_BASE + 0x18)))
#define SIO_GPIO_OUT_CLR    (*((volatile uint32_t *)(SIO_BASE + 0x20)))
#define SIO_GPIO_OE         (*((volatile uint32_t *)(SIO_BASE + 0x30)))
#define SIO_GPIO_OE_SET     (*((volatile uint32_t *)(SIO_BASE + 0x38)))
#define SIO_GPIO_OE_CLR     (*((volatile uint32_t *)(SIO_BASE + 0x40)))

// SCB Registers
#define SCB_CPACR           (*((volatile uint32_t *)(SCB_BASE + 0x88)))
#define SCB_CPACR_CP0_FULL  (0b11 << 0)

// Used by assembly
#define VAL_SIO_GPIO_IN         (SIO_BASE + 0x04)
#define VAL_SIO_GPIO_OUT        (SIO_BASE + 0x10)
#define VAL_SIO_GPIO_OE         (SIO_BASE + 0x30)

// RAM Size
#define RP2350_RAM_SIZE_KB  520

// Maximum number of used GPIOs - those exposed on the QFN60 RP2350A
#define MAX_USED_GPIOS      30


// Boot block structure
typedef struct {
    uint32_t start_marker;          // 0xffffded3, start market
    uint8_t  image_type_tag;        // 0x42, image type
    uint8_t  image_type_len;        // 0x1, item is one word in size
    uint16_t image_type_data;       // 0b0001000000100001, RP2350, ARM, Secure, EXE
    uint8_t  type;                  // 0xff, size type, last item
    uint16_t size;                  // 0x0001, size
    uint8_t  pad;                   // 0
    uint32_t next_block;            // 0, link to self, no next block
    uint32_t end_marker;            // 0xab123579, end marker
} __attribute__((packed)) rp2350_boot_block_t;

#endif // REG_RP235X_H