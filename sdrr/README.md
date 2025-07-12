# SDRR Firmware

This directory and its subdirectories contain the source code for the SDRR firmware.

To build and flash the firmware, use the top-level Makefile in the SDRR repository.  For example:

```bash
STM=f411re CONFIG=config/c64.mk make run
```

If you would like to understand the build system, see [Build System](/docs/BUILD-SYSTEM.md).
