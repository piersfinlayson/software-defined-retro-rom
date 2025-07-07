// roms-test main()

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "roms-test.h"

int main(void) {
    printf("Starting ROM image test...\n");

    // Load original ROM configurations
    rom_config_t *configs;
    int count = parse_rom_configs(getenv("ROM_CONFIGS"), &configs);
    if (count < 0) {
        printf("Error parsing ROM configurations\n");
        return -1;
    }
    printf("Parsed %d ROM configurations\n", count);

    // Load original ROM files
    loaded_rom_t *loaded_roms = NULL;
    int rc = load_all_roms(configs, count, &loaded_roms);
    if (rc) {
        printf("Error loading ROMs: %d\n", rc);
        free_rom_configs(configs, count);
        return -1;
    }

    // Print loaded ROM analysis
    print_loaded_rom_analysis(loaded_roms, configs, count);

    // Print compiled ROM information
    print_compiled_rom_info();

    // Validate the loaded ROMs against the compiled sets
    rc = validate_all_rom_sets(loaded_roms, configs, count);

    free_all_roms(loaded_roms, count);
    free_rom_configs(configs, count);

    return rc;
}

