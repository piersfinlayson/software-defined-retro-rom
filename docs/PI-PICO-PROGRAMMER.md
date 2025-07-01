# Raspberry Pi Pico as a Programmer

To build the debug probe firmware for a stock Raspberry Pi Pico:

```bash
git clone https://github.com/raspberrypi/debugprobe
cd debugprobe
git submodule update --init --recursive  # This step takes some time
mkdir build
cd build
cmake -DDEBUG_ON_PICO=ON ..
make  # You can use multiple cores to speed this up, e.g. `make -j4`
```

Mount your Raspberry Pi Pico in programming mode and copy the `debugprobe_on_pico.uf2` file to the Raspberry Pi Pico's file system.

The required pins on the Pico Debug Probe are:

- CLK - GP2 (pin 4)
- DIO - GP3 (pin 5)
- GND - any GND pin (3, 8, 13, 18, 23, 28, 33, 38)
- 5V - (if **not** connected to a retro system) - VBUS (pin 40)
