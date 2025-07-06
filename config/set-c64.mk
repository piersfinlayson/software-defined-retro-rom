# Serves a set of stock ROMs for a C64 - all of kernal, basic and character
# ROMs - when installed in the kernal socket.
#
# Instructions:
# 1. Install ROM in (kernal) socket
# 2. Connect Basic ROM socket CS pin (20) to SDRR pin X1
# 3. Connect Character ROM socket CS1 pin (20) to SDRR pin X2
# 4. Use SDRR image select jumpers to select required set, 0 = all open
#
# Image details:
#
# Set 0:
# a. Kernal ROM 901227-03, Basic ROM, 901
# b. Basic ROM 901226-01
# c. Character ROM 901225-01
#
# Notes:
# - Character ROM CS2 is set to `ignore` as is it tied to 5V within the C64,
#   so always active 
# - Requires HW_REV=f
ROM_CONFIGS = \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/kernal.901227-03.bin,type=2364,cs1=0 \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/basic.901226-01.bin,type=2364,cs1=0 \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.901225-01.bin,type=2332,cs1=0,cs2=ignore