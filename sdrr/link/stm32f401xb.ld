/* Linker script for STM32F401xE microcontroller */
MEMORY
{
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 128K
    RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 96K
}

INCLUDE stm32f-common.ld