/* Entry Point */
ENTRY(Reset_Handler)

SECTIONS
{
    /* Flash sections */
    INCLUDE "flash_isr_vector.ld"

    INCLUDE "platform_boot_block.ld"

    INCLUDE "flash_sdrr_info.ld"

    INCLUDE "flash_main_loop.ld"

    INCLUDE "flash_rodata.ld"

    /* RAM sections */

    /* MUST COME FIRST IN RAM */
    INCLUDE "ram_sdrr_runtime_info.ld"

    INCLUDE "ram_rom_image.ld"

    INCLUDE "ram_functions.ld"

    INCLUDE "ram_data.ld"

    INCLUDE "stack.ld"

    /* Remove information from standard libraries */
    /DISCARD/ :
    {
        libc.a ( * )
        libm.a ( * )
        libgcc.a ( * )
    }
}

/* Set initial stack pointer to end of RAM */
_ram_size = LENGTH(RAM);
_estack = ORIGIN(RAM) + LENGTH(RAM);

/* Check enough space for any RAM function */
ASSERT((_main_loop_end - _main_loop_start) <= _Ram_Func_Size, 
    "Error: main_loop section too large for RAM buffer")

/* Check enough space for sdrr_runtime_info structure */
ASSERT(SIZEOF(.sdrr_runtime_info) <= _Sdrr_Runtime_Info_Size, 
    "Error: sdrr_runtime_info structure too large for allocated space")