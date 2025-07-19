# Contains dynamically bank switchable C64 character sets
#
# Images:
# Set 0, Bank 0 - 4KB Character ROM, English, 901225-01
# Set 0, Bank 1 - 4KB Character ROM, Swedish, 325018-02
# Set 0, Bank 2 - 4KB Character ROM, Spanish, 325056-03
# Set 0, Bank 3 - 4KB Character ROM, Danish, 901225-01-DK
# Set 1, Bank 0 - 4KB Character ROM, English, 901225-01
# Set 1, Bank 1 - 4KB Character ROM, Turkish
# Set 1, Bank 2 - 4KB Character ROM, Croatian
# Set 1, Bank 3 - 4KB Character ROM, Japanese, 906143-02

ROM_CONFIGS = \
	set=0,bank=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.901225-01.bin,type=2332,cs1=0,cs2=1 \
	set=0,bank=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.325018-02.bin,type=2332,cs1=0,cs2=1 \
	set=0,bank=2,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.325056-03.bin,type=2332,cs1=0,cs2=1 \
	set=0,bank=3,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.901225-01-DK.bin,type=2332,cs1=0,cs2=1 \
	set=1,bank=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.901225-01.bin,type=2332,cs1=0,cs2=1 \
	set=1,bank=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.turkish.bin,type=2332,cs1=0,cs2=1 \
	set=1,bank=2,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/croatian.zip,extract=c64_cro/chargen,type=2332,cs1=0,cs2=1 \
	set=1,bank=3,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.906143-02.bin,type=2332,cs1=0,cs2=1