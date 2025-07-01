# Rev E

This revision supports the STM32F401, STM32F411, STM32F405 and STM32F415 variants.

<div style="text-align: center;">
  <a href="/sdrr-pcb/unverified/rev-e/sdrr-rev-e.png">
    <img src="sdrr-rev-e.png" alt="SDRR rev E" width="400">
  </a>
</div>

## Files

- [Schematic](sdrr-rev-e-schematic.pdf)
- [Fab Notes](sdrr-rev-e-fab-notes.pdf)

## Notes

- Now supports STM32F405 and STM32F415 variants, in addition to the STM32F401 and STM32F411 variants.

## Changelog

- Consolidated and improved 401/411 vs 405/415 support - all variants are supported on the same board with minimal component changes required between variants.
- All passives are 0603.
- Renumbered components.
- Corrected VCAP_1 value for F401/F411 to 4.7uF (2.2uF causes F411 to lock up at 100MHz).
- Removed pull-downs on image select pins, to reduce component count (device uses internal pull-downs).
- Power LED replaced with status LED driven by PB15.
- Added 4th image select jumper (PB7), allowing up to 16 images to be selected.
- Limited silk-screen markings are provided.
