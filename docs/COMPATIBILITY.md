# Compatibility

One ROM is designed to replace original 2364, 2332 and 2316 ROMs from retro computers, disk drivs, etc.  It should also be compatible with 2564, 2532 and 2516 EPROMs.

One ROM running on an STM32F401 at 84Mhz is sufficient to emulate ROMs in a C64 system.  Faster STM32F4s (such as the F405/446 - 168Mhz/180Mz) may be required for faster systems.  See [STM32 Selection](docs/STM32-SELECTION.md) for more information on selecting and STM32F4 variant.

## Tested Compatible Systems

One ROM is know to have been successfully tested with the following systems:

| System Type | System | Variants | ROM Types |
|-------------|--------|----------|-----------|
| Computer    | C64    | PAL | Kernal/BASIC/Character/Diagnostics |
| Computer    | VIC-20 | PAL+NTSC | Kernal/BASIC/Character/Diagnostics |
| Computer    | PET    | 40xx/40 col/50Hz | Kernal/BASIC/Character/Diagnostics |
| Computer    | TI-99  | 4A | System |
| Computer    | CoCo 2  | n/a | BASIC |
| Disk Drive  | 1541   | n/a | Kernal |
| Disk Drive  | 2031   | n/a | Kernal |

## Tested Incompatible Systems

One ROM is not yet known to have failed to work on any systems which took original 2364, 2332 or 2316 ROMs.  As and when incompatible systems are found, they will be added below.