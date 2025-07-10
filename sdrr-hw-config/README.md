# sdrr-hw-config

This directory contains the SDRR PCB hardware configuration files, which tell the software what the port/pin mappings for the various pin types are.

This allows new hardware revisions (i.e. new PCB layouts with different pin mappings) to be supported without needing to modify the source code.

## Usage

See the `*.json` files in this directory for the format of the configuration files.  There are some restrictions on which pins can be used for which pin types:

- All pins of a particular type (address, data, CS/CE/OE in particular) must be on the same port.
- For now, CS/CE/OE pins must share the same port as the address pins, to allow a single STM32F4 port to be read for both address and chip select status.

You should store your files either in:

- [`user/`](/sdrr-hw-config/user/) - for your own, private, hardware configurations
- [`third-party/`](/sdrr-hw-config/third-party/) - for hardware configurations that you plan to submit pull requests for, and want to share with the community

[`sdrr-gen`](/sdrr-gen/README.md) looks for hardware configuration files in this directory, as well as the directories above, and uses them to generate the firmware for your hardware.
