# Serves a pair of stock ROMs for a PAL VIC20 - both kernal and basic ROMs
# SDDR cannot also serve the character ROM for the VIC20, as it has a different
# data bus to the kernal and basic ROMs.
#
# The VIC20 dead test kernal ROM is installed as set 1.
#
# Instructions:
# 1. Install ROM in (kernal) socket
# 2. Connect Basic ROM socket CS pin (20) to SDRR pin X1
# 3. Use SDRR image select jumpers to select required set, 0 = all open
#
# Image details:
#
# Set 0:
# a. Kernal ROM 901487-07
# b. Basic ROM 901486-01
#
# Set 1:
# a. Dead Test Kernal ROM, V1.1.01
#
# Notes:
# - Requires HW_REV=f
ROM_CONFIGS = \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/kernal.901486-07.bin,type=2364,cs1=0 \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/basic.901486-01.bin,type=2364,cs1=0 \
	set=1,file=https://github.com/piersfinlayson/Vic20-dead-test/releases/download/v1.1.01/dead-test.pal.e0,type=2364,cs1=0
