# Contains dynamically bank switchable VIC-20 character sets
#
# Images:
# Set 0, Bank 0 - 4KB Character ROM, English, 901486-01
# Set 0, Bank 1 - 4KB Character ROM, Swedish/Finnish, NEC P22101-207
# Set 0, Bank 2 - 4KB Character ROM, Danish, 901460-03
# Set 0, Bank 3 - 4KB Character ROM, Japanese, 901460-02

ROM_CONFIGS = \
	set=0,bank=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.901460-03.bin,type=2332,cs1=0,cs2=0 \
	set=0,bank=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.NecP22101-207.bin,type=2332,cs1=0,cs2=0 \
	set=0,bank=2,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.DK_901460-03.bin,type=2332,cs1=0,cs2=0 \
	set=0,bank=3,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.901460-02.bin,type=2332,cs1=0,cs2=0