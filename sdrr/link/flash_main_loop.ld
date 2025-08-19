/* main_loop() has its own section, so it can be copied to RAM, if so */
/* configured */
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
