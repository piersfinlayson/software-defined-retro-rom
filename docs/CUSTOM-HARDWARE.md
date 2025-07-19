# Custom Hardware

This document explains the key steps for designings custom SDRR hardware and integrating it with the SDRR build system.

## Hardware Design Requirements - STM32F4xxR

There are a number of mandatory requirements when designing your own SDRR hardware.  These are explained in [Requirements Justification](#requirements-justification).

If you are using an STM32F4xxR based system:

- You **must** assign all of the address lines and chip select lines to Port C.
- You **must** "right justify" the address and chip select lines, so they start at PC0, and use contiguous lines.
- If you provide X1/X2 they **must** follow the address and CS lines, so be PC14/PC15.
- You **must** assign the data lines to pins PA0-7 inclusive.

With the above requirements, the address and chip select lines, and data lines, can be assigned to the specified range of pins in any order.  Address and CS lines can be mixed together.

In addition, it is recommended to:

- Avoid assigning additional purposes to PA13/PA14, but dedicating them to an SWD programming/debug interface.
- Avoid assigning additional purposes to PA8, but break it out to a test point, for optional MCO access.
- Avoid connecting any external pull-ups/downs to PB2, as it is also BOOT1.  (PB2 can be used, for example as one of the image select jumpers.)
- Avoid any external pull-ups/downs on the image select lines, as the firmware enables internal pull-downs on these lines.
- Expose BOOT0.  While the firmware will keep SWD active at all times, if so configured, it may still be useful to be able to force the device into the bootloader mode by pulling BOOT0 high.
- Tie NRST high, to avoid unexpected glitches.
- Avoid the RT9013 or other switching regulators to provide 3.3V power to the STM32F4.  This voltage regular causes problems with some systems.  AP2112K-3.3 is a good, small form factor alternative, which is also used on the official SDRR hardware.

## Non-STM32F4xxR Hardware

If you are using a different model of microcontroller, read [Requirements Justification](#requirements-justification), and select the microcontroller and design the hardware following the same philosophy.

## Firmware Integration

These instructions apply to STM32F4xxR based hardware only.  If you have selected a different microcontroller, you will likely need to adapt the [firmware source code](/sdrr/).

Add a config file to either [`sdrr-hw-config/third-party`](/sdrr-hw-config/third-party/README.md) or [`sdrr-hw-config/user`](/sdrr-hw-config/user/README.md) using the name of your hardware configuration, e.g. `piers.rocks-24pin-rev-a.json`.

This file contains the ports and pins used for each function.  Use the existing files in [`sdrr-hw-config/`](/sdrr-hw-config/) as a template.

No firmware changes are required, so long as you follow the requirements above.  The firmware will automatically detect the hardware configuration based on the config file, and apply the correct pin mappings.

## Building

From the root of the SDRR repository, run:

```bash
STM=f411re HW_REV=piers.rocks-24pin-rev-a BOOT_LOGGING=1 make       # Replace f411re with your STM model
STM=f411re HW_REV=piers.rocks-24pin-rev-a BOOT_LOGGING=1 make test
```

Examine the build logs to ensure your configuration has been detected correctly.

It is **strongly recommended** to run the tests with `make test` after building your firmware, to ensure that the firmware has correctly detected and applied your hardware configuration, and mapped the ROM image(s) correctly.  

## Testing

Flash the firmware to your hardware, using an SWD programmer with RTT support:

```bash
STM=f411re HW_REV=piers.rocks-24pin-rev-a BOOT_LOGGING=1 make run
```

Check the logs indicate a successful boot and the device is serving the image.

Now test the device in an appropriate host.

## Requirements Justification

The 64 pin STM32F4xx microcontroller is used, because it has sufficient 5V tolerant GPIOs for this application, in suitable locations.  In particular, Port C is required, as Port A contains the 2 SWD pins, which we want to keep free for SWD debugging.  All GPIOs on this range of microcontrollers are 5V tolerant, and have sufficient tolerances for this purpose.  However you should double check the datasheet for your specific microcontroller model.

Port C is used for the address and CS lines, as:

- All 16 GPIOs are exposed on the 64-pin package.  SDRR needs a minimum of 14 for the address and CS lines for the 2364 ROM.
- We want the address and CS lines states to be on the same port as each other so they are retrievable with a single assembly instruction.
- We want them to start from PC0, and to be contiguous, to avoid any bit shifting or other manipulation before using the read input register value as an offset into the ROM image being served.

Port A is used for the data lines, as:

- We need 8 GPIOs.
- We want them to be pins 0-7, to avoid any bit shifting or other manipulation before applying the read data byte to the port output register.
- Port B has BOOT1 on PB2, which we want to avoid on the address or data buses (or CS line), as its state may change during boot - either conflicting with the bus, or causing the device to boot into an incorrect mode.
- Port C is taken.

Note that on most of the STM32F4xxR devices PB11 is not broken out.  (It is present on the F405 and F415.)
