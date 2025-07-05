# Future Enhancements

This file contains a list of enhancements under consideration for future versions.

- Support multiple ROM sockets simultaneously, allowing a single SDRR to replace multiple ROMs in a system - see [Multiple ROM Support](#multiple-rom-support---prototyping).
- Support 16KB 23128 ROMs, and possibly 23256/23512 ROMs - see [23128 Support](#23128-support---prototyping).
- Add a WiFi programming daughterboard, allowing the SDRR to be programmed over WiFi - see [WiFi Programming](#wifi-programming---prototyping).
- Support the Apple 2513 character ROM - see [Apple 2513 Character ROM](#apple-2513-character-rom).

## Multiple ROM Support - prototyping

Suggestion by Adrian Black.

Expose additional pins on the PCB to allow the chip select lines of other ROMs to be connected to the SDRR.  Assuming all ROMs share the same address and data lines, this allows a single SDRR to serve as a drop-in replacement for all ROMS in the system.

Hardware [revision F](/sdrr-pcb/unverified/24-pin-rev-f/README.md) includes support for two additional ROM select pins.

This is fairly straightforward to implement in firmware for 2364s.  We just need a single 64KB image, with the X1/X2 pins addressing the appropriate segment of the image to use for each ROM.

However, this is more complex for 2332s, as they have 2 CS lines.

- On machines where the 2332 only has a single active CS line (because the other is permanently tied to ground/high), the same approach works.
- In some cases multiple 2332s share one of the CS line - for example in the 2040/3040/404 disk drives, CS2 is shared and used to carry Phi2 to the ROMs.  Therefore it should be possible to use one of the main CS pins to select any of the 2332s, and the other CS pin + X1/X2 to select the appropriate segment of the image.

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

Suggestion by Adrian Black.

This ROM was used in the Apple I and early Apple II systems, and is a 24-pin ROM.  It has a different pinout to the 23xx ROMs, and in the Apple I use case, has to cope with negative voltages on the address lines.  This can be done with diodes, see [P-L4B](https://p-l4b.github.io/2513/), but there are likely other approaches.

In any case, as additional components are required, in addition to a different PCB, this is probably best tackled using a separate adapter PCB.
