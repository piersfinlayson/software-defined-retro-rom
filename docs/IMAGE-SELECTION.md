# Image Selection

As above, up to 16 ROM images can be stored on the SDRR device, and selected at boot time using the jumpers on the PCB.  The SDRR image select jumpers are labelled om the underside `1`, `2`, `4` and `8`.  The jumper being open is a `0` and closed is a `1`.

Do not confuse the image select jumpers with the BOOT0 jumper, labelled `B0`.  This is provided to assist with [problems flashing](/README.md#flashing-problems).

The images with the SDRR firmware are numbered from 0 onwards, so, to select image 5 (the 6th image), you would close jumpers `1` and `4`.

If you select a higher image number than there are images installed, SDRR will wrap back to the first image and keep counting.  So, if you only have 4 images installed (as numbers 0-3), a jumper setting of 4 will select image 0, a jumper setting of 5 will select image 1, etc.

You must reboot the SDRR device after changing the jumper settings, to cause SDRR to load the new image and settings, and start serving it.  You may need to physically power your retro system off and on again to do this - a reset may not be enough if power is not removed.

If you are developing a ROM image and reflashing it [in-situ](/README#flashing-in-situ), it is recommended to use the same image index on a reflash, so when the SDRR device resets after reflashing, the correct image will be reloaded automatically.  This avoids the need for a host device power cycle (although a host device reset may be required, depending on what type of ROM you are providing).
