// Functions to check the compiled ROMs against the original files

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "roms-test.h"

void validate_single_rom_set(loaded_rom_t *loaded_roms, rom_config_t *configs, int count) {
    (void)configs;  // Suppress unused warning
    
    // Verify we have exactly one ROM set with one ROM
    if (sdrr_rom_set_count != 1) {
        printf("Error: Expected 1 ROM set, got %d\n", sdrr_rom_set_count);
        return;
    }
    
    if (rom_set[0].rom_count != 1) {
        printf("Error: Expected 1 ROM in set, got %d\n", rom_set[0].rom_count);
        return;
    }
    
    if (count != 1) {
        printf("Error: Expected 1 loaded ROM, got %d\n", count);
        return;
    }
    
    if (rom_set[0].size != 16384) {
        printf("Error: Expected 16KB ROM set, got %u bytes\n", rom_set[0].size);
        return;
    }
    
    printf("=== Validating Single ROM Set ===\n");
    printf("Original ROM: %zu bytes\n", loaded_roms[0].size);
    printf("Compiled ROM: %u bytes\n", rom_set[0].size);
    
    int errors = 0;
    int total_checked = 0;
    
    // Test every byte in the 16KB compiled ROM
    for (uint16_t logical_addr = 0; logical_addr < 16384; logical_addr++) {
        // Create mangled address with CS1 active (single ROM)
        uint16_t mangled_addr = create_mangled_address(logical_addr, 0, 0, 0);  // CS1=0 (active), X1=0, X2=0 (pulled down)
        
        // Get byte from compiled ROM and demangle it
        uint8_t compiled_byte = lookup_rom_byte(0, mangled_addr);
        uint8_t demangled_byte = demangle_byte(compiled_byte);
        
        // Calculate expected byte from original ROM (with duplication)
        uint16_t original_addr = logical_addr % loaded_roms[0].size;
        uint8_t expected_byte = loaded_roms[0].data[original_addr];
        
        // Compare
        if (demangled_byte != expected_byte) {
            if (errors < 10) {  // Limit error spam
                printf("MISMATCH at logical 0x%04X (mangled 0x%04X): "
                       "expected 0x%02X, got 0x%02X (compiled 0x%02X)\n", 
                       logical_addr, mangled_addr, expected_byte, demangled_byte, compiled_byte);
            }
            errors++;
        }
        
        total_checked++;
    }
    
    printf("Validation complete:\n");
    printf("  Total addresses checked: %d\n", total_checked);
    printf("  Errors found: %d\n", errors);
    
    if (errors == 0) {
        printf("  Result: PASS ✓\n");
    } else {
        printf("  Result: FAIL ✗\n");
    }
}

// Determine which ROM should respond at a given address, or -1 if none
int find_responding_rom(uint16_t address, rom_config_t *configs, int count) {
    // Extract CS line states from address (active low)
    int cs1_active = ((address & (1 << 10)) == 0);
    int x1_active = ((address & (1 << 14)) == 0);
    int x2_active = ((address & (1 << 15)) == 0);
    
    // For single ROM, only CS1 matters
    if (count == 1) {
        return cs1_active ? 0 : -1;
    }
    
    // For multi-ROM sets, check each ROM
    for (int i = 0; i < count; i++) {
        int rom_selected = 0;
        
        // Check which CS line this ROM uses (based on position in set)
        switch (i) {
            case 0: rom_selected = cs1_active; break;
            case 1: rom_selected = x1_active; break;
            case 2: rom_selected = x2_active; break;
            default: continue;
        }
        
        if (rom_selected) {
            // Check CS2/CS3 requirements for this ROM
            rom_config_t *config = &configs[i];
            
            // Check CS2 if specified
            if (config->cs2 != -1 && config->cs2 != 2) {  // Not unspecified and not ignore
                int cs2_bit = (strcmp(config->type, "2332") == 0) ? 9 : 12;  // 2332->bit9, 2316->bit12
                int cs2_active = ((address & (1 << cs2_bit)) == 0);
                
                if ((config->cs2 == 0 && !cs2_active) || (config->cs2 == 1 && cs2_active)) {
                    continue;  // CS2 requirement not met
                }
            }
            
            // Check CS3 if specified (only for 2316)
            if (config->cs3 != -1 && config->cs3 != 2) {  // Not unspecified and not ignore
                if (strcmp(config->type, "2316") == 0) {
                    int cs3_active = ((address & (1 << 9)) == 0);  // CS3 always bit 9 for 2316
                    
                    if ((config->cs3 == 0 && !cs3_active) || (config->cs3 == 1 && cs3_active)) {
                        continue;  // CS3 requirement not met
                    }
                }
            }
            
            return i;  // This ROM responds
        }
    }
    
    return -1;  // No ROM responds
}

// Calculate logical address by removing CS selection bits
uint16_t get_logical_address(uint16_t address) {
    uint16_t logical = address;
    
    // Remove CS selection bits
    logical &= ~(1 << 10);  // Remove CS1
    logical &= ~(1 << 14);  // Remove X1
    logical &= ~(1 << 15);  // Remove X2
    
    // Remove CS2/CS3 bits (simplified - could be more precise based on ROM type)
    logical &= ~(1 << 9);   // Remove CS3/CS2 bit 9
    logical &= ~(1 << 12);  // Remove CS2 bit 12
    
    // Keep only lower 13 bits (8KB max ROM)
    return logical & 0x1FFF;
}

int validate_all_rom_sets(loaded_rom_t *loaded_roms, rom_config_t *configs, int count) {
    printf("\n=== Validating All ROM Sets ===\n");
    
    int total_errors = 0;
    int total_checked = 0;

    int overall_rom_idx = 0;
    
    // Validate each ROM set
    for (int set_idx = 0; set_idx < sdrr_rom_set_count; set_idx++) {
        printf("\nValidating ROM set %d (%d ROMs)...\n", set_idx, rom_set[set_idx].rom_count);
        
        int errors = 0;
        int checked = 0;
        
        uint8_t num_roms = rom_set[set_idx].rom_count;
        if (num_roms == 1) {
            // Single ROM: all CS lines pulled down (0,0,0), test 16KB
            for (uint16_t logical_addr = 0; logical_addr < 16384; logical_addr++) {
                uint16_t mangled_addr = create_mangled_address(logical_addr, 0, 0, 0);
                
                uint8_t compiled_byte = lookup_rom_byte(set_idx, mangled_addr);
                uint8_t demangled_byte = demangle_byte(compiled_byte);
                
                // Find the loaded ROM for this set
                int loaded_rom_idx = overall_rom_idx;
                uint16_t original_addr = logical_addr % loaded_roms[loaded_rom_idx].size;
                uint8_t expected_byte = loaded_roms[loaded_rom_idx].data[original_addr];
                
                if (demangled_byte != expected_byte) {
                    if (errors < 5) {
                        printf("  MISMATCH at logical 0x%04X (mangled 0x%04X): expected 0x%02X, got 0x%02X\n", 
                               logical_addr, mangled_addr, expected_byte, demangled_byte);
                    }
                    errors++;
                }
                checked++;
            }
            overall_rom_idx++;
        } else {
            // Multi-ROM set: test each ROM with appropriate CS combinations  
            for (int rom_idx = 0; rom_idx < rom_set[set_idx].rom_count; rom_idx++) {
                printf("  Testing ROM %d in set %d...\n", rom_idx, set_idx);
                
                // Find corresponding loaded ROM
                int loaded_rom_idx = overall_rom_idx;
                if (loaded_rom_idx >= count) {
                    printf("  Internal error - ran out of ROMs");
                    continue;
                }
                
                // Determine CS values for this ROM
                int cs1, x1, x2;
                if (rom_idx == 0) {
                    // CS1 active, X1 and X2 inactive
                    cs1 = (configs[loaded_rom_idx].cs1 == 0) ? 0 : 1;  
                    x1 = (cs1 == 0) ? 1 : 0;
                    x2 = x1;
                } else if (rom_idx == 1) {
                    // X1 active, CS1 and X2 inactive
                    x1 = (configs[loaded_rom_idx].cs1 == 0) ? 0 : 1;  
                    cs1 = (x1 == 0) ? 1 : 0;
                    x2 = cs1;
                } else { // rom_idx == 2
                    // X2 active, CS1 and X1 inactive
                    assert(rom_idx == 2); // Only max 3 ROMs per set
                    x2 = (configs[loaded_rom_idx].cs1 == 0) ? 0 : 1;
                    cs1 = (x2 == 0) ? 1 : 0;  
                    x1 = cs1;
                }
                
                int rom_errors = 0;
                for (uint16_t logical_addr = 0; logical_addr < loaded_roms[loaded_rom_idx].size; logical_addr++) {
                    uint16_t mangled_addr = create_mangled_address(logical_addr, cs1, x1, x2);
                    
                    uint8_t compiled_byte = lookup_rom_byte(set_idx, mangled_addr);
                    uint8_t demangled_byte = demangle_byte(compiled_byte);
                    
                    uint8_t expected_byte = loaded_roms[loaded_rom_idx].data[logical_addr];
                    
                    if (demangled_byte != expected_byte) {
                        if (rom_errors < 5) {
                            printf("    MISMATCH ROM %d at logical 0x%04X (mangled 0x%04X): expected 0x%02X, got 0x%02X\n", 
                                   rom_idx, logical_addr, mangled_addr, expected_byte, demangled_byte);
                        }
                        rom_errors++;
                    }
                    checked++;
                }

                errors += rom_errors;
                overall_rom_idx++;
            }
        }
        
        printf("Set %d: %d ROMs, %d addresses checked, %d errors\n", set_idx, num_roms, checked, errors);
        total_errors += errors;
        total_checked += checked;
    }
    
    printf("\nOverall validation:\n");
    printf("  Total ROM sets: %d\n", sdrr_rom_set_count);
    printf("  Total ROMs: %d\n", overall_rom_idx);
    printf("  Total addresses checked: %d\n", total_checked);
    printf("  Total errors found: %d\n", total_errors);
    printf("  Result: %s\n", (total_errors == 0) ? "PASS ✓" : "FAIL ✗");

    if (total_errors > 0) {
     return -1;
    } else {
        return 0;
    }
}
