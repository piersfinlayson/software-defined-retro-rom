# Technical Details

There are 4 parts to this project:

1. [`sdrr`](/sdrr/README.md) - The main firmware source code for the STM32F4 microcontroller.
2. [`sdrr-gen`](/rust/sdrr-gen/README.md) - A program to generate the required custom SDRR firmware files for your ROM images and configurations.
3. [`sdrr-tester`](/rust/sdrr-tester/README.md) - A test program for the Raspberry Pi Pico to test the SDRR device.
4. [`sdrr-pcb`](/sdrr-pcb/) - The PCB design files for the SDRR device.

## Firmware

This is a "bare metal" project, and doesn't require any external HALs, LLs or IDEs.  It does use SEGGER's RTT library for debug output, but this is automatically cloned by the Makfile process.

The firmware uses a number of techniques to reach the required performance:

- The microcontroller is clocked via the internal oscillator, to the maximum supported speed (84MHz for the STM32F401, 100MHz for the STM32F411, 168MHz for the STM32F405 and 180MHz for the STM32F446).
- Interrupts are not used, as they would slow down the main loop.
- The main loop function operates from flash, and leverages the STM32F4's fairly advanced flash prefetch and instruction caching capabilities.  This is faster than loading from RAM.
- The selected ROM image is pre-copied to RAM on boot, so the data is loaded very quickly based on the address pin state.
- Each ROM image is stored in a separate 16KB chunk of data on the flash.  This is true for all ROM types, even though they only take up 8KB, 4KB or 2KB of space.  This is done to make the main loop faster, so it can access to the ROM image using the chip select line states as part of the address offset into the ROM image.
- The ROM images are stored in flash in a "mangled" fashion - that is, to avoid having to remap the address lines and data lines in software when the ROM is being queried (as the PCB layout does not assign address and data lines to the equivalent STM32F4 GPIOs).  Therefore, if you look at the ROM images on the flash, they will appear very different to the original files you supplied.
- Main ROM emulation loop is done in assembly - see [`rom_impl.c`](/sdrr/src/rom_impl.c).

  - The assembly code preloads all pre-known values into registers prior to its main loop, to avoid loading the values in the loop.  (Nearly every register is therefore used.)
  - The required chip select masks are precomputed to avoid having to do any calculations in the main loop.
  - As the ROM's target access times different for address lines activation and chip select activation, and the latter is a tougher target to meet, the main loop tests the chip select lines more frequently than the address lines - essentially twice as often.
  - Low registers are used where possible to reduce the code size (which impacts how fast the microcontroller can load it).

There are some 8-bit systems that use these types of ROMs that use 2 Mhz clocks.  As of writing these haven't been tested.  However, a 1MHz Commodore PET can stably access the SDRR when it is clocked at just 30MHz.  Hence, with an STM32F405 running at 168MHz, it should be possible to support these systems as well.

## Hardware Selection

The STM32F4 microcontroller was chosen for a number of key reasons:

- The 64-pin variant has enough 5V tolerant GPIO pins to support 2332/2364 ROMs emulation.  (The 48-pin variant does not have sufficient contiguous pins to allow an optimal firmware implementation to meet the timing requirements.)
- Unlike the STM32F1xx series, the STM32F4xx has more 5V tolerant GPIOs which means the pin assignment can be made in a way that allows more optimal assembly code.  It is unlikely that the STM32F1xx series would be able to meet the timing requirements of the ROMs given it's sub-optimal 5V tolerant GPIO arrangement - in particular non-contiguous 5V-tolerant pins.
- The 64-pin variants fits (just) between the pins of the 2318/2332/2364 ROM sockets.
- It can run fast enough with its internal HSI oscillator to meet the required ROM access times, with no additional components.
- It has enough flash to store the SDRR firmware and numerous ROM images (each of which takes up 16KB on flash for reasons explained [earlier](#firmware)).
- It is widely available and cheap and can come in at less than $2 each at volume.  This compares very favourably to other 5V tolerant microcontrollers with enough GPIOs, like the ATMEGA.
- It uses a widely understood architecture and instruction set, from a reputable manufacturer.
- It tends to be lower cost than ATMEGA microcontrollers.

## PCB Design

### Component selection

- Surface mount components are used throughout, to reduce the PCB size.  Through-hole passives would not be viable in the desired footprint.
- 0603 passives are used, which are the smallest size that can be easily hand soldered.  Small pads were chosen to reduce the space, and allow the components to fit, rather than the large (Kicad "handsoldering") pads.  This may require the use of a small soldering iron tip and microscope if hand-soldering depending on the assembler's skill level and eye-sight.
- This designed provides the recommended number and value of bypass capacitors for the STM32F4xxR from the datasheet and application notes.
- Although the F405 and F415 pin-out varies slightly from the F401, F411, F446 and other F4 variants, the PCB design supports all of these variants, with different resistor and capacitor placement and values.
- An LQFP64 STM32F104RX**T** package is used, which just fits between the pins of the 2332/2364 socket.  A TBGA64 STM32F104RX**Y** package would be smaller, but is less common, much harder to hand solder so this was avoided, and makes it harder to examine the solder joints.  Due to the space constraints, care must be taken when soldering the STM32 to avoid briding the IC pins to the ROM socket pins, or other components.  If hand-soldering, it is recommended to tin the LQFP64 pads first, then use flux and hot-air to solder the STM32F4xxR to the PCB, and then more flux and solder to ensure all the pins are well-soldered.
- The STM32F4's internal oscillator is used to reduce component count.  This reduces the accuracy of the clock, but clock accuracy is not vital in this application.

### Routing

- 6 mil traces and spacings are used for signals, which is the minimum that can be easily manufactured and soldered by the cheap PCB services.
- 12 mil power traces are used, again with 6 mil spacing.
- As mentioned in the [firmware](#firmware) section, the address and data lines are not assigned in the same order on the STM32F4 ports as the original ROM, to ease PCB routing.  "Mangling" of the ROM images before storing on flash is done to allow the fastest possible serving of the images at run-time.

### Silk-screening

- A small silk-screen rectangle is provided on the underside of the PCB, to allow SDRR identification markings to be added.
- Little component silk-screen marking as been done, due to space constraints.  From hardware revision E onwards components which differ between F405/415 and other variants are marked with a `*` within the component footprint.  Due to the silk-screen size, this may appear as a dot `.`.

### Additional functionality

- An SWD port is exposed, to allow SDRR programming and debug tracing using RTT.  This port includes the NRST (STM32 reset) pin and a 5V input to externally power the device using programming.  **Do not** externally power the SDRR device when it is installed in a retro system, as this may damage the SDRR, 5v source or the retro system.
- The STM32F4 MCO1 pin is exposed, and MCO1 can be configured, using the Makefile, to expose PLL/4.  As the STM32F4 is clocked from the PLL, this shows what frequency the STM32F4 is running at (actually it shows 1/4 of that frequency).  This is not recommended to be used in production, as such a high frequency signal may cause interference with other components in the retro system, but it is useful for debugging purposes.
- A status LED is exposed from hardware revision E onwards to show when the device has successfully booted and is ready to serve ROM images.
- 2.54mm (0.1") pin spacing is used for the ROM bank select jumpers and programming pins, for ease of use.
