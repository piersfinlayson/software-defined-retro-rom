# Configuration

The top-level [Makefile](/Makefile) allows you to specify a significant number of configuration options, including developer and debug level options.  These are commented in the Makefile.

You can either pass options on the command line, or specify them in a config file which you then include using the `CONFIG` environment variable.  It is recommended to do this rather than modify the original `Makefile`.  For example:

```bash
# Specify ROM images on the command line
STM=f411re HW_REV=e ROM_CONFIGS=file=images/test/0_63_8192.rom,type=2364,cs1=0 make run
```

```bash
# Or use a pre-built config file
STM=f405rg CONFIG=config/c64.mk make run
```

## Mandatory Configuration

## `STM`

Defines your STM32 variant, for example:

```bash
STM=f446re
```

See [Supported STM32 Microcontrollers](/README.md#supported-stm32-microcontrollers) for the list of supported variants.

## `HW_REV`

Use this to specify the hardware revision of your SDRR's PCB.  It is silkscreened on the underside of the PCB.  Other ways to identify the revision are:

- Rev D
  - Uses 0402 passives.
  - Has 2x4 jumpers next 2 pin 1 (3 image select jumpers and the `BOOT0` jumper).

- Rev E
  - Uses 0603 passives.
  - Has 2x5 jumpers next to pin 1 (4 image select jumpers and the `BOOT0` jumper).
  - Has a status LED roughly between pins 10 and 14.

```bash
HW_REV=e
```

## `ROM_CONFIGS`

Use this option to specify all of the ROM images to include in your SDRR firmware.  They will be numbered, in the order you provide them, starting from 0.  They can then be selected at boot type, using the image select jumpers.

The format of ROM_CONFIGS is:

```bash
ROM_CONFIGS=FIRST_ROM_INFO SECOND_ROM_INFO THIRD_ROM_INFO ...
```

Where each `..._ROM_INFO` takes the form:

```bash
key1=value1,key2=value2,...,command1,command2
```

Supported key/value pairs are:

- `file`: Required.  The path to the ROM file to include in the firmware.  This can be a local file, or a URL.
- `extract`: Optional.  Valid only if `file` is a URL, and the file to be downloaded is a ZIP file.  This specifies the path within the ZIP file to extract the ROM from.  Use URL encoding to encode any special characters in the path, such as spaces (%20).
- `licence`: Optional.  URL to path for this ROM image's licence.  Including this causes the build process to force the user to explicitly acknowledge they accept the licence terms before the firmware is built.
- `type`: Required.  One of 2364, 2332, 2316 for 8KB, 4KB and 2KB ROM images respectively.  The `file` must be small enough to fit into the specified type.
- `cs1`: Required.  0 or 1, specifying chip select line 1 acts as active low (0) or active high (1).
- `cs2`: Valid for `type=2332` or `type=2316`.  0 or 1, specifying chip select line 2 acts as active low (0) or active high (1).
- `cs3`: Valid for `type=2316`only. 0 or 1, specifying chip select line 3 acts as active low (0) or active high (1).

Supprted commands are:

- `dup`: Optional.  Valid only if file size is less than indicated by `type`, and only if ROM type size if an exact multiple of file size.  Causes the ROM image to be duplicated to fill the remaining space.  This is useful for 2KB and 4KB ROM images which need to be exposed as larger ROMs.
- `pad`: Optional.  Valid only if file size ia less than indicated by `type`.  Pad out the remaining space with 0xAA bytes.  Useful for 2KB and 4KB ROM images which need to be exposed as larger ROMs.

See [`config`](/config/) for examples of how to specify ROM images.

## Advanced Configuration

A few of the most useful advanced options are described below, but see the [Makefile](/Makefile) for the full list.

### `SWD`

Setting this to 1 enables SWD support within the firmware.  If you do **not** enable this, you will be unable to easily reprogram the device after flashing the image - you will have to use the `BOOT0` jumper.  Also see [`BOOTLOADER`](#bootloader).

This option is also required for [`BOOT_LOGGING`](#boot_logging) (and the other [`logging`](/docs/LOGGING.md) options) to work, as it uses SWD for logging.

### `BOOT_LOGGING`

This is enabled by default, and logs using RTT over SWD as the SDRR boots up.  It stops logging when it enters its main ROM serving loop for performance reasons.

You may wish to disable this logging, if startup speed of SDRR is a problem.  With boot logging enabled, SDRR takes around 2ms to start up and be ready to serve the ROM.  Without boot logging it takes around 500us.

These timings will vary based on clock speed.

### `FREQ`

This is not specified by default, which uses the maximum frequency for the chosen STM32F4xxR variant.  You can lower the frequency if you want to reduce SDRR's power draw.  The [Makefile](/Makefile) gives some guidance on what values are required by what systems, but your mileage may vary.

### `OVERCLOCK`

⚠️ Use with Caution

Enabling this feature allows you to specify a higher frequency than supported by a particular STM32F4xxR variant.  SDRR uses very few peripherals and hence it should be possible for the STM32 to be powered at a higher than stock frequency, but the silicon may not be stable.

### `STATUS_LED`

SDRR hardware revisions E and onwards provide a status LED, which is lit once the SDRR has booted and is ready to serve the image.  It also flashes this LED if it crashes.

If you want to reduce power consumption, or just not have the LED lit, use this option (or do not populate the LED on your board).

### `BOOTLOADER`

In addition to the BOOT0 jumper, SDRR provides an option, disabled by default, which will put the STM32 into bootloader mode if all image select jumpers are closed.  This may be useful if you consistently hit [problems flashing](/README.md#flashing-problems).

To use:

- Close all image select jumpers (`1`, `2`, `4`, `8`), but not the BOOT0 (`B0`) jumper.
- Reboot the SDRR device.  It will now be in bootloader mode.
- Optionally change the image select jumpers to the selection you want when the device next boots.
- Reprogram the SDRR device.  It will then reboot, re-read the image select jumpers and start serving the appropriate image.

Enabling this option means that the 16th image (if installed) cannot be selected.
