/* Reserve space for the sdrr_runtime_info structure at start of RAM */
_sdrr_runtime_info_ram = LOADADDR(.sdrr_runtime_info);
.sdrr_runtime_info :
{
    . = ALIGN(4);
    _sdrr_runtime_info_start = .;
    *(.sdrr_runtime_info)
    *(.sdrr_runtime_info*)
    . = _sdrr_runtime_info_start + _Sdrr_Runtime_Info_Size;
    . = ALIGN(4);
    _sdrr_runtime_info_end = .;
} >RAM AT> FLASH
