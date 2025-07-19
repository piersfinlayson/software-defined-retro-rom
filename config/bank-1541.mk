# Contains stock dynamically bank switchable upper ($E000) kernal ROMs for the
# 1540/1541 drives which use 8KB (2364) ROMs
#
# The second set contains just a single image - the lower ($C000) ROM 
#
# Images:
# Set 0 - Bank 0 - 1541 $E000 8KB ROM, 901229-06AA
# Set 0 - Bank 1 - 1541 $E000 8KB ROM, 901229-05
# Set 0 - Bank 2 - 1541 $E000 8KB ROM, 325303-01
# Set 1 - 1541/1540 $C000 8KB ROM, 325302-01

ROM_CONFIGS = \
	set=0,bank=0,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-e000.901229-06AA.bin,type=2364,cs1=0 \
	set=0,bank=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-e000.901229-05.bin,type=2364,cs1=0 \
	set=0,bank=2,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1540-e000.325303-01.bin,type=2364,cs1=0 \
	set=1,file=http://www.zimmers.net/anonftp/pub/cbm/firmware/drives/new/1541/1541-c000.325302-01.bin,type=2364,cs1=0
