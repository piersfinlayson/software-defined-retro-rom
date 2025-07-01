/* Linker script for 64KB flash version of the STM32F103 microcontroller */
MEMORY
{
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 64K
    RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 20K
}

INCLUDE stm32f-common.ld