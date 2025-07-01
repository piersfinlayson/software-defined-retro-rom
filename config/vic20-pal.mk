# Contains stock ROMs for a PAL VIC-20
#
# Images:
# 0 - 8KB Kernal ROM, 901486-07
# 1 - 8KB Basic ROM, 901486-01
# 2 - 4KB Character ROM, 901460-03
# 3 - 8KB Dead Test Kernal ROM, V1.1

ROM_CONFIGS = \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/kernal.901486-07.bin,type=2364,cs1=0 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/basic.901486-01.bin,type=2364,cs1=0 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.901460-03.bin,type=2332,cs1=0,cs2=0 \
	file=https://github.com/piersfinlayson/Vic20-dead-test/releases/download/v1.1/dead-test.pal.e0,type=2364,cs1=0
