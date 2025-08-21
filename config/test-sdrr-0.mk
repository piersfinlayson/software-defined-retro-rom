# Contains various ROM images to enable testing the SDRR firmware and build
# process.
#
# Images:
# 0 - 8KB ROM filled with 0x00
# 1 - 8KB ROM filled with repeating 0x00-0x3F
# 2 - 4KB ROM filled with repeating 0x00-0x3F, CS2 active high
# 3 - 4KB ROM filled with repeating 0x00-0x3F, CS2 active low
# 4 - 2KB ROM filled with repeating 0x00-0x3F, CS2/CS3 active high
# 5 - 8KB ROM filled with 0xFF, CS active high
# 6 - 4KB ROM containing a simple PET test program, can be located at any address, CS2 active high, SYS <START ADDRESS> to run

ROM_CONFIGS = \
	file=images/test/zero8192.rom,type=2364,cs1=0 \
	file=images/test/0_63_8192.rom,type=2364,cs1=0 \
	file=images/test/0_63_4096.rom,type=2332,cs1=0,cs2=1 \
	file=images/test/0_63_4096.rom,type=2332,cs1=0,cs2=0 \
	file=images/test/0_63_2048.rom,type=2316,cs1=0,cs2=1,cs3=1 \
	file=images/test/ff8192.rom,type=2332,cs1=1 \
	file=https://github.com/piersfinlayson/pet-ieee-loader/releases/download/v0.1.1/test.bin,type=2332,cs1=0,cs2=1,pad
