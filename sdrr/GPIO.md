# GPIO

## 5V Tolerant Pins

The STM32F103R (64-pin package) variants has the following 5V tolerant GPIO pins:

| Pin | Function | Alternate Function |
|-----|----------|--------------------|
| PB15 | A12 |  |
| PB14 | A11 |  |
| PB13 | A10 |  |
| PB12 | A9 |  |
| PB11 | A8 |  |
| PB10 | A7 |  |
| PB9 | A6 |  |
| PB8 | A5 |  |
| PB7 | A4 |  |
| PB6 | A3 |  |
| PB4 | A2 |  |
| PB3 | A1 |  |
| PB2 | A0 |  |
| PC12 |  |  |
| PC11 |  |  |
| PC10 |  |  |
| PC9 | D7 |  |
| PC8 | D2 |  |
| PC7 | D1 |  |
| PC6 | D0 |  |
| PA15 |  |  |
| PA14 |  | SWCLK |
| PA13 |  | SWDIO |
| PA12 | CS1 |  |
| PA11 | D3 |  |
| PA10 | D4 |  |
| PA9 | D5 |  |
| PA8 | D6 |  |
| PD2 |  |  |

### rev a

We some errors in the first version of the board, and this has led to less efficient code.  Specifically:
- On the 2332 the A12 pin is actually CS2.  In order to support 2332 ROMs more efficiently (saving 2 cycles), the A12 pin should be connected to PA15, in order to share the same port and bank as the CS1 pin.
- Similarly, to support a 2316 ROM, the A11 pin would ideally also be connected to either PA13 or PA14 (and only supported with SWD disabled).
- Alternatively - and perhaps a better approach, would be to use the 3 spare port C pins - probably PC6-PC8 for the CS lines.

### rev b

- Moved CS1 from PA12 to PC, and added CS2/3 to PC.  Note CS2/3 labelled per the 2316.  (2332 CS2 is pin 21 like CS3 on 2316.)  Hence 2332 technically uses CS3 not CS2.
- Left D0-7 split across Port A and Port C, but will move to to entirely Port A for production, losing SWD support.

## Non-5V Tolerant Pins

We also make use of some of the non-5V tolerant GPIO pins:

| Pin | Function | Alternate Function |
|-----|----------|--------------------|
| PC0 | SEL0 |  |
| PC1 | SEL1 |  |
| PC2 | SEL2 |  |

These are ROM bank selection pins, and allow the ROM image from flash to be seleccted at power on.  The software does not support reading the bank after boot, so changing the jumpers after boot will not take effect until a reboot.

We use non-5V tolerant pins, because we can - they either get pulled down to ground (default) or pulled up to 3.3V (when the jumper is in place).
