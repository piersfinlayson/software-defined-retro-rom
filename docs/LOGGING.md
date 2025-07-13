# Logging

There are three types of logging which can be enabled on SDRR.

## `BOOT_LOGGING`

This logs the boot process to the SWD interface, using RTT.  It is enabled by default.  This type of logging stops when the device enters its main ROM serving loop, as with logging enabled the device cannot hit the required performance.

In general, you should be able to leave `BOOT_LOGGING` on, as this only adds around 1.5ms to SDRR's startup time, increasing it to roughly 3ms total.  This is substantially below most retro systems' reset circuit timers, so SDRR should boot and be ready to serve the configured ROM image well before it is required.

Sample boot logs from a startup of firmware built with [`c64-no-destestmax.mk`](/config/c64-no-destestmax.mk), are shown below:

```log
13:49:28.943: -----
13:49:28.943: SDRR v0.2.0 (build 1) - https://piers.rocks/u/sdrr
13:49:28.943: Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
13:49:28.943: Build date: Jul 13 2025 13:49:23
13:49:28.943: Git commit: bed08d2
13:49:28.943: -----
13:49:28.943: Hardware info ...
13:49:28.943: STM32F411RE
13:49:28.943: PCB rev 24-d
13:49:28.943: Flash size: 512KB
13:49:28.943: Flash used: 82KB
13:49:28.943: RAM: 128KB
13:49:28.943: Target freq: 100MHz
13:49:28.943: Oscillator: HSI
13:49:28.943: PLL MNPQ: 8/100/0/4
13:49:28.943: MCO: disabled
13:49:28.943: Bootloader: disabled
13:49:28.943: -----
13:49:28.943: Pin Configuration ...
13:49:28.943: ROM emulation: 24 pin ROM
13:49:28.943: Data pins D[0-7]: PA7,6,5,4,3,2,1,0
13:49:28.943: Addr pins A[0-15]: PC5,4,6,7,3,2,1,0,8,13,11,12,9,255,255,255
13:49:28.943: CS pins - 2364: PC10 2332: PC10,9 2316: PC10,12,9 X1: PC255 X2: PC255
13:49:28.943: Sel pins: PB0,1,2,255
13:49:28.943: Status pin: PNONE255
13:49:28.943: -----
13:49:28.943: ROM info ...
13:49:28.943: # of ROM sets: 4
13:49:28.943: Set #0: 1 ROM(s), size: 16384 bytes
13:49:28.943:   ROM #0: kernal.901227-03.bin, 2364, CS1: Active Low, CS2: -, CS3: -
13:49:28.943: Set #1: 1 ROM(s), size: 16384 bytes
13:49:28.943:   ROM #0: basic.901226-01.bin, 2364, CS1: Active Low, CS2: -, CS3: -
13:49:28.943: Set #2: 1 ROM(s), size: 16384 bytes
13:49:28.943:   ROM #0: characters.901225-01.bin, 2332, CS1: Active Low, CS2: Active High, CS3: -
13:49:28.943: Set #3: 1 ROM(s), size: 16384 bytes
13:49:28.943:   ROM #0: dead%20test.BIN, 2364, CS1: Active Low, CS2: -, CS3: -
13:49:28.943: -----
13:49:28.943: Running ...
13:49:28.943: !!! VOS not ready - proceeding anyway
13:49:28.943: Set VOS to scale 1
13:49:28.943: Configured PLL MNPQ: 8/100/0/4
13:49:28.943: Set flash config: 3 ws
13:49:28.943: ROM sel/index 0/0
13:49:28.943: ROM kernal.901227-03.bin preloaded to RAM 0x20000000 size 16384 bytes
13:49:28.943: Set ROM count: 1, Serving algorithm: 0, multi-ROM CS1 state: -
13:49:28.943: Start main loop - logging ends
```

Pulling out some highlights:

- The `Hardare info` section logs provide information about the SDRR hardware that this firmware image **was built for**.  This may not necessarily the hardware you are running the firmware on.
- The `Pin Configuration` section logs show the pin mapping used by the firmware, including:
  - The data pins (D[0-7])
  - The address pins (A[0-15])
  - The chip select (CS) pins for each ROM type
  - The image select pins (SEL0, SEL1, SEL2)
  - The status LED pin (if supported)
- The `ROM info` section logs show you how many ROM images are included in the firmare, and some details about them:
  - their file names (as seen by the build process)
  - the ROM type (`2364`, `2332`, `2316`)
  - the chip select line configuration:
    - `0` means active low
    - `1` means active high
    - `-` means not used for this ROM type
- The `Running ...` section logs show SDRR's main activities as it starts up, including:
  - Whether the VOS (voltage scaling) is set to scale 1 (only done on the F411 and only when `FREQ` is > 84Mhz).  While the F405 also required VOS set to scale 1 for high frequency operation, this is its power-on default.
  - What the PLL MNPQ values have been set to (to allow you to check they are as intended to achieve the target `FREQ`).
  - How many flash wait states have been configured (the STM32 required a different number of wait states based on the `FREQ`).
  - `ROM sel/index` shows you what value the image select jumpers were set to, and what index that corresponds to in the firmware.
  - The logs also show the filename of the active ROM image, and whether it has been preloaded to RAM (the default behaviour).
  - Finally, the last line shows that the main loop has started, and that logging has stopped.

## `DEBUG_LOGGING`

This logs extra debug information to the SWD interface, using RTT.  Enabling debug logging can sometimes cause RTT to lose some logs due to its added verbosity - this is typically shown as blank logs.  However, the RTT buffer has been increased in size, so this should not be a problem in most cases.

It is disabled by default, but can be enabled by setting the `DEBUG_LOGGING` configuration option to `1`.  This type of logging is useful for debugging SDRR itself.

Example debug logging:

```log
13:50:58.502: -----
13:50:58.502: SDRR v0.2.0 (build 1) - https://piers.rocks/u/sdrr
13:50:58.502: Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
13:50:58.502: Build date: Jul 13 2025 13:50:51
13:50:58.502: Git commit: bed08d2
13:50:58.502: -----
13:50:58.502: Hardware info ...
13:50:58.502: STM32F411RE
13:50:58.502: PCB rev 24-d
13:50:58.502: Flash size: 512KB (524288 bytes)
13:50:58.502: Flash used: 84KB 85112 bytes
13:50:58.502: RAM: 128KB (131072 bytes)
13:50:58.502: Target freq: 100MHz
13:50:58.502: Oscillator: HSI
13:50:58.502: PLL MNPQ: 8/100/0/4
13:50:58.502: MCO: disabled
13:50:58.502: Bootloader: disabled
13:50:58.502: -----
13:50:58.502: Pin Configuration ...
13:50:58.502: ROM emulation: 24 pin ROM
13:50:58.502: Data pins D[0-7]: PA7,6,5,4,3,2,1,0
13:50:58.502: Addr pins A[0-15]: PC5,4,6,7,3,2,1,0,8,13,11,12,9,255,255,255
13:50:58.502: CS pins - 2364: PC10 2332: PC10,9 2316: PC10,12,9 X1: PC255 X2: PC255
13:50:58.502: Sel pins: PB0,1,2,255
13:50:58.502: Status pin: PNONE255
13:50:58.502: -----
13:50:58.502: ROM info ...
13:50:58.502: # of ROM sets: 4
13:50:58.502: Set #0: 1 ROM(s), size: 16384 bytes
13:50:58.502:   ROM #0: kernal.901227-03.bin, 2364, CS1: Active Low, CS2: -, CS3: -
13:50:58.502: Set #1: 1 ROM(s), size: 16384 bytes
13:50:58.502:   ROM #0: basic.901226-01.bin, 2364, CS1: Active Low, CS2: -, CS3: -
13:50:58.502: Set #2: 1 ROM(s), size: 16384 bytes
13:50:58.502:   ROM #0: characters.901225-01.bin, 2332, CS1: Active Low, CS2: Active High, CS3: -
13:50:58.502: Set #3: 1 ROM(s), size: 16384 bytes
13:50:58.502:   ROM #0: dead%20test.BIN, 2364, CS1: Active Low, CS2: -, CS3: -
13:50:58.502: Execute from: Flash
13:50:58.502: -----
13:50:58.502: Running ...
13:50:58.502: !!! VOS not ready - proceeding anyway
13:50:58.502: Set VOS to scale 1
13:50:58.502: HSI cal value: 0x7C
13:50:58.502: Not trimming HSI
13:50:58.502: Configured PLL MNPQ: 8/100/0/4
13:50:58.502: PLL started
13:50:58.502: SYSCLK/2->APB1
13:50:58.502: Set flash config: 3 ws
13:50:58.502: PLL->SYSCLK
13:50:58.502: ROM sel/index 0/0
13:50:58.502: ROM filename: kernal.901227-03.bin
13:50:58.502: ROM type 2364
13:50:58.502: ROM size 16384 bytes
13:50:58.502: ROM kernal.901227-03.bin preloaded to RAM 0x20000000 size 16384 bytes
13:50:58.502: Set ROM count: 1, Serving algorithm: 0, multi-ROM CS1 state: -
13:50:58.502: Start main loop - logging ends
13:50:58.502: Serve ROM #0: kernal.901227-03.bin via mode: 0
13:50:58.502: ROM type: 2364
13:50:58.502: CS1 active low
13:50:58.502: -----
13:50:58.502: Register locations and values:
13:50:58.502: GPIOA_MODER: 0x28000000
13:50:58.502: GPIOA_PUPDR: 0x24000000
13:50:58.502: GPIOA_OSPEEDR: 0x0000AAAA
13:50:58.502: GPIOC_MODER: 0x00000000
13:50:58.502: GPIOC_PUPDR: 0xA0000000
13:50:58.502: VAL_GPIOA_ODR: 0x40020014
13:50:58.502: VAL_GPIOA_MODER: 0x40020000
13:50:58.502: VAL_GPIOC_IDR: 0x40020810
13:50:58.502: CS check mask: 0x00000400
13:50:58.502: CS invert mask: 0x00000000
13:50:58.502: Data output mask: 0x28005555
13:50:58.502: Data input mask: 0x28000000
13:50:58.502: ROM table: 0x20000000
13:50:58.502: -----
```

## `MAIN_LOOP_LOGGING`

This enables logging within the main_loop (see [`rom_impl.c`](/sdrr/src/rom_impl.c).  It does not control logging after every byte is served - see [`MAIN_LOOP_ONE_SHOT`](#main_loop_one_shot)

You mus have `SWD` and `BOOT_LOGGING` enabled for this to work.

## `MAIN_LOOP_ONE_SHOT`

This makes a log after every requested byte has been served, once the chip select has gone inactive.

Some notes:

- It is **highly** likely that enabling this will cause the SDRR to not be able to keep up with the requested data, and so it is disabled by default.
- It is also likely, unless you are only querying the odd byte here and there, that RTT will not be able to keep up with the logged data.
- Although the address and data values served are logged, these are **mangled** versions of each - they are the value read directly from the STM32 GPIO port with the address and CS lines, and the value written directly to the STM32 GPIO port with the data lines.  As the pin mapping is complex, they are not the values you would naively expect to see.  See [Technical Details](/docs/TECHNICAL-DETAILS.md) for more information on the the pin mapping.

This logging is disabled by default and can be enabled by setting the `MAIN_LOOP_ONE_SHOT` configuration option to `1`.  This type of logging is useful for debugging the SDRR's main loop.

You must have `SWD`, `BOOT_LOGGING`, and `MAIN_LOOP_LOGGING` enabled for this to work.
