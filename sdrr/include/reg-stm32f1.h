// STM32F1 family registers header file

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef REG_STM32F1_H
#define REG_STM32F1_H

#if !defined(STM32F1)
#error "STM32F1 family not defined"
#endif // !STM32F1

// Register base addresses
#define AFIO_BASE       0x40010000
#define GPIOA_BASE      0x40010800
#define GPIOB_BASE      0x40010C00
#define GPIOC_BASE      0x40011000
#define RCC_BASE        0x40021000
#define FLASH_REG_BASE  0x40022000

// AFIO registers
#define AFIO_MAPR       (*(volatile uint32_t *)(AFIO_BASE + 0x04))

// RCC registers
#define RCC_CR          (*(volatile uint32_t *)(RCC_BASE + 0x00))
#define RCC_CFGR        (*(volatile uint32_t *)(RCC_BASE + 0x04))
#define RCC_CIR         (*(volatile uint32_t *)(RCC_BASE + 0x08))
#define RCC_APB2RSTR    (*(volatile uint32_t *)(RCC_BASE + 0x0C))
#define RCC_APB1RSTR    (*(volatile uint32_t *)(RCC_BASE + 0x10))
#define RCC_AHBENR      (*(volatile uint32_t *)(RCC_BASE + 0x14))
#define RCC_APB2ENR     (*(volatile uint32_t *)(RCC_BASE + 0x18))
#define RCC_APB1ENR     (*(volatile uint32_t *)(RCC_BASE + 0x1C))
#define RCC_BDCR        (*(volatile uint32_t *)(RCC_BASE + 0x20))
#define RCC_CSR         (*(volatile uint32_t *)(RCC_BASE + 0x24))

// Flash registers
#define FLASH_ACR       (*(volatile uint32_t *)(FLASH_REG_BASE + 0x00))

// GPIO registers
#define GPIO_CRL_OFFSET 0x00
#define GPIO_CRH_OFFSET 0x04
#define GPIO_IDR_OFFSET 0x08
#define GPIO_ODR_OFFSET 0x0C
#define GPIO_BSRR_OFFSET 0x10
#define GPIOA_CRL       (*(volatile uint32_t *)(GPIOA_BASE + GPIO_CRL_OFFSET))
#define GPIOA_CRH       (*(volatile uint32_t *)(GPIOA_BASE + GPIO_CRH_OFFSET))
#define GPIOA_IDR       (*(volatile uint32_t *)(GPIOA_BASE + GPIO_IDR_OFFSET))
#define GPIOA_ODR       (*(volatile uint32_t *)(GPIOA_BASE + GPIO_ODR_OFFSET))
#define GPIOA_BSRR      (*(volatile uint32_t *)(GPIOA_BASE + GPIO_BSRR_OFFSET))
#define GPIOB_CRL       (*(volatile uint32_t *)(GPIOB_BASE + GPIO_CRL_OFFSET))
#define GPIOB_CRH       (*(volatile uint32_t *)(GPIOB_BASE + GPIO_CRH_OFFSET))
#define GPIOB_IDR       (*(volatile uint32_t *)(GPIOB_BASE + GPIO_IDR_OFFSET))
#define GPIOB_ODR       (*(volatile uint32_t *)(GPIOB_BASE + GPIO_ODR_OFFSET))
#define GPIOC_CRL       (*(volatile uint32_t *)(GPIOC_BASE + GPIO_CRL_OFFSET))
#define GPIOC_CRH       (*(volatile uint32_t *)(GPIOC_BASE + GPIO_CRH_OFFSET))
#define GPIOC_IDR       (*(volatile uint32_t *)(GPIOC_BASE + GPIO_IDR_OFFSET))
#define GPIOC_ODR       (*(volatile uint32_t *)(GPIOC_BASE + GPIO_ODR_OFFSET))

// AFIO mask
#define AFIO_MAPR_RSVD_RO_MASK  (0b1 << 31) | \
                                (0b1 << 27) | \
                                (0b111 << 21)

// Flash configuration values
#define FLASH_ACR_LATENCY_MASK (0x07) // Latency mask
#define FLASH_ACR_PRFTBE      (0x01 << 4) // Prefetch buffer enable

// GPIO configuration values
#define GPIO_MODE_OUTPUT_50MHZ   (0x03)
#define GPIO_CNF_OUTPUT_PUSHPULL (0x00)

// RCC_CR bit definitions
#define RCC_CR_HSION        (1 << 0)
#define RCC_CR_HSIRDY       (1 << 1)
#define RCC_CR_HSEON        (1 << 16)
#define RCC_CR_HSERDY       (1 << 17)
#define RCC_CR_PLLON        (1 << 24)
#define RCC_CR_PLLRDY       (1 << 25)
#define RCC_CR_HSITRIM_MAX  (0x1F << 3) // Max HSI trim value
#define RCC_CFGR_PLLSRC     (1 << 16)
#define RCC_CFGR_PLLSRC_HSI_DIV2 (0x00 << 16) // HSI/2 as PLL source

// RCC_CFGR Clock configuration values
#define RCC_CFGR_SW_PLL     (0x02 << 0)
#define RCC_CFGR_SW_MASK    (0x03 << 0)
#define RCC_CFGR_SWS_PLL    (0x02 << 2)
#define RCC_CFGR_SWS_MASK   (0x03 << 2)
#define RCC_CFGR_PPRE1_MASK (0x07 << 8)
#define RCC_CFGR_PPRE1_DIV2 (0x04 << 8)  // APB1 prescaler
#define RCC_CFGR_PPRE2_MASK (0x07 << 11)
#define RCC_CFGR_PLLSRC_HSI 0x00  // HSI/2 as PLL source
#define RCC_CFGR_PLLSRC_HSE 0x01  // HSE as PLL source

// RCC mask definitions
#define RCC_CR_RSVD_RO_MASK     (0b111111 << 26) |   \
                                (0b1 << 25) |        \
                                (0b1111 << 20) |     \
                                (0b1 << 17) |        \
                                (0b11111111 << 8) |  \
                                (0b1 << 2) |         \
                                (0b1 << 1)
#define RCC_CFGR_RSVD_RO_MASK   (0b11111 << 27) |    \
                                (0b1 << 23) |        \
                                (0b11 << 2)
#define RCC_CIR_RSVD_RO_MASK    (0b11111111 << 24) | \
                                (0b1 << 23) |        \
                                (0b11 << 21) |       \
                                (0b11111 << 16) |    \
                                (0b111 < 13) |       \
                                (0b1 << 7) |         \
                                (0b11 << 5) |        \
                                (0b11111 << 0)
#define RCC_APB2RSTR_RSVD_RO_MASK   (0b1111111111 << 22) | \
                                    (0b111 << 16) |        \
                                    (0b1 << 1)
#define RCC_APB1RSTR_RSVD_RO_MASK   (0b11 << 30) |   \
                                    (0b1 << 26) |    \
                                    (0b1 << 24) |    \
                                    (0b1 << 16) |    \
                                    (0b11 << 12) |   \
                                    (0b11 << 9)
#define RCC_AHBENR_RSVD_RO_MASK     (0b111111111111111111111 << 11) | \
                                    (0b1 << 9) |                      \
                                    (0b1 << 7) |                      \
                                    (0b1 << 5) |                      \
                                    (0b1 << 3)
#define RCC_APB2ENR_RSVD_RO_MASK    (0b1111111111 << 22) | \
                                    (0b111 << 16) |        \
                                    (0b1 << 1)
#define RCC_APB1ENR_RSVD_RO_MASK    (0b11 << 30) |   \
                                    (0b1 << 26) |    \
                                    (0b1 << 24) |    \
                                    (0b1 << 16) |    \
                                    (0b11 << 12) |   \
                                    (0b11 << 9)
#define RCC_BDCR_RSVD_RO_MASK   (0b111111111111111 << 17) | \
                                (0b11111 << 10) |           \
                                (0b11111 << 3) |            \
                                (0b1 << 1)
#define RCC_CSR_RSVD_RO_MASK    (0b1 << 25) |        \
                                (0b11111111 << 16) | \
                                (0b11111111 << 8) |  \
                                (0b111111 << 2) |    \
                                (0b1 << 1)
#define RCC_CFGR_PLLXTPRE_MASK (0b1 << 17)
#define RCC_CFGR_PLLMULL_MASK (0b1111 << 18)
#define RCC_CFGR_PLLXTPRE_MASK (0b1 << 17)
#define RCC_CFGR_PLLMULL_MASK (0b1111 << 18)
#define RCC_CFGR_MCO_MASK (0b111 << 24)
#define RCC_CFGR_HPRE_MASK (0b1111 << 4)

// RCC bit definitions
#define RCC_CFGR_MCO_NO_CLK 0b000 // No clock
#define RCC_CFGR_MCO_SYSCLK 0b100 // System clock
#define RCC_CFGR_MCO_HSI 0b101 // HSI clock
#define RCC_CFGR_MCO_HSE 0b110 // HSE clock
#define RCC_CFGR_MCO_PLL_DIV2 0b111 // PLL/2 clock

#endif // REG_STM32F1_H