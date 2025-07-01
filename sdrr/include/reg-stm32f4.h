// STM32F4 family registers header file

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef REG_STM32F4_H
#define REG_STM32F4_H

#if !defined(STM32F4)
#error "STM32F4 family not defined"
#endif // !STM32F4

// Register base addresses
#define RCC_BASE        0x40023800
#define FLASH_REG_BASE  0x40023C00
#define GPIOA_BASE      0x40020000
#define GPIOB_BASE      0x40020400
#define GPIOC_BASE      0x40020800
#define PWR_BASE        0x40007000

// Power registers
#define PWR_CR          (*(volatile uint32_t *)(PWR_BASE + 0x00))
#define PWR_VOS_MASK    (0b11 << 14)
#define PWR_VOS_SCALE_1 (0b11 << 14)  // Scale 1 mode (100MHz max clock speed)

// RCC registers
#define RCC_AHB1ENR_OFFSET 0x30
#define RCC_CR          (*(volatile uint32_t *)(RCC_BASE + 0x00))
#define RCC_PLLCFGR     (*(volatile uint32_t *)(RCC_BASE + 0x04))
#define RCC_CFGR        (*(volatile uint32_t *)(RCC_BASE + 0x08))
#define RCC_CIR         (*(volatile uint32_t *)(RCC_BASE + 0x0C))
#define RCC_AHB1RSTR    (*(volatile uint32_t *)(RCC_BASE + 0x10))
#define RCC_AHB2RSTR    (*(volatile uint32_t *)(RCC_BASE + 0x14))
#define RCC_APB1RSTR    (*(volatile uint32_t *)(RCC_BASE + 0x20))
#define RCC_APB2RSTR    (*(volatile uint32_t *)(RCC_BASE + 0x24))
#define RCC_AHB1ENR     (*(volatile uint32_t *)(RCC_BASE + RCC_AHB1ENR_OFFSET))
#define RCC_AHB2ENR     (*(volatile uint32_t *)(RCC_BASE + 0x34))
#define RCC_APB1ENR     (*(volatile uint32_t *)(RCC_BASE + 0x40))
#define RCC_APB2ENR     (*(volatile uint32_t *)(RCC_BASE + 0x44))
#define RCC_BDCR        (*(volatile uint32_t *)(RCC_BASE + 0x70))
#define RCC_CSR         (*(volatile uint32_t *)(RCC_BASE + 0x74))

// Flash registers
#define FLASH_ACR       (*(volatile uint32_t *)(FLASH_REG_BASE + 0x00))

// GPIO registers
#define GPIO_MODER_OFFSET  0x00
#define GPIO_OTYPER_OFFSET 0x04
#define GPIO_OSPEEDR_OFFSET 0x08
#define GPIO_PUPDR_OFFSET  0x0C
#define GPIO_IDR_OFFSET    0x10
#define GPIO_ODR_OFFSET    0x14
#define GPIO_BSRR_OFFSET   0x18
#define GPIO_LCKR_OFFSET   0x1C
#define GPIO_AFRL_OFFSET   0x20
#define GPIO_AFRH_OFFSET   0x24
#define GPIOA_MODER        (*(volatile uint32_t *)(GPIOA_BASE + GPIO_MODER_OFFSET))
#define GPIOA_OTYPER       (*(volatile uint32_t *)(GPIOA_BASE + GPIO_OTYPER_OFFSET))
#define GPIOA_OSPEEDR      (*(volatile uint32_t *)(GPIOA_BASE + GPIO_OSPEEDR_OFFSET))
#define GPIOA_PUPDR        (*(volatile uint32_t *)(GPIOA_BASE + GPIO_PUPDR_OFFSET))
#define GPIOA_IDR          (*(volatile uint32_t *)(GPIOA_BASE + GPIO_IDR_OFFSET))
#define GPIOA_ODR          (*(volatile uint32_t *)(GPIOA_BASE + GPIO_ODR_OFFSET))
#define GPIOA_BSRR         (*(volatile uint32_t *)(GPIOA_BASE + GPIO_BSRR_OFFSET))
#define GPIOA_LCKR         (*(volatile uint32_t *)(GPIOA_BASE + GPIO_LCKR_OFFSET))
#define GPIOA_AFRL         (*(volatile uint32_t *)(GPIOA_BASE + GPIO_AFRL_OFFSET))
#define GPIOA_AFRH         (*(volatile uint32_t *)(GPIOA_BASE + GPIO_AFRH_OFFSET))
#define GPIOB_MODER        (*(volatile uint32_t *)(GPIOB_BASE + GPIO_MODER_OFFSET))
#define GPIOB_OTYPER       (*(volatile uint32_t *)(GPIOB_BASE + GPIO_OTYPER_OFFSET))
#define GPIOB_OSPEEDR      (*(volatile uint32_t *)(GPIOB_BASE + GPIO_OSPEEDR_OFFSET))
#define GPIOB_PUPDR        (*(volatile uint32_t *)(GPIOB_BASE + GPIO_PUPDR_OFFSET))
#define GPIOB_IDR          (*(volatile uint32_t *)(GPIOB_BASE + GPIO_IDR_OFFSET))
#define GPIOB_ODR          (*(volatile uint32_t *)(GPIOB_BASE + GPIO_ODR_OFFSET))
#define GPIOB_BSRR         (*(volatile uint32_t *)(GPIOB_BASE + GPIO_BSRR_OFFSET))
#define GPIOB_LCKR         (*(volatile uint32_t *)(GPIOB_BASE + GPIO_LCKR_OFFSET))
#define GPIOB_AFRL         (*(volatile uint32_t *)(GPIOB_BASE + GPIO_AFRL_OFFSET))
#define GPIOB_AFRH         (*(volatile uint32_t *)(GPIOB_BASE + GPIO_AFRH_OFFSET))
#define GPIOC_MODER        (*(volatile uint32_t *)(GPIOC_BASE + GPIO_MODER_OFFSET))
#define GPIOC_OTYPER       (*(volatile uint32_t *)(GPIOC_BASE + GPIO_OTYPER_OFFSET))
#define GPIOC_OSPEEDR      (*(volatile uint32_t *)(GPIOC_BASE + GPIO_OSPEEDR_OFFSET))
#define GPIOC_PUPDR        (*(volatile uint32_t *)(GPIOC_BASE + GPIO_PUPDR_OFFSET))
#define GPIOC_IDR          (*(volatile uint32_t *)(GPIOC_BASE + GPIO_IDR_OFFSET))
#define GPIOC_ODR          (*(volatile uint32_t *)(GPIOC_BASE + GPIO_ODR_OFFSET))
#define GPIOC_BSRR         (*(volatile uint32_t *)(GPIOC_BASE + GPIO_BSRR_OFFSET))
#define GPIOC_LCKR         (*(volatile uint32_t *)(GPIOC_BASE + GPIO_LCKR_OFFSET))
#define GPIOC_AFRL         (*(volatile uint32_t *)(GPIOC_BASE + GPIO_AFRL_OFFSET))
#define GPIOC_AFRH         (*(volatile uint32_t *)(GPIOC_BASE + GPIO_AFRH_OFFSET))

// Hardware register addresses as values, for use directly in CPU registers
// in assembly code 
#define VAL_GPIOA_IDR     (GPIOA_BASE + GPIO_IDR_OFFSET)
#define VAL_GPIOA_ODR     (GPIOA_BASE + GPIO_ODR_OFFSET) 
#define VAL_GPIOA_MODER   (GPIOA_BASE + GPIO_MODER_OFFSET)
#define VAL_GPIOA_PUPDR   (GPIOA_BASE + GPIO_PUPDR_OFFSET)
#define VAL_GPIOB_IDR     (GPIOB_BASE + GPIO_IDR_OFFSET)
#define VAL_GPIOB_MODER   (GPIOB_BASE + GPIO_MODER_OFFSET)
#define VAL_GPIOB_PUPDR   (GPIOB_BASE + GPIO_PUPDR_OFFSET)
#define VAL_GPIOC_IDR     (GPIOC_BASE + GPIO_IDR_OFFSET)
#define VAL_GPIOC_MODER   (GPIOC_BASE + GPIO_MODER_OFFSET)
#define VAL_GPIOC_PUPDR   (GPIOC_BASE + GPIO_PUPDR_OFFSET)
#define VAL_RCC_AHB1ENR   (RCC_BASE + RCC_AHB1ENR_OFFSET)

// RCC mask definitions
#define RCC_CR_RSVD_RO_MASK     (0b1111 << 28) |     \
                                (0b1 << 27) |        \
                                (0b1 << 25) |        \
                                (0b1111 << 20) |     \
                                (0b1 << 17) |        \
                                (0b11111111 << 8) |  \
                                (0b1 << 2) |         \
                                (0b1 << 1)
#define RCC_PLLCFGR_RSVD_RO_MASK    (0b1111 << 28) | \
                                    (0b1 << 23) |    \
                                    (0b1111 << 18) | \
                                    (0b1 << 15)
#define RCC_CFGR_RSVD_RO_MASK   (0b11 << 8) | \
                                (0b11 << 2)
#define RCC_CIR_RSVD_RO_MASK    (0b11111111 << 24) | \
                                (0b1 << 23) |        \
                                (0b1 << 22) |        \
                                (0b111111 << 16) |   \
                                (0b11 < 14) |        \
                                (0b1 << 7) |         \
                                (0b1 << 6) |         \
                                (0b111111 << 0)
#define RCC_PLLCFGR_PLLSRC_MASK (1 << 22)
#define RCC_PLLCFGR_PLLSRC_HSI  0
#define RCC_PLLCFGR_PLLSRC_HSE  1

#define RCC_CFGR_MCO2_MASK (0b11 << 30)
#define RCC_CFGR_MCO1_MASK (0b11 << 21)
#define RCC_CFGR_PPRE2_MASK (0b111 << 13)
#define RCC_CFGR_PPRE1_MASK (0b111 << 10)
#define RCC_CFGR_HPRE_MASK (0b1111 << 4)

// RCC bit and position definitions
#define RCC_CR_HSION    (1 << 0)
#define RCC_CR_HSIRDY   (1 << 1)
#define RCC_CR_HSEON    (1 << 16)
#define RCC_CR_HSERDY   (1 << 17)
#define RCC_CR_PLLON    (1 << 24)
#define RCC_CR_PLLRDY   (1 << 25)
#define RCC_CR_HSITRIM_MAX (0x1F << 3) // Max HSI trim value

// RCC bit definitions
#define RCC_CFGR_MCO2_SYSCLK  0b00
#define RCC_CFGR_MCO2_PLL     0b11
#define RCC_CFGR_MCO1_HSI     0b00
#define RCC_CFGR_MCO1_PLL     0b11

// Clock configuration values
#define RCC_CFGR_SW_HSI     (0b00 << 0)
#define RCC_CFGR_SW_HSE     (0b01 << 0)
#define RCC_CFGR_SW_PLL     (0b10 << 0)
#define RCC_CFGR_SW_MASK    (0b11 << 0)
#define RCC_CFGR_SWS_PLL    (0b10 << 2)
#define RCC_CFGR_SWS_MASK   (0b11 << 2)
#define RCC_CFGR_PPRE1_DIV2 (0x04 << 10)  // APB1 prescaler

// Clock enable bits
#define RCC_AHB1ENR_GPIOAEN (1 << 0)
#define RCC_AHB1ENR_GPIOBEN (1 << 1)
#define RCC_AHB1ENR_GPIOCEN (1 << 2)

// Flash configuration values
#if defined(STM32F405)
#define FLASH_ACR_LATENCY_MASK  (0x07 << 0) // Latency mask
#else // !STM32F405
#define FLASH_ACR_LATENCY_MASK  (0x0F << 0) // Latency mask
#endif // STM32F405
#define FLASH_ACR_PRFTEN    (0x01 << 8) // Prefetch enable
#define FLASH_ACR_ICEN      (0x01 << 9) // Instruction cache enable
#define FLASH_ACR_DCEN      (0x01 << 10) // Data cache enable

#endif // REG_STM32F4_H