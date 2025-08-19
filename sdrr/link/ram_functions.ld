/* Reserve space for RAM functions such as main_loop*/
.ram_func :
{
    . = ALIGN(4);
    _ram_func_start = .;
    . = . + _Ram_Func_Size;
    . = ALIGN(4);
    _ram_func_end = .;
} >RAM

/* Reserve space for data required by RAM functions */
.sdrr_info_ram :
{
    . = ALIGN(4);
    _sdrr_info_ram_start = .;
    . = . + _Sdrr_Info_Ram_Size;
    . = ALIGN(4);
    _sdrr_info_ram_end = .;
}