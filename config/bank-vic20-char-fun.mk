# Contains a variety of C64 character sets, but with VIC-20 character ROM CS settings
#
# Images:
# Set 0, Bank 0 - Original VIC-20 character ROM
# Set 0, Bank 1 - Original C64 character ROM
# Set 0, Bank 2 - Custom Apple II character ROM
# Set 0, Bank 3 - Custom ZX Spectrum character ROM
# Set 1, Bank 0 - Original VIC-20 character ROM
# Set 1, Bank 1 - Custom Aniron character ROM (Tolkein inspired)
# Set 1, Bank 2 - Custom Aurebesh character ROM (Star Wars inspired)
# Set 1, Bank 3 - Custom Minecraft character ROM
# Set 2, Bank 0 - Custom Comic Sans character ROM
# Set 2, Bank 1 - Custom Kirby Forgotten Land character ROM
# Set 2, Bank 2 - Custom Hachicro character ROM (Futuristic)
# Set 2, Bank 3 - Custom Kauno character ROM (Caligraphic)

ROM_CONFIGS = \
	set=0,bank=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.901460-03.bin,type=2332,cs1=0,cs2=0 \
	set=0,bank=1,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/original/c64.bin,type=2332,cs1=0,cs2=0 \
	set=0,bank=2,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/apple_ii.bin,type=2332,cs1=0,cs2=0 \
	set=0,bank=3,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/zx_spectrum.bin,type=2332,cs1=0,cs2=0 \
	set=1,bank=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/vic20/characters.901460-03.bin,type=2332,cs1=0,cs2=0 \
	set=1,bank=1,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/aniron.bin,type=2332,cs1=0,cs2=0 \
	set=1,bank=2,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/aurebesh.bin,type=2332,cs1=0,cs2=0 \
	set=1,bank=3,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/minecraft.bin,type=2332,cs1=0,cs2=0 \
	set=2,bank=0,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/comic_sans.bin,type=2332,cs1=0,cs2=0	\
	set=2,bank=1,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/hachicro.bin,type=2332,cs1=0,cs2=0 \
	set=2,bank=2,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/kauno.bin,type=2332,cs1=0,cs2=0 \
	set=2,bank=3,file=https://raw.githubusercontent.com/patrickmollohan/c64-fonts/refs/heads/master/custom/kirby_forgotten_land.bin,type=2332,cs1=0,cs2=0
