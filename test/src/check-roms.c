// Functions to check the compiled ROMs against the original files

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "roms-test.h"
#include "json-config.h"

int validate_all_rom_sets(json_config_t *json_config, loaded_rom_t *loaded_roms, rom_config_t *configs, int count) {
    printf("\n=== Validating All ROM Sets ===\n");

    assert(json_config != NULL);
    create_address_mangler(json_config);
    create_byte_demangler(json_config);

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
            int loaded_rom_idx = overall_rom_idx;
            printf("- Testing ROM %d in set %d\n  - Type: %s, Name: %s\n", 0, set_idx, configs[overall_rom_idx].type, configs[loaded_rom_idx].filename);

            // Single ROM: test both CS=0 and CS=1 against expected value.
            // 2332/2316 ROMs are tested by virtue of their extra CS line(s)
            // being tested as an address line.
            for (uint16_t logical_addr = 0; logical_addr < 8192; logical_addr++) {
                // Get expected value first
                uint16_t original_addr = logical_addr % loaded_roms[loaded_rom_idx].size;
                uint8_t expected_byte = loaded_roms[loaded_rom_idx].data[original_addr];
                
                // Test CS=0
                uint16_t mangled_addr_cs0 = create_mangled_address(logical_addr, 0, 0, 0);
                uint8_t compiled_byte_cs0 = lookup_rom_byte(set_idx, mangled_addr_cs0);
                uint8_t demangled_byte_cs0 = demangle_byte(compiled_byte_cs0);
                
                if (demangled_byte_cs0 != expected_byte) {
                    if (errors < 5) {
                        printf("    - CS=0 MISMATCH at logical 0x%04X: expected 0x%02X, got 0x%02X\n", 
                            logical_addr, expected_byte, demangled_byte_cs0);
                    }
                    errors++;
                }
                
                // Test CS=1
                uint16_t mangled_addr_cs1 = create_mangled_address(logical_addr, 1, 0, 0);
                uint8_t compiled_byte_cs1 = lookup_rom_byte(set_idx, mangled_addr_cs1);
                uint8_t demangled_byte_cs1 = demangle_byte(compiled_byte_cs1);
                
                if (demangled_byte_cs1 != expected_byte) {
                    if (errors < 5) {
                        printf("    - CS=1 MISMATCH at logical 0x%04X: expected 0x%02X, got 0x%02X\n", 
                            logical_addr, expected_byte, demangled_byte_cs1);
                    }
                    errors++;
                }
                
                checked += 2;
            }
            overall_rom_idx++;
        } else {
            // Multi-ROM set: test all 8 CS combinations
            int cs_combinations[8][3] = {
                {0, 0, 0}, {0, 0, 1}, {0, 1, 0}, {0, 1, 1},
                {1, 0, 0}, {1, 0, 1}, {1, 1, 0}, {1, 1, 1}
            };
            
            for (int combo_idx = 0; combo_idx < 8; combo_idx++) {
                int cs1 = cs_combinations[combo_idx][0];
                int x1 = cs_combinations[combo_idx][1]; 
                int x2 = cs_combinations[combo_idx][2];
                
                // Determine which ROM (if any) this combination should activate
                int active_rom = -1;
                
                // Check if this CS combination matches one of the 3 active patterns
                for (int rom_idx = 0; rom_idx < rom_set[set_idx].rom_count; rom_idx++) {
                    int loaded_rom_idx = overall_rom_idx + rom_idx;
                    if (loaded_rom_idx >= count) {
                        printf("  Internal error - ran out of ROMs");
                        continue;
                    }
                    
                    int expected_active = (configs[loaded_rom_idx].cs1 == 0) ? 0 : 1;
                    int expected_inactive = (expected_active == 0) ? 1 : 0;
                    
                    if ((rom_idx == 0 && cs1 == expected_active && x1 == expected_inactive && x2 == expected_inactive) ||
                        (rom_idx == 1 && x1 == expected_active && cs1 == expected_inactive && x2 == expected_inactive) ||
                        (rom_idx == 2 && x2 == expected_active && cs1 == expected_inactive && x1 == expected_inactive)) {
                        active_rom = rom_idx;
                        break;
                    }
                }
                
                // Print header for this combination
                if (active_rom >= 0) {
                    int loaded_rom_idx = overall_rom_idx + active_rom;
                    if (loaded_rom_idx < count) {
                        printf("- ROM %d in set %d - CS1=%d, X1=%d, X2=%d", active_rom, set_idx, cs1, x1, x2);
                        printf("  - Type: %s, Name: %s\n", configs[loaded_rom_idx].type, configs[loaded_rom_idx].filename);
                    } else {
                        printf("- ROM %d in set %d (ERROR: out of bounds)\n", active_rom, set_idx);
                    }
                } else {
                    printf("- Testing blank section CS1=%d, X1=%d, X2=%d\n", cs1, x1, x2);
                }
                
                // Test all addresses for this combination
                int combo_errors = 0;
                for (uint16_t logical_addr = 0; logical_addr < 8192; logical_addr++) {
                    uint16_t mangled_addr = create_mangled_address(logical_addr, cs1, x1, x2);
                    uint8_t compiled_byte = lookup_rom_byte(set_idx, mangled_addr);
                    uint8_t demangled_byte = demangle_byte(compiled_byte);
                    
                    uint8_t expected;
                    if (active_rom >= 0) {
                        int loaded_rom_idx = overall_rom_idx + active_rom;
                        if (loaded_rom_idx >= count) {
                            expected = 0xAA;
                        } else {
                            uint16_t rom_addr = logical_addr % loaded_roms[loaded_rom_idx].size;
                            expected = loaded_roms[loaded_rom_idx].data[rom_addr];
                        }
                    } else {
                        expected = 0xAA;
                    }
                    
                    if (demangled_byte != expected) {
                        if (combo_errors < 5) {
                            printf("    - MISMATCH at logical 0x%04X: expected 0x%02X, got 0x%02X\n", 
                                logical_addr, expected, demangled_byte);
                        }
                        combo_errors++;
                    }
                    checked++;
                }
                errors += combo_errors;
            }
            
            overall_rom_idx += rom_set[set_idx].rom_count;
        }
        
        char *roms;
        if (num_roms > 1) {
            roms = "ROMs";
        } else {
            roms = "ROM";
        }
        printf("- Result: Set %d: %d %s, %d addresses checked, %d errors\n", set_idx, num_roms, roms, checked, errors);
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
