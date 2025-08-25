/* General read-only data */
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