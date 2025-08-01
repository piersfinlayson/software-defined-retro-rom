# Runtime Access

The Software Defined Retro ROM can be accessed and controlled at runtime by using [Airfrog](https://piers.rocks/u/airfrog) or other SWD probes. This allows you to inspect the firmware and runtime state of the SDRR device, and change its configuration and ROM data - **while it is serving ROMs**.

## Features

Access to SDRR while it is running is via ARM's Serial Wire Debug protocol, which provides runtime access **in paralle to the MCU's core** to the device's flash, RAM and hardware registers.

Airfrog is designed to plug on top of SDRR, using its programming port, and draws power from SDRR's 5V rail.  It automatically connects to WiFi and exposes a number of interfaces for accessing and controlling SDRR.

1. **Web Interface**: Use a browser to
  - inspect the details SDRR's firmware, including what ROM images are installed
  - view the runtime configuration and state of SDRR, including which ROM image(s) are currently being served, and how many times the CS line(s) have gone active (`COUNT_ROM_ACCESS` feature)
  - change the runtime configuration of SDRR, including which ROM image(s) are currently being served.

3. **Custom Firmware**: Write your own custom firmware for Airfrog, using its API and extensive examples as a base, to access and control SDRR.

4. **Programming**: Airfrog can be used to erase and reflash SDRR, using a dedicated SWD programmer such as [probe-rs](https://probe.rs/).  This means you can reflash your SDRR without needing to dismantle the host or connect any wires.

2. **REST API**: Use a command line or other script to access flash, memory and hardware registers, and to change the runtime configuration of SDRR - such as the image being served - using SWD-like primitives.

5. **Binary API**: For high performance programmatic access to low-level SWD primitives.

## Examples

### ROM Access Counting

The following is an example output from an Airfrog example which broadcasts the ROM access count from a running SDRR via MQTT.  This telemetry is then picked up and graphed by a PC-based Python script.  This example used 3 SDRR instances (C64 Kernal, BASIC and character ROMs), each with its own Airfrog.

![ROM Access Graph](images/access-rate.png)
