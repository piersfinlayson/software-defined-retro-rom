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

This device has been tested, using One ROM hardware revision E, using a C64, as both kernal and character ROMs.

Some differences were noted:

- Anomalous pin behaviour on reset.  Status LED comes on immediately at "half" brightness on, then brightens, probably when the LED is actually turned on by the firmware.  On the STM32 the LED stays off until fully booted, then turns on.  This suggests the pins are in a different state to the STM32 on boot, but is unlikely to affect operation in practice.
- Longer startup time.  The GD32F405 takes significantly longer to boot than the STM32F405.  The precise time has not been measured - perhaps 0.5-0.75s, which is much more than the ~1-3ms of the STM32F405.  This is _unlikely_ to be long enough to cause an issue in a device with a long reset delay, and didn't cause an issue with the C64 being tested.
- Higher minimum clock speed.  The GD32F405 needed to be clocked at a minimum 81MHz to replace the C64 kernal ROM, vs the STM32F405's 79MHz.  This might just be normal variation between silicon, although the STM32F405 claims its internal oscillator is factory trimmed to within 1% at 25C, and the variation here is around 2.5%.

## Geehy APM32F405RGT6

Currently untested.

## CKS CKS32F405RGT6

Currently untested.

## Artery AT32F405RCT7

Untested.

This device has significantly different specs than the other F405s listed:

- It has a higher max clock speed.
- It has less RAM
- It has less flash.

Even with these changes it _should_ be compatible with One ROM firmware, but this is yet to be confirmed.
