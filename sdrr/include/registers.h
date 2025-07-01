// STM32 registers header file

// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

#ifndef REGISTERS_H
#define REGISTERS_H

#if defined(STM32F1)
#include <reg-stm32f1.h>
#elif defined(STM32F4)
#include <reg-stm32f4.h>
#else
#error "Unsupported STM32 family"
#endif // STM32F1/STM32F4

#endif // REGISTERS_H