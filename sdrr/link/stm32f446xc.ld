/* Linker script for STM32F446xC microcontroller */
MEMORY
{
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 256K
    RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 128K
}

INCLUDE stm32f-common.ld