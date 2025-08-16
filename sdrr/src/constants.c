// Contains constants

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#include "include.h"

// Metadata strings to include in the binary
const char mcu_variant[] = MCU_VARIANT;
const char license[] = "MIT License";
const char project_url[] = "https://piers.rocks/u/one";
const char copyright[] = "Copyright (c) 2025";
const char author[] = "Piers Finlayson <piers@piers.rocks>";
const char product[] = "One ROM";
const char log_divider[] = "-----";
const char inverted[] = "~";
const char r2364[] = "2364";
const char r2332[] = "2332";
const char r2316[] = "2316";
const char unknown[] = "unknown";
const char cs_low[] = "0";
const char cs_high[] = "1";
const char cs_na[] = "-";
const char flash[] = "Flash";
const char ram[] = "RAM";
const char rom_type[] = "ROM type";
const char stm32_bootloader_mode[] = "STM32 bootloader mode";
const char disabled[] = "disabled";
const char enabled[] = "enabled";
const char oscillator[] = "Oscillator";
#if defined(BOOT_LOGGING)
const char *port_names[] = {"NONE", "A", "B", "C", "D"};
const char *cs_values[] = {"Active Low", "Active High", "-"};
#endif // BOOT_LOGGING
