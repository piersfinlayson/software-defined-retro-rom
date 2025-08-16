// One ROM utils

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"
#include "roms.h"

//
// Logging functions
//

#if defined(BOOT_LOGGING)
// Logging function to output various debug information via RTT
void log_init(void) {
    LOG("%s", log_divider);
    LOG("%s v%d.%d.%d (build %d) - %s", product, sdrr_info.major_version, sdrr_info.minor_version, sdrr_info.patch_version, sdrr_info.build_number, project_url);
    LOG("%s %s", copyright, author);
    LOG("Build date: %s", sdrr_info.build_date);
    LOG("Git commit: %s", sdrr_info.commit);

    if (sdrr_info.bootloader_capable) {
        LOG("Bootloader: %s", enabled);
    } else {
        LOG("Bootloader: %s", disabled);
    }

    if (sdrr_info.status_led_enabled) {
        LOG("Status LED: enabled - PB%d", sdrr_info.pins->status);
    } else {
        LOG("Status LED: disabled");
    }

    platform_logging();

#if defined(C_MAIN_LOOP)
    LOG("C main loop: enabled");
#endif // C_MAIN_LOOP

    LOG("%s", log_divider);
    LOG("Pin Configuration ...");
    
    // Port assignments
    const char *port_names[] = {"NONE", "A", "B", "C", "D"};
    
    LOG("ROM emulation: %d pin ROM", sdrr_info.pins->rom_pins);
    
    // Data pins
    LOG("Data pins D[0-7]: P%s%d,%d,%d,%d,%d,%d,%d,%d", 
        port_names[sdrr_info.pins->data_port],
        sdrr_info.pins->data[0], sdrr_info.pins->data[1], sdrr_info.pins->data[2], sdrr_info.pins->data[3],
        sdrr_info.pins->data[4], sdrr_info.pins->data[5], sdrr_info.pins->data[6], sdrr_info.pins->data[7]);
    
    // Address pins
    LOG("Addr pins A[0-15]: P%s%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d", 
        port_names[sdrr_info.pins->addr_port],
        sdrr_info.pins->addr[0], sdrr_info.pins->addr[1], sdrr_info.pins->addr[2], sdrr_info.pins->addr[3],
        sdrr_info.pins->addr[4], sdrr_info.pins->addr[5], sdrr_info.pins->addr[6], sdrr_info.pins->addr[7],
        sdrr_info.pins->addr[8], sdrr_info.pins->addr[9], sdrr_info.pins->addr[10], sdrr_info.pins->addr[11],
        sdrr_info.pins->addr[12], sdrr_info.pins->addr[13], sdrr_info.pins->addr[14], sdrr_info.pins->addr[15]);
    
    // Chip select pins
    LOG("CS pins - 2364: P%s%d 2332: P%s%d,%d 2316: P%s%d,%d,%d X1: P%s%d X2: P%s%d", 
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->cs1_2364,
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->cs1_2332, sdrr_info.pins->cs2_2332,
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->cs1_2316, sdrr_info.pins->cs2_2316, sdrr_info.pins->cs3_2316,
        port_names[sdrr_info.pins->cs_port], sdrr_info.pins->x1, port_names[sdrr_info.pins->cs_port], sdrr_info.pins->x2);
    
    // Select and status pins
    LOG("Sel pins: P%s%d,%d,%d,%d", port_names[sdrr_info.pins->sel_port], 
        sdrr_info.pins->sel[0], sdrr_info.pins->sel[1], 
        sdrr_info.pins->sel[2], sdrr_info.pins->sel[3]);
    LOG("Status pin: P%s%d", port_names[sdrr_info.pins->status_port], sdrr_info.pins->status);

    LOG("%s", log_divider);
    LOG("ROM info ...");
    LOG("# of ROM sets: %d", sdrr_rom_set_count);
    for (uint8_t ii = 0; ii < sdrr_rom_set_count; ii++) {
        LOG("Set #%d: %d ROM(s), size: %d bytes", ii, rom_set[ii].rom_count, rom_set[ii].size);
        
        for (uint8_t jj = 0; jj < rom_set[ii].rom_count; jj++) {
            const char *rom_type_str;
            const sdrr_rom_info_t *rom = rom_set[ii].roms[jj];
            switch (rom->rom_type) {
                case ROM_TYPE_2364:
                    rom_type_str = r2364;
                    break;
                case ROM_TYPE_2332:
                    rom_type_str = r2332;
                    break;
                case ROM_TYPE_2316:
                    rom_type_str = r2316;
                    break;
                default:
                    rom_type_str = unknown;
                    break;
            }

            LOG("  ROM #%d: %s, %s, CS1: %s, CS2: %s, CS3: %s",
                jj, rom->filename,
                rom_type_str,
                cs_values[rom->cs1_state], cs_values[rom->cs2_state], cs_values[rom->cs3_state]);
        }
    }

#if !defined(EXECUTE_FROM_RAM)
    DEBUG("Execute from: %s", flash);
#else // EXECUTE_FROM_RAM
    LOG("Execute from: %s", ram);
#endif // EXECUTE_FROM_RAM

    LOG("%s", log_divider);
    LOG("Running ...");
}
#endif // BOOT_LOGGING

#if defined(BOOT_LOGGING)
// Special version of logging function that remains on flash, and we can get
// a pointer to, to call from within functions (potentially) loaded to RAM.
// Those functions call RAM_LOG(), which only takes a single arg.
void __attribute__((noinline)) do_log(const char* msg, ...) {
    va_list args;
    va_start(args, msg);
    SEGGER_RTT_vprintf(0, msg, &args);
    va_end(args);
    SEGGER_RTT_printf(0, "\n");
}
#endif // BOOT_LOGGING

//
// Functions to handle copying functions to and executing them from RAM
//

#if defined(EXECUTE_FROM_RAM)

// Copies a function from flash to RAM
void copy_func_to_ram(void (*fn)(void), uint32_t ram_addr, size_t size) {
    // Copy the function to RAM
    memcpy((void*)ram_addr, (void*)((uint32_t)fn & ~1), size);
}

void execute_ram_func(uint32_t ram_addr) {
    // Execute the function in RAM
    void (*ram_func)() = (void(*)(void))(ram_addr | 1);
    ram_func();
}

#endif // EXECUTE_FROM_RAM

// Simple delay function
void delay(volatile uint32_t count) {
    while(count--);
}
