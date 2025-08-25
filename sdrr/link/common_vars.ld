/* Stack size */
_Min_Stack_Size = 0x400;    /* 1KB */

/* Space for storing functions in RAM */
_Ram_Func_Size = 0x1000; /* 4KB */

/* Space for sdrr_raminfo_t in RAM */
_Sdrr_Runtime_Info_Size = 0x100; /* 256 bytes */

/* Space for sdrr_info_t and other stuff required by main_loop when running */
/* from RAM */
_Sdrr_Info_Ram_Size = 0x100; /* 256 bytes */ 

/* Offset from start of RAM for sdrr_info_t */
_Sdrr_Info_Offset = 0x200;
