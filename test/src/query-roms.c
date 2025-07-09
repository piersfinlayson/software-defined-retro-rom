// Query generated roms.c

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "roms-test.h"

// lookup_rom_byte - Simulates the lookup of a byte from the ROM image based on the mangled address
uint8_t lookup_rom_byte(uint8_t set, uint16_t mangled_addr) {  // Removed unused CS parameters
    return rom_set[set].data[mangled_addr];
}

// Convert logical address + CS states to mangled address for lookup
uint16_t create_mangled_address(uint16_t logical_addr, int cs1, int x1, int x2) {
    uint16_t mangled = 0;
    
    // Set CS selection bits (active low)
    if (cs1) mangled |= (1 << 10);  // CS1 -> PC10
    if (x1)  mangled |= (1 << 14);  // CX1 -> PC14  
    if (x2)  mangled |= (1 << 15);  // CX2 -> PC15
    
    // Map logical address bits to scrambled GPIO positions
    if (logical_addr & (1 << 0))  mangled |= (1 << 5);   // A0 -> PC5
    if (logical_addr & (1 << 1))  mangled |= (1 << 4);   // A1 -> PC4
    if (logical_addr & (1 << 2))  mangled |= (1 << 6);   // A2 -> PC6
    if (logical_addr & (1 << 3))  mangled |= (1 << 7);   // A3 -> PC7
    if (logical_addr & (1 << 4))  mangled |= (1 << 3);   // A4 -> PC3
    if (logical_addr & (1 << 5))  mangled |= (1 << 2);   // A5 -> PC2
    if (logical_addr & (1 << 6))  mangled |= (1 << 1);   // A6 -> PC1
    if (logical_addr & (1 << 7))  mangled |= (1 << 0);   // A7 -> PC0
    if (logical_addr & (1 << 8))  mangled |= (1 << 8);   // A8 -> PC8
    if (logical_addr & (1 << 9))  mangled |= (1 << 13);  // A9 -> PC13
    if (logical_addr & (1 << 10)) mangled |= (1 << 11);  // A10 -> PC11
    if (logical_addr & (1 << 11)) mangled |= (1 << 12);  // A11 -> PC12
    if (logical_addr & (1 << 12)) mangled |= (1 << 9);   // A12 -> PC9

    return mangled;
}

// Convert mangled byte (as read from GPIO pins) back to logical data
uint8_t demangle_byte(uint8_t mangled_byte) {
    uint8_t logical = 0;
    
    // Reverse the bit order - PA0-PA7 maps to D7-D0
    if (mangled_byte & (1 << 0)) logical |= (1 << 7);  // PA0 -> D7
    if (mangled_byte & (1 << 1)) logical |= (1 << 6);  // PA1 -> D6
    if (mangled_byte & (1 << 2)) logical |= (1 << 5);  // PA2 -> D5
    if (mangled_byte & (1 << 3)) logical |= (1 << 4);  // PA3 -> D4
    if (mangled_byte & (1 << 4)) logical |= (1 << 3);  // PA4 -> D3
    if (mangled_byte & (1 << 5)) logical |= (1 << 2);  // PA5 -> D2
    if (mangled_byte & (1 << 6)) logical |= (1 << 1);  // PA6 -> D1
    if (mangled_byte & (1 << 7)) logical |= (1 << 0);  // PA7 -> D0

    return logical;
}

// Convert ROM type number to string
const char* rom_type_to_string(int rom_type) {
    switch (rom_type) {
        case ROM_TYPE_2316: return "2316";
        case ROM_TYPE_2332: return "2332";  
        case ROM_TYPE_2364: return "2364";
        default: return "unknown";
    }
}

// Convert CS state number to string
const char* cs_state_to_string(int cs_state) {
    switch (cs_state) {
        case CS_ACTIVE_LOW: return "active_low";
        case CS_ACTIVE_HIGH: return "active_high";
        case CS_NOT_USED: return "not_used";
        default: return "unknown";
    }
}

// Get expected ROM size for type
size_t get_expected_rom_size(int rom_type) {
    switch (rom_type) {
        case 0: return 2048;  // 2316 = 2KB
        case 1: return 4096;  // 2332 = 4KB
        case 2: return 8192;  // 2364 = 8KB
        default: return 0;
    }
}

void print_compiled_rom_info(void) {
    printf("\n=== Compiled ROM Sets Analysis ===\n");
    printf("Total ROM images: %d\n", SDRR_NUM_IMAGES);
    printf("Total ROM sets: %d\n", sdrr_rom_set_count);
    
    // Print details for each ROM set
    for (int set_idx = 0; set_idx < sdrr_rom_set_count; set_idx++) {
        printf("\nROM Set %d:\n", set_idx);
        printf("  Size: %u bytes (%s)\n", rom_set[set_idx].size, 
               (rom_set[set_idx].size == 16384) ? "16KB" : 
               (rom_set[set_idx].size == 65536) ? "64KB" : "other");
        printf("  ROM count: %d\n", rom_set[set_idx].rom_count);
        
        // Expected image size based on ROM count
        const char* expected_size = (rom_set[set_idx].rom_count == 1) ? "16KB" : "64KB";
        printf("  Expected size: %s", expected_size);
        if ((rom_set[set_idx].rom_count == 1 && rom_set[set_idx].size == 16384) ||
            (rom_set[set_idx].rom_count > 1 && rom_set[set_idx].size == 65536)) {
            printf(" ✓\n");
        } else {
            printf(" ✗\n");
        }
        
        // Print details for each ROM in this set
        for (int rom_idx = 0; rom_idx < rom_set[set_idx].rom_count; rom_idx++) {
            const sdrr_rom_info_t *rom_info = rom_set[set_idx].roms[rom_idx];
            
            printf("  ROM %d:\n", rom_idx);
#if defined(BOOT_LOGGING)
            printf("    File: %s\n", rom_info->filename);
#endif // BOOT_LOGGING
            printf("    Type: %s (%d)\n", rom_type_to_string(rom_info->rom_type), rom_info->rom_type);
            printf("    CS1: %s (%d)", cs_state_to_string(rom_info->cs1_state), rom_info->cs1_state);
            
            if (rom_info->cs2_state != CS_NOT_USED) {
                printf(", CS2: %s (%d)", cs_state_to_string(rom_info->cs2_state), rom_info->cs2_state);
            }
            if (rom_info->cs3_state != CS_NOT_USED) {
                printf(", CS3: %s (%d)", cs_state_to_string(rom_info->cs3_state), rom_info->cs3_state);
            }
            printf("\n");
            
            // Expected ROM size check
            size_t expected_rom_size = get_expected_rom_size(rom_info->rom_type);
            printf("    Expected ROM size: %zu bytes\n", expected_rom_size);
        }
        
        // Show first 8 bytes of the ROM set data
        printf("  First 8 bytes of mangled set data: ");
        for (size_t j = 0; j < 8 && j < rom_set[set_idx].size; j++) {
            printf("0x%02X ", rom_set[set_idx].data[j]);
        }
        printf("\n");
    }
    
    printf("\n");
}
