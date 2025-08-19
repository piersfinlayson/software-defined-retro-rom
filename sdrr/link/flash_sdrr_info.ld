/* sdrr_info contains firmware configuration info */
.sdrr_info :
{
    . = ORIGIN(FLASH) + _Sdrr_Info_Offset;
    . = ALIGN(4);
    _sdrr_info_start = .;
    KEEP(*(.sdrr_info))
    . = ALIGN(4);
    _sdrr_info_end = .;
} >FLASH