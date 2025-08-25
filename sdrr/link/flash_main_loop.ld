/* main_loop() has its own section, so it can be copied to RAM, if so */
/* configured.  We use 8 byte alignment in case we need to do XIP cache */
/* pinning on the RP2350 */
.text :
{
    . = ALIGN(8);
    _text_start = .;

    /* Mark the beginning and end of the main loop so it can be copied */
    /* to RAM at runtime */
    . = ALIGN(8);
    _main_loop_start = .;
    KEEP(*(.main_loop));
    . = ALIGN(8);
    _main_loop_end = .;

    *(.text)
    *(.text*)

    . = ALIGN(8);
    _text_end = .;
} >FLASH
