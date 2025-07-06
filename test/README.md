# roms-test

This is a test suite for the `roms.c` output from `sdrr-gen`, verifying that the generated images and data are correct - that is, they are properly mangled in the way that the STM32F4 expects (based on the STM32F4 wiring to the address, data and CS lines).

The best way to execute this test suite is to run `make test` from the root directory of the repository.  Pass in whatever environment variabls you are using to build your firmware.  For example:

```bash
HW_REV=f CONFIG=config/set-c64.mk make test
```

This will build the firmware, then build the test suite with the same auto-generated `roms.c` file, and then test that against the original ROM files.

These tests have an external dependency on `libcurl` which is used to retrieve any ROM files specified with URLs (as opposed to local files).

```bash
sudo apt install libcurl4-openssl-dev
```
