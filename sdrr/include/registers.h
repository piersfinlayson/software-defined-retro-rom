// STM32 registers header file

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef REGISTERS_H
#define REGISTERS_H

#if defined(STM32F4)
#include <reg-stm32f4.h>
#elif defined(RP235X)
#include <reg-rp235x.h>
#else
#error "Unsupported MCU family"
#endif // STM32F4

#endif // REGISTERS_H