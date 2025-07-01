# Contains stock ROMs for a BASIC 4 PET 40 column 50 Hz version
#
# Images:
# 0 - $F000 4KB Kernal ROM, 901465-22
# 1 - $E000 4KB Edit ROM, 901498-01 (2 copies)
# 2 - $D000 4KB Basic ROM, 901465-21
# 3 - $C000 4KB Basic ROM, 901465-20
# 4 - $B000 4KB Basic ROM, 901465-19
# 5 - 4KB Character ROM, 901447-10 (2 copies)
# 6 - $F000 4KB Diagnostics ROM
# 7 - Simple PET test program, can be loaded to any address - if loaded to $9000 execute like this `SYS 36864`

ROM_CONFIGS = \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/pet/kernal-4.901465-22.bin,type=2332,cs1=0,cs2=1 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/pet/edit-4-40-n-50Hz.901498-01.bin,type=2332,cs1=0,cs2=1,dup \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/pet/basic-4-d000.901465-21.bin,type=2332,cs1=0,cs2=1 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/pet/basic-4-c000.901465-20.bin,type=2332,cs1=0,cs2=1 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/pet/basic-4-b000.901465-19.bin,type=2332,cs1=0,cs2=1 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/pet/characters-2.901447-10.bin,type=2332,cs1=0,cs2=1,dup \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/misc/petdiag/40col_diag_20_10_1_f000.bin,type=2332,cs1=0,cs2=1 \
	file=https://github.com/piersfinlayson/pet-ieee-loader/releases/download/v0.1.1/test.bin,type=2332,cs1=0,cs2=1,pad
