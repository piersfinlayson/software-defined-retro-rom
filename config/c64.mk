# Contains stock ROMs for a C64
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
	file=http://www.zimmers.net/anonftp/pub/cbm/firmware/misc/c64diag/c64_diag-rev781220-pn314139-03.zip,extract=Diag_DeadTest/dead%20test.BIN,type=2364,cs1=0 \
	file=https://img1.wsimg.com/blobby/go/66231677-2ba7-409f-8dee-bbd44bbf8f37/downloads/destest-max-v0.1.zip?ver=1750944987245,extract=destest-max.rom,licence=https://factorofmatt.com/destestmax%2Fdestestmax-sl,type=2364,cs1=0
