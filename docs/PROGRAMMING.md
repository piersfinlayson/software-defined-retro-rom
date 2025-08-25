# Programming One ROM

The instructions in [Quick Start](/README.md#quick-start) show you how to flash the One ROM device using `make run`, for example:

```bash
MCU=f411re CONFIG=config/c64.mk make run
```

## Programmer

One ROM uses SWD and [probe-rs](https://probe.rs) to program the STM32F4 microcontroller.  You can use any SWD programmer that supports the STM32F4xxR series, such as:

- ST-Link (or clone)
- Raspberry Pi Debug Probe
- a Raspberry Pi Pico with the debug probe firmware installed.

The ST-Link and Pi Debug Probe should work out of the box.

Connect the CLK, DIO and GND pins from the programmer to One ROM.  If One ROM is **not** currently installed in a retro system, you will also need to provide 5V to it (and ensure the programmer and 5V source share a common GND).

To use a Raspberry Pi Pico as a programmer, see the instructions in [PI-PICO-PROGRAMMER.md](/docs/PI-PICO-PROGRAMMER.md).

## Flashing in-situ

You can flash One ROM when it is installed in your retro system - you will need the CLK, DIO and GND lines connected and the retro system powered on to do this.

⚠️ **Do not** externally power One ROM with 5V when installed in a retro system, even if it is powered off.  Doing so may overload your 5V source, or cause damage to your programmer, One ROM, or retro system.

If the retro system is actively running from One ROM it will crash while flashing, and you will need to reset it once flashing is complete.  This happens as One ROM stops serving data while being reflashed.

If you are using One ROM as the device's character ROM, your device may not crash, but you may instead see corruption on screen while flashing.  The display should return to normal once the flashing is complete (assuming you have reflashed with a valid character ROM at the same image location).

If One ROM is installed as a ROM in the system but not being actively used, your system will probably not crash while flashing - but it may do if the STM32 causes any glitches on the address or data buses.  Be prepared to reset the system once flashing is complete if necessary.

## Flashing Problems

One ROM leaves SWD support active while running by default (this is controlled by the [`SWD`](/docs/CONFIGURATION.md#swd) configuration option), so it should be possible to reflash One ROM after initial flashing, without installing the device's BOOT0 jumper.

However, if you have problems with flashing, reboot the device with the BOOT0 jumper installed, and reflash it.  (The BOOT0 jumper forces the STM32 to boot into its bootloader.)  Once flashing it complete, remove the BOOT0 jumper and reset the device (a power cycle is required) - your new firmware should now be running.

There is also a [`BOOTLOADER`](/docs/CONFIGURATION.md#bootloader) configuration option which will put the STM32 into bootloader mode if all image select jumpers are closed.  This may be useful if you consistently hit problems flashing.
