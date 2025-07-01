# Voltage Levels

## Background

One of the biggest difficulties of this project, alongside hitting the timing requirements, was finding a microcontroller tha could support the 5V logic levels required by retro systems.

The STM32F1 series (64-bit variant) on the face of it has the required number of 5V tolerant (FT) GPIOs, but on further examination, there are not sufficient contiguous and starting at pin 0, FT pins on any port to hit the required performance - that is to allow the assembly code to perform optimal chip select comparisons, address lookups, and apply the results those to the data pins.

Therefore the STM32F4 series was chosen, both to give sufficient raw horsepower (clock speed and flash instruction prefetch and cache), and also to provide the required FT hardware configuration.

## 5V and 3.3V Logic Levels

As SDRR's purpose is to replace 5V logic level ROMs, it must be compatible with the voltage levels required by those systems.  We do this by examining and comparing the [STM32F411 datasheet](https://www.st.com/resource/en/datasheet/stm32f411re.pdf) (as a representative STM32F4 family chip) with the datasheets for the:

- [6502](http://www.6502.org/documents/datasheets/mos/mos_65ce02_mpu.pdf)
- [6567](http://www.6502.org/documents/datasheets/mos/mos_6567_vic_ii_preliminary.pdf) (C64 VIC-II chip)
- [6560/6561](http://www.6502.org/documents/datasheets/mos/mos_6560_6561_vic.pdf) (VIC-20 VIC chip)

### SDRR outputs, 6502 inputs

Note - SDRR's 5V outputs are the data lines, PA0-7.  These are set to "fast" speed, with IIO = +8mA.  We avoid "high" speed, as this can result in a maximum low output voltage VOL of 1.3V, which is too high for the 6502's maximum low input voltage VIL of 0.8V.

| SDRR VOL | 6502 VIL | 6567 VIL | 6560/6561 VIL |
|----------|----------|----------|---------------|
| 0.4V     | 0.8V     | 0.8V     | 0.4V          |

SDRR's maxmimum low output voltage is less than the 6502/6567's maximum low input voltage, and the same as the 6560/6561, so we're good.

| SDRR VOH | 6502 VIH | 6567 VIH | 6560/6561 VIH |
|----------|----------|----------|---------------|
| 2.4V*     | 2.0V     | 2.0V     | 2.4V          |

*Note it is highly likely the STM32F4 datasheet says that VOH is VDD-0.4V = 2.9V in this case - so we take 2.4V to be conservative.*

SDRR's maximum minimum output voltage is greater than the 6502/6567's minimum high input voltage, and the same as the 6560/6561, so we're good.

### SDRR inputs, 6502 outputs

| SDRR VIL | 6502 VOL | 6567 VOL | 6560/6561 VOL |
|----------|----------|----------|---------------|
| 1.0V     | 0.4V     | 0.4V     | 0.4V          |

SDRR's minimum low input voltage is greater than the 6502/6567/6560/6561's maximum low output voltage so we're good.

| SDRR VIH | 6502 VOH | 6567 VOH | 6560/6561 VOH |
|----------|----------|----------|---------------|
| 2.3V     | 2.4V     | 2.4V     | 2.4V          |

SDRR's minimum high input voltage is less than the 6502/6567/6560/6561's minimum high output voltage so we're good.

### Absolute Maximum STM32F4 VIL

The absolute maximum VIN for the STM32F4 on its FT pins is VDD+4.0 = 7.3V.  This is higher than the absolute maximum supply and input voltage of the 6502/6567/6560/6561 which is 7.0V.

Hence the STM32F4 will not be damaged by a system over-voltage scenario, unless the 6502/VIC-II/VIC (and other compoents) have likely already been damaged.
