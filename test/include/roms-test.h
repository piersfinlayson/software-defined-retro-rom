// Header file for roms-test

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef ROMS_TEST_H
#define ROMS_TEST_H

#define _GNU_SOURCE  // Required for strdup
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <curl/curl.h>
#include <zip.h>

#include "sdrr_config.h"
#include "config_base.h"
#include "roms.h"

#include "json-config.h"

//
// Types
//

typedef struct {
    char *filename;
    char *extract_name;
    char *type;     // "2316", "2332", "2364"
    int cs1;        // 0 or 1
    int cs2;        // 0, 1, or -1 for not specified
    int cs3;        // 0, 1, or -1 for not specified
    int set;        // set number, or -1 for not specified
    int dup;        // 0 - not present, 1 - present
    int pad;        // 0 - not present, 1 - present
} rom_config_t;

typedef struct {
    uint8_t *data;
    size_t size;
    rom_config_t config;
} loaded_rom_t;

typedef struct {
    uint8_t *data;
    size_t size;
    size_t capacity;
} download_buffer_t;

//
// Function prototypes
//

// query-roms.c
extern void create_address_mangler(json_config_t* config);
extern void create_byte_demangler(json_config_t* config);
extern uint16_t create_mangled_address(uint16_t logical_addr, int cs1_active, int x1_active, int x2_active);
extern uint8_t lookup_rom_byte(uint8_t set, uint16_t mangled_addr);
extern uint8_t demangle_byte(uint8_t mangled_byte);
extern const char* rom_type_to_string(int rom_type);
extern const char* cs_state_to_string(int cs_state);
extern size_t get_expected_rom_size(int rom_type);
extern void print_compiled_rom_info(void);

// roms-config.c
extern int parse_rom_configs(const char *configs, rom_config_t **roms);
extern void free_rom_configs(rom_config_t *roms, int count);
extern int load_rom_file(rom_config_t *config, uint8_t **data, size_t *size);
extern int load_all_roms(rom_config_t *configs, int count, loaded_rom_t **loaded_roms);
extern void free_all_roms(loaded_rom_t *loaded_roms, int count);
extern void print_loaded_rom_analysis(loaded_rom_t *loaded_roms, rom_config_t *configs, int count);

// check-roms.c
extern int validate_all_rom_sets(json_config_t *json_config, loaded_rom_t *loaded_roms, rom_config_t *configs, int count);

// json-config.c
extern json_config_t* load_json_config(const char* hw_rev);
extern void free_json_config(json_config_t* config);

#endif // ROMS_TEST_H