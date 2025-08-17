// Function prototypes

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef FUNCTIONS_H
#define FUNCTIONS_H

// main.c
extern int main(void);

// utils.c
extern uint32_t check_sel_pins(uint32_t *sel_mask);
#if defined(BOOT_LOGGING)
extern void log_init();
extern void do_log(const char *, ...);
#endif // BOOT_LOGGING
#if defined(MAIN_LOOP_LOGGING) || defined(DEBUG_LOGGING)
typedef void (*ram_log_fn)(const char*, ...);
#endif // MAIN_LOOP_LOGGING
#if defined(EXECUTE_FROM_RAM)
extern void copy_func_to_ram(void (*fn)(void), uint32_t ram_addr, size_t size);
extern void execute_ram_func(uint32_t ram_addr);
#endif // EXECUTE_FROM_RAM
extern void setup_status_led(void);
extern void delay(volatile uint32_t count);
extern void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats);

// rp235x.c
#if defined(RP235X)
extern void platform_specific_init(void);
extern void setup_clock(void);
extern void setup_gpio(void);
extern void setup_mco(void);
extern uint32_t setup_sel_pins(uint32_t *sel_mask);
extern uint32_t get_sel_value(uint32_t sel_mask);
extern void disable_sel_pins(void);
extern void setup_status_led(void);
extern void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats);
extern void enter_bootloader(void);
extern void platform_logging(void);

// Internal
extern void setup_xosc(void);
extern void setup_pll(void);
#endif // RP235X

// stm32f4.c
#if defined(STM32F4)
extern void platform_specific_init(void);
extern void setup_clock(void);
extern void setup_gpio(void);
extern void setup_mco(void);
extern uint32_t setup_sel_pins(uint32_t *sel_mask);
extern uint32_t get_sel_value(uint32_t sel_mask);
extern void disable_sel_pins(void);
extern void setup_status_led(void);
extern void blink_pattern(uint32_t on_time, uint32_t off_time, uint8_t repeats);
extern void enter_bootloader(void);
extern void platform_logging(void);

// Internal
extern void setup_pll_mul(uint8_t m, uint16_t n, uint8_t p, uint8_t q);
extern void setup_pll_src(uint8_t src);
extern void setup_pll(void);
extern void enable_pll(void);
extern void enable_hse(void);
extern uint8_t get_hsi_cal(void);
extern void set_clock(uint8_t clock);
extern void trim_hsi(uint8_t trim);
extern void set_bus_clks(void);
extern void set_flash_ws(void);
#endif // STM32F4

// rom_impl.c
#if !defined(TIMER_TEST) && !defined(TOGGLE_PA4)
extern void main_loop(
#ifndef EXECUTE_FROM_RAM
    const sdrr_info_t *info,
    const sdrr_rom_set_t *set
#else // EXECUTE_FROM_RAM
    sdrr_info_t *info,
    sdrr_rom_set_t *set
#endif // !EXECUTE_FROM_RAM
);
extern uint8_t get_rom_set_index(void);
extern void* preload_rom_image(const sdrr_rom_set_t *set);
#endif // !TIMER_TEST && !TOGGLE_PA4

// test function prototypes
#if defined(TIMER_TEST) || defined(TOGGLE_PA4)
extern void main_loop(void);
#endif // TIMER_TEST || TOGGLE_PA4

#endif // FUNCTIONS_H