// Data structures for JSON hardware configuration

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef JSON_CONFIG_H
#define JSON_CONFIG_H

#include <stdint.h>
#include "config_base.h"

#define NUM_DATA_LINES 8
#define MAX_ADDR_LINES 16
#define MAX_FAMILY_LEN 16

typedef struct {
    uint8_t pin_count;
} json_rom_config_t;

typedef struct {
    sdrr_stm_port_t data_port;
    sdrr_stm_port_t addr_port;
    sdrr_stm_port_t cs_port;
    sdrr_stm_port_t sel_port;
    sdrr_stm_port_t status_port;
} stm_ports_t;

typedef struct {
    uint8_t pin_2364;
    uint8_t pin_2332;
    uint8_t pin_2316;
} cs_config_t;

typedef struct {
    uint8_t data[NUM_DATA_LINES];
    uint8_t addr[MAX_ADDR_LINES];
    cs_config_t cs1;
    cs_config_t cs2;
    cs_config_t cs3;
    uint8_t x1;
    uint8_t x2;
    uint8_t sel[4];
    uint8_t status;
} stm_pins_t;

typedef struct {
    char family[MAX_FAMILY_LEN];
    stm_ports_t ports;
    stm_pins_t pins;
} stm_config_t;

typedef struct {
    char *description;
    json_rom_config_t rom;
    stm_config_t stm;
} json_config_t;

#endif // JSON_CONFIG_H
