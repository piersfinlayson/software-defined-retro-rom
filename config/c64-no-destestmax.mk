# Contains stock ROMs for a C64, without DesTestMAX (as that requires accepting
# a licence)
#
# Images:
# 0 - 8KB Kernal ROM, 901227-03
# 1 - 8KB Basic ROM, 901226-01
# 2 - 4KB Character ROM, 901225-01
# 3 - 8KB Dead Test Kernal ($E000) ROM, 781220
# 4 - 8KB DesTestMAX Kernal ($E000) ROM - https://factorofmatt.com/destestmax

ROM_CONFIGS = \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/kernal.901227-03.bin,type=2364,cs1=0 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/basic.901226-01.bin,type=2364,cs1=0 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/characters.901225-01.bin,type=2332,cs1=0,cs2=1 \
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/misc/c64diag/c64_diag-rev781220-pn314139-03.zip,extract=Diag_DeadTest/dead%20test.BIN,type=2364,cs1=0
