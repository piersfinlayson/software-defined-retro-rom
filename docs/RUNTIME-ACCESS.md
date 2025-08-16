# Runtime Access

The Software Defined Retro ROM can be accessed and controlled at runtime by using [Airfrog](https://piers.rocks/u/airfrog) or other SWD probes. This allows you to inspect the firmware and runtime state of One ROM, and change its configuration and ROM data - **while it is serving ROMs**.

## Features

Access to One ROM while it is running is via ARM's Serial Wire Debug protocol, which provides runtime access **in paralle to the MCU's core** to the device's flash, RAM and hardware registers.

Airfrog is designed to plug on top of One ROM, using its programming port, and draws power from One ROM's 5V rail.  It automatically connects to WiFi and exposes a number of interfaces for accessing and controlling One ROM.

1. **Web Interface**: Use a browser to
  - inspect the details One ROM's firmware, including what ROM images are installed
  - view the runtime configuration and state of One ROM, including which ROM image(s) are currently being served, and how many times the CS line(s) have gone active (`COUNT_ROM_ACCESS` feature)
  - change the runtime configuration of One ROM, including which ROM image(s) are currently being served.

3. **Custom Firmware**: Write your own custom firmware for Airfrog, using its API and extensive examples as a base, to access and control One ROM.

4. **Programming**: Airfrog can be used to erase and reflash One ROM, using a dedicated SWD programmer such as [probe-rs](https://probe.rs/).  This means you can reflash your One ROM without needing to dismantle the host or connect any wires.

2. **REST API**: Use a command line or other script to access flash, memory and hardware registers, and to change the runtime configuration of One ROM - such as the image being served - using SWD-like primitives.

5. **Binary API**: For high performance programmatic access to low-level SWD primitives.

## Examples

### ROM Access Counting

The following is an example output from an Airfrog example which broadcasts the ROM access count from a running One ROM via MQTT.  This telemetry is then picked up and graphed by a PC-based Python script.  This example used 3 One ROM instances (C64 Kernal, BASIC and character ROMs), each with its own Airfrog.

![ROM Access Graph](images/access-rate.png)
