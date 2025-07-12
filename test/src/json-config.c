// Functions for JSON hardware configuration

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ctype.h>
#include <json-c/json.h>
#include <json-config.h>

#define HW_CONFIG_DIR  "sdrr-hw-config"
static const char* base_dirs[] = {HW_CONFIG_DIR "/", "../" HW_CONFIG_DIR "/"};
static const char* sub_dirs[] = {"third-party/", "user/"};

static char* normalize_filename(const char* hw_rev) {
    size_t len = strlen(hw_rev);
    char* normalized = malloc(len + 6);

    if (!strcmp(hw_rev, "d")) {
        strcpy(normalized, "24-d.json");
        return normalized;
    } else if (!strcmp(hw_rev, "e")) {
        strcpy(normalized, "24-e.json");
        return normalized;
    } else if (!strcmp(hw_rev, "f")) {
        strcpy(normalized, "24-f.json");
        return normalized;
    }
    
    for (size_t i = 0; i < len; i++) {
        if (hw_rev[i] == '_') {
            normalized[i] = '-';
        } else {
            normalized[i] = tolower(hw_rev[i]);
        }
    }
    strcpy(normalized + len, ".json");
    return normalized;
}

static sdrr_stm_port_t parse_port(const char* port_str) {
    if (!port_str) return PORT_NONE;
    switch (port_str[0]) {
        case 'A': return PORT_A;
        case 'B': return PORT_B;
        case 'C': return PORT_C;
        case 'D': return PORT_D;
        default: return PORT_NONE;
    }
}

static void parse_int_array(json_object* arr, uint8_t* dest, size_t max_size) {
    size_t len = json_object_array_length(arr);
    for (size_t i = 0; i < len && i < max_size; i++) {
        json_object* item = json_object_array_get_idx(arr, i);
        dest[i] = json_object_get_int(item);
    }
}

static void parse_cs_config(json_object* cs_obj, cs_config_t* cs_config) {
    cs_config->pin_2364 = 255;
    cs_config->pin_2332 = 255;
    cs_config->pin_2316 = 255;
    
    json_object* pin_obj;
    if (json_object_object_get_ex(cs_obj, "2364", &pin_obj)) {
        cs_config->pin_2364 = json_object_get_int(pin_obj);
    }
    if (json_object_object_get_ex(cs_obj, "2332", &pin_obj)) {
        cs_config->pin_2332 = json_object_get_int(pin_obj);
    }
    if (json_object_object_get_ex(cs_obj, "2316", &pin_obj)) {
        cs_config->pin_2316 = json_object_get_int(pin_obj);
    }
}

static FILE* find_config_file(const char* filename) {
    char filepath[512];
    FILE* fp;
    
    // First try base directories
    snprintf(filepath, sizeof(filepath), HW_CONFIG_DIR "/%s", filename);
    fp = fopen(filepath, "r");
    if (fp) return fp;

    snprintf(filepath, sizeof(filepath), "../" HW_CONFIG_DIR "/%s", filename);
    fp = fopen(filepath, "r");
    if (fp) return fp;
    
    // If not found, try subdirectories under whichever base exists    
    for (size_t i = 0; i < 2; i++) {
        for (size_t j = 0; j < 2; j++) {
            snprintf(filepath, sizeof(filepath), "%s%s%s", base_dirs[i], sub_dirs[j], filename);
            fp = fopen(filepath, "r");
            if (fp) return fp;
        }
    }
    
    return NULL;
}

json_config_t* load_json_config(const char* hw_rev) {
    if (!hw_rev) {
        printf("No HW_REV supplied\n");
        return NULL;
    }
    
    char* filename = normalize_filename(hw_rev);
    FILE* fp = find_config_file(filename);
    
    if (!fp) {
        printf("Couldn't find hardware configuration file %s\n", filename);
        free(filename);
        return NULL;
    }
    free(filename);

    fseek(fp, 0, SEEK_END);
    long len = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    
    char* data = malloc(len + 1);
    fread(data, 1, len, fp);
    data[len] = '\0';
    fclose(fp);
    
    json_object* root = json_tokener_parse(data);
    free(data);
    if (!root) {
        printf("Failed to parse JSON configuration\n");
        return NULL;
    }

    json_config_t* config = calloc(1, sizeof(json_config_t));
    
    json_object* desc_obj;
    if (json_object_object_get_ex(root, "description", &desc_obj)) {
        config->description = strdup(json_object_get_string(desc_obj));
    }
    
    json_object* rom_obj;
    if (json_object_object_get_ex(root, "rom", &rom_obj)) {
        json_object* pins_obj;
        if (json_object_object_get_ex(rom_obj, "pins", &pins_obj)) {
            json_object* quantity_obj;
            if (json_object_object_get_ex(pins_obj, "quantity", &quantity_obj)) {
                config->rom.pin_count = json_object_get_int(quantity_obj);
            }
        }
    }
    
    json_object* stm_obj;
    if (json_object_object_get_ex(root, "stm", &stm_obj)) {
        json_object* family_obj;
        if (json_object_object_get_ex(stm_obj, "family", &family_obj)) {
            strncpy(config->stm.family, json_object_get_string(family_obj), MAX_FAMILY_LEN - 1);
        }
        
        json_object* ports_obj;
        if (json_object_object_get_ex(stm_obj, "ports", &ports_obj)) {
            json_object* port_obj;
            if (json_object_object_get_ex(ports_obj, "data_port", &port_obj)) {
                config->stm.ports.data_port = parse_port(json_object_get_string(port_obj));
            }
            if (json_object_object_get_ex(ports_obj, "addr_port", &port_obj)) {
                config->stm.ports.addr_port = parse_port(json_object_get_string(port_obj));
            }
            if (json_object_object_get_ex(ports_obj, "cs_port", &port_obj)) {
                config->stm.ports.cs_port = parse_port(json_object_get_string(port_obj));
            }
            if (json_object_object_get_ex(ports_obj, "sel_port", &port_obj)) {
                config->stm.ports.sel_port = parse_port(json_object_get_string(port_obj));
            }
            if (json_object_object_get_ex(ports_obj, "status_port", &port_obj)) {
                config->stm.ports.status_port = parse_port(json_object_get_string(port_obj));
            }
        }
        
        json_object* pins_obj;
        if (json_object_object_get_ex(stm_obj, "pins", &pins_obj)) {
            json_object* arr_obj;
            if (json_object_object_get_ex(pins_obj, "data", &arr_obj)) {
                parse_int_array(arr_obj, config->stm.pins.data, NUM_DATA_LINES);
            }
            if (json_object_object_get_ex(pins_obj, "addr", &arr_obj)) {
                parse_int_array(arr_obj, config->stm.pins.addr, MAX_ADDR_LINES);
            }
            if (json_object_object_get_ex(pins_obj, "sel", &arr_obj)) {
                parse_int_array(arr_obj, config->stm.pins.sel, 4);
            }
            
            json_object* cs_obj;
            if (json_object_object_get_ex(pins_obj, "cs1", &cs_obj)) {
                parse_cs_config(cs_obj, &config->stm.pins.cs1);
            }
            if (json_object_object_get_ex(pins_obj, "cs2", &cs_obj)) {
                parse_cs_config(cs_obj, &config->stm.pins.cs2);
            }
            if (json_object_object_get_ex(pins_obj, "cs3", &cs_obj)) {
                parse_cs_config(cs_obj, &config->stm.pins.cs3);
            }
            
            json_object* pin_obj;
            if (json_object_object_get_ex(pins_obj, "x1", &pin_obj)) {
                config->stm.pins.x1 = json_object_get_int(pin_obj);
            }
            if (json_object_object_get_ex(pins_obj, "x2", &pin_obj)) {
                config->stm.pins.x2 = json_object_get_int(pin_obj);
            }
            if (json_object_object_get_ex(pins_obj, "status", &pin_obj)) {
                config->stm.pins.status = json_object_get_int(pin_obj);
            }
        }
    }
    
    json_object_put(root);
    return config;
}

void free_json_config(json_config_t* config) {
    if (config) {
        free(config->description);
        free(config);
    }
}