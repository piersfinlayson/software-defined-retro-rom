# Contains stock ROMs sets for 1540/1541 drives which use 8KB (2364) ROMs
#
# Sets:
# 0:
# - 1541/1540 $C000 8KB ROM, 325302-01
# - 1541 $E000 8KB ROM, 901229-06AA
# 1:
# - 1541/1540 $C000 8KB ROM, 325302-01
# - 1541 $E000 8KB ROM, 901229-05
# 2:
# - 1540 $E000 8KB ROM, 325303-01
# - 1541/1540 $C000 8KB ROM, 325302-01
#
# Notes:
# - The _lower_ ROM ($C000) is the first image in the set.  This means you
#   must install SDRR in the $C000 ROM slot (UB3), and connect X1 to pin 20
#   of the $E000 ROM slot (UB4).

ROM_CONFIGS = \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-c000.325302-01.bin,type=2364,cs1=0 \
	set=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-e000.901229-06AA.bin,type=2364,cs1=0 \
	set=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-c000.325302-01.bin,type=2364,cs1=0 \
	set=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-e000.901229-05.bin,type=2364,cs1=0 \
	set=2,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-c000.325302-01.bin,type=2364,cs1=0 \
	set=2,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1540-e000.325303-01.bin,type=2364,cs1=0
