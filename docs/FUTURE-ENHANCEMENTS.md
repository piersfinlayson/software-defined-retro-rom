# Future Enhancements

This file contains a list of enhancements under consideration for future versions.

- Support 16KB 23128 ROMs, and possibly 23256/23512 ROMs - see [23128 Support](#23128-support---prototyping).
- Add a WiFi programming daughterboard, allowing the SDRR to be programmed over WiFi - see [WiFi Programming](#wifi-programming---prototyping).
- Support the Apple 2513 character ROM - see [Apple 2513 Character ROM](#apple-2513-character-rom).

## 23128 Support - prototyping

Newer C64s, 1541Cs, 1541-IIs, 1571s, and other systems used larger 16KB 23128 ROMs, which has a 28-pin DIP package.  This enhancement adds support for 23128 ROMs and _possibly_ 23256/23512 ROMs.

This requires a new PCB design, and the [28 pin revision A](/sdrr-pcb/unverified/28-pin-rev-a/README.md) is under development to support this.

23128 replacement should be possible, using the timings are achievable (likely with the F405/F446).

However 23256/23512 may not be viable, because the combined number of chip select and address lines exceeds 16 - which s the number of GPIOs which can be read using a single instruction on the STM32.

## WiFi Programming - prototyping

This enhancement adds an optional daughtboard, connecting on top, to the programming pins, to expose them over WiFi.

A prototype PCB design [WiFi Programmer revision A](sdrr-pcb/unverified/wifi-prog-rev-a/README.md) is under development.

This prototype uses the ESP32-C3-Mini-1, so that the daughtboard footprint does not exceed SDRR's.

This enhancement is likely to take a bunch of development work to build the appropriate daughtboard firmware, although there a number of BlackMagicProbe ports to ESP32, one of which may be a good startig point.

- https://github.com/Ebiroll/esp32_blackmagic
- https://github.com/flipperdevices/blackmagic-esp32-s2

## Apple 2513 Character ROM

Suggestion by [Adrian Black](https://www.youtube.com/@adriansdigitalbasement).

This ROM was used in the Apple I and early Apple II systems, and is a 24-pin ROM.  It has a different pinout to the 23xx ROMs, and in the Apple I use case, has to cope with negative voltages on the address lines.  This can be done with diodes, see [P-L4B](https://p-l4b.github.io/2513/), but there are likely other approaches.

In any case, as additional components are required, in addition to a different PCB, this is probably best tackled using a separate adapter PCB.

As there are already inexpensive replacements for this ROM, and it's more nice than regular 23xx ROMs, this may not be implemented.
