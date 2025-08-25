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