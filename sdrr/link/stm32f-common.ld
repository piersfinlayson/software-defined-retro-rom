/* Entry Point */
ENTRY(Reset_Handler)

/* Stack size */
_Min_Stack_Size = 0x400;    /* 1KB */

/* Space for storing functions in RAM */
_Ram_Func_Size = 0x800; /* 1KB */

SECTIONS
{
    /* Vector table and code must be in FLASH */
    .isr_vector :
    {
        . = ALIGN(4);
        _flash_start = .;
        _isr_vector_start = .;
        KEEP(*(.isr_vector))
        . = ALIGN(4);
        _isr_vector_end = .;
    } >FLASH

    /* Start at 0x200 to give some space after the vector table, in case */
    /* it's bigger than planned on a specific variant. */
    .sdrr_info :
    {
        . = ORIGIN(FLASH) + 0x200;
        . = ALIGN(4);
        _sdrr_info_start = .;
        KEEP(*(.sdrr_info))
        . = ALIGN(4);
        _sdrr_info_end = .;
    } >FLASH

    .text :
    {
        . = ALIGN(4);
        _text_start = .;
        *(.text)
        *(.text*)

        /* Mark the beginning and end of the main loop so it can be copied */
        /* to RAM at runtime */
        . = ALIGN(4);
        _main_loop_start = .;
        KEEP(*(.main_loop));
        . = ALIGN(4);
        _main_loop_end = .;

        . = ALIGN(4);
        _text_end = .;
    } >FLASH

    .rodata :
    {
        . = ALIGN(4);
        _rodata_start = .;
        KEEP(*(.rodata));
        KEEP(*(.rodata*));
        . = ALIGN(4);
        _rodata_end = .;
        _flash_end = .;
    } >FLASH

    /* Reserve space at start of RAM for ROM image */
    .ram_rom_image :
    {
        . = ALIGN(4);
        _ram_rom_image_start = .;
        . = . + 0x10000;  /* 64KB for max 4 16KB images for a set */
        . = ALIGN(4);
        _ram_rom_image_end = .;
    } >RAM

    /* Reserve space at start of RAM for RAM functions */
    .ram_func :
    {
        . = ALIGN(4);
        _ram_func_start = .;
        . = . + _Ram_Func_Size;
        . = ALIGN(4);
        _ram_func_end = .;
    } >RAM

    /* Data to be copied from flash to RAM */
    _sidata = LOADADDR(.data);

    .data :
    {
        . = ALIGN(4);
        _sdata = .;
        *(.data)
        *(.data*)
        . = ALIGN(4);
        _edata = .;
    } >RAM AT> FLASH

    /* Uninitialized data */
    .bss :
    {
        . = ALIGN(4);
        _sbss = .;
        *(.bss)
        *(.bss*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    } >RAM

    /* Reserve stack space */
    ._user_heap_stack :
    {
        . = ALIGN(8);
        . = . + _Min_Stack_Size;
        . = ALIGN(8);
    } >RAM

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
