# Manufacturing Test

When manufacturing a new batch of One ROMs, whether it be hand soldering them, or using an assembly house, it is recommended to perform the following set of tests to ensure proper functionality of One ROM.

1. **Visual Inspection**: Check for any obvious defects in the soldering, such as cold joints, bridges, or missing components.
2. **[Short Circuit Test](#short-circuit-test)**: Check for lack of continuity between GND, 3V3 and 5V.
3. **[Jumper/LED Test](#jumper-led-test)**: Use `sdrr-check` to verify the jumpers and LEDs are correctly connected.
4. **[Live Test](#live-test)**: Write a kernal ROM to the One ROM and boot a host with it.  The C64 is a good choice, as it is relatively taxing on One ROM.

If all tests pass, you can be reasonably confident that One ROM has been manufactured correctly.

## Short Circuit Test

The best places to check for shorts between are:

- GND located on the left most programming pin, at the bottom of the PCB - or pin ROM 12.
- 5V located on the right most programming pin, at the bottom of the PCB - or pin ROM 24.
- 3V3 located as the second row of jumper pins from the top of the board.

There should be no continuity between any of these rails, but a multimeter on continuit mode may give a quick chirp when checking between GND and the power pins.

## Jumper/LED Test

Ensure no jumpers are installed.  Flash the `sdrr-check` utility to One ROM, and connect to logging.

Throughout the tests, the LED should toggle on and off.

Then run the following tests:

- Set/image select jumpers 1, 2, 4 and 8 - close then open each jumper and check that a log is produced.
- NRST jumper - close - check LED "sticks" on or off - then open and check LED starts flashing again.

There's no way to test the BOOT0 pin from within STM32F4 firmware.  

## Live Test

Replace `f411re` and `24-f` with the appropriate STM32 device and hardware revision for your One ROM.

```bash
# From the root of the One ROM repository
STM=f411re HW_REV=24-f CONFIG=config/c64-no-destestmax.mk make
```

Install in your C64's kernel ROM socket, with no jumpers installed, and boot the C64.  If the C64 boots up normally, your One ROM works.
