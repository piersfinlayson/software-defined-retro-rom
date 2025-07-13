# Logging

There are three types of logging which can be enabled on SDRR.

## `BOOT_LOGGING`

This logs the boot process to the SWD interface, using RTT.  It is enabled by default.  This type of logging stops when the device enters its main ROM serving loop, as with logging enabled the device cannot hit the required performance.

In general, you should be able to leave `BOOT_LOGGING` on, as this only adds around 1.5ms to SDRR's startup time, increasing it to roughly 3ms total.  This is substantially below most retro systems' reset circuit timers, so SDRR should boot and be ready to serve the configured ROM image well before it is required.

Sample boot logs from a startup of firmware built with [`c64.mk`](/config/c64.mk), are shown below:

```log
14:44:53.730: -----
14:44:53.730: SDRR v0.1.0 - https://piers.rocks/u/sdrr
14:44:53.730: Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
14:44:53.730: Build date: Jun 28 2025 14:44:47
14:44:53.730: -----
14:44:53.730: Hardware info ...
14:44:53.730: STM32F411RE
14:44:53.730: PCB rev D
14:44:53.730: Flash size: 512KB
14:44:53.730: Flash used: 95KB
14:44:53.730: RAM: 128KB
14:44:53.730: Target freq: 100MHz
14:44:53.730: Oscillator: HSI
14:44:53.730: PLL MNPQ: 8/100/0/4
14:44:53.730: MCO: disabled
14:44:53.730: STM32 bootloader mode disabled
14:44:53.730: -----
14:44:53.730: Firmware info ...
14:44:53.730: # of ROM images: 5
14:44:53.730: #0: kernal.901227-03.bin, 2364, CS1: 0, CS2: -, CS3: -
14:44:53.730: #1: basic.901226-01.bin, 2364, CS1: 0, CS2: -, CS3: -
14:44:53.730: #2: characters.901225-01.bin, 2332, CS1: 0, CS2: 1, CS3: -
14:44:53.730: #3: dead%20test.BIN, 2364, CS1: 0, CS2: -, CS3: -
14:44:53.730: #4: destest-max.rom, 2364, CS1: 0, CS2: -, CS3: -
14:44:53.730: -----
14:44:53.730: Running ...
14:44:53.730: Set VOS to scale 1
14:44:53.730: Configured PLL MNPQ: 8/100/0/4
14:44:53.730: Set flash config: 3 ws
14:44:53.730: ROM sel/index 3/3
14:44:53.730: ROM dead%20test.BIN preloaded to RAM 0x20000000
14:44:53.730: Start main loop - logging ends
```

Logging from v0.2.0 onwards is even more extensive.

Pulling out some highlights:

- The `Hardare info` section logs provide information about the SDRR hardware that this firmware image **was built for**.  This may not necessarily the hardware you are running the firmware on.
- The `Firmware info` section logs show you how many ROM images are included in the firmare, and some details about them:
  - their file names (as seen by the build process)
  - the ROM type (`2364`, `2332`, `2316`)
  - the chip select line configuration:
    - `0` means active low
    - `1` means active high
    - `-` means not used for this ROM type
- The `Running ...` section logs show SDRR's main activities as it starts up, including:
  - Whether the VOS (voltage scaling) is set to scale 1 (only done on the F411m and only when `FREQ` is > 84Mhz).  While the F405 also required VOS set to scale 1 for high frequency operation, this is its power-on default.
  - What the PLL MNPQ values have been set to (to allow you to check they are as intended to achieve the target `FREQ`).
  - How many flash wait states have been configured (the STM32 required a different number of wait states based on the `FREQ`).
  - `ROM sel/index` shows you what value the image select jumpers were set to, and what index that corresponds to in the firmware.
  - The logs also show the filename of the active ROM image, and whether it has been preloaded to RAM (the default behaviour).
  - Finally, the last line shows that the main loop has started, and that logging has stopped.

## `DEBUG_LOGGING`

This logs extra debug information to the SWD interface, using RTT.  Enabling debug logging can sometimes cause RTT to lose some logs due to its added verbosity - this is typically shown as blank logs.

It is disabled by default, but can be enabled by setting the `DEBUG_LOGGING` configuration option to `1`.  This type of logging is useful for debugging SDRR itself.

## `MAIN_LOOP_LOGGING`

This enables logging within the main_loop (see [`rom_impl.c`](/sdrr/src/rom_impl.c).  It does not control logging after every byte is served - see [`MAIN_LOOP_ONE_SHOT`](#main-loop-one-shot)

You mus have `SWD` and `BOOT_LOGGING` enabled for this to work.

## `MAIN_LOOP_ONE_SHOT`

This makes a log after every requested byte has been served, once the chip select has gone inactive.

Some notes:

- It is **highly** likely that enabling this will cause the SDRR to not be able to keep up with the requested data, and so it is disabled by default.
- It is also likely, unless you are only querying the odd byte here and there, that RTT will not be able to keep up with the logged data.
- Although the address and data values served are logged, these are **mangled** versions of each - they are the value read directly from the STM32 GPIO port with the address and CS lines, and the value written directly to the STM32 GPIO port with the data lines.  As the pin mapping is complex, they are not the values you would naively expect to see.  See [Technical Details](/docs/TECHNICAL-DETAILS.md) for more information on the the pin mapping.

This logging is disabled by default and can be enabled by setting the `MAIN_LOOP_ONE_SHOT` configuration option to `1`.  This type of logging is useful for debugging the SDRR's main loop.

You must have `SWD`, `BOOT_LOGGING`, and `MAIN_LOOP_LOGGING` enabled for this to work.
