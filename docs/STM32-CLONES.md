# STM32 Clones

Various Chinese manufacturers produce clones of the STM32F4xxR series, which claim to be pin compatible, cheaper, and can be used in place of the STM32F4xxR series.

For example:

| Manufacturer | Part #        | Max Speed | Flash  | RAM*   | Price† | Stock‡ | Tested |
|--------------|---------------|-----------|--------|-------|--------|-------------------|--------|
| STM          | STM32F405RGT6 | 168MHz    | 1024KB | 128KB | $3.25  | 0 | Yes |
| GigaDevices  | [GD32F405RGT6](#gigadevices-gd32f405rgt6)  | 168MHz    | 1024KB | 128KB | $3.32  | 1002 | Yes |
| Geehy        | [APM32F405RGT6](#geehy-apm32f405rgt6) | 168MHz    | 1024KB | 128KB | $2.78  | 2262 | No |
| CKS          | [CKS32F405RGT6](#cks-cks32f405rgt6) | 168MHz    | 1024KB | 128KB | $2.77  | 962 | No |
| Artery       | [AT32F405RCT7](#artery-at32f405rct7)  | 216MHz    | 256KB | 96KB | $2.34  | 160 | No |

\* SRAM, not CCM (CCM is unused by One ROM)  
† Price at LCSC, for 1, as of July 2025.  aliexpress often has cheaper prices of the STM and other brand parts.  
‡ Stock at LCSC, as of July 2025

## GigaDevices GD32F405RGT6

This IC has been tested, primarily using:
- hardware revision E
- firmware v0.4.0
- a C64, as character ROM

Key differences from the STM32F405RGT6:
- Much slower boot - 150ms vs 2-3ms.  This may be due to the IC preloading first 512KB of flash to RAM before executing it from this RAM.  This may be GigaDevice's alternative to STM's Art Accelerator.
- Only the first half (512KB) of flash is available for code execution - see previous bullet for likely explanation.  A larger firmware image hasn't been tested, but in the One ROM firmware code lives at the start of the firmware, so this is unlikely to cause a problem, even with images beyond 512KB in size.
- 15-20% higher performance Hz for Hz.  It is equivalent Hz for Hz to the non F405 STMs - the STM32F405 is around 20% slower than these.
- Much high overclock potential.  It has been overclocked, stably for severals hours, to 400MHz.  It gets quite warm at this speed, but is much cooler at 300MHz.
- No requirement for capacitors on the VCAP pins.  (They can be populated, but are not required.)
- Around 10% cheaper (August 2025).

If your application can stomach the much slower boot time (it is fine for a C64 for axample), then this is a good alternative to the STM32F405RGT6.

## Geehy APM32F405RGT6

This device has bene tested, primarily using:
- hardware revision F
- firmware v0.4.0
- a C64, as character ROM

Key differences from the STM32F405RGT6:
- Marginally faster boot time - roughly 1.8ms.
- Sam performance Hz for Hz.  I.e. slower than non F405's Hz for Hz.
- Lower overclock.  Seems stable at 190MHz, but unstable at 200MHz.
- Around 20% cheaper (August 2025).

This is a good alternative to the STM32F405RGT6, if you are looking for a lower cost alternative, and overclock is not a priority.

## CKS CKS32F405RGT6

Currently untested.

## Artery AT32F405RCT7

Untested.

This device has significantly different specs than the other F405s listed:

- It has a higher max clock speed.
- It has less RAM
- It has less flash.

Even with these changes it _should_ be compatible with One ROM firmware, but this is yet to be confirmed.
