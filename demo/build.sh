PETCAT="petcat"
C1541="c1541"

C64_BAS="intro-c64.bas"
C64_PRG="intro-c64.prg"
C64_BAS2="one-c64.bas"
C64_PRG2="one-c64.prg"
C64_OUT="sdrrc64"
C64_OUT2="sdrr2c64"
C64_ASM3="rpc64.asm"
C64_OBJ3="rpc64.o"
C64_PRG3="rpc64.prg"
C64_OUT3="rpc64"
VIC20_BAS="intro-vic20.bas"
VIC20_PRG="intro-vic20.prg"
VIC20_OUT="sdrrvic"
DISK_HDR="piers.rocks,01"
DISK_NAME="one.d64"

rm *.prg *.d64 2> /dev/null
$PETCAT -w2 -o $C64_PRG -- $C64_BAS
$PETCAT -w2 -o $C64_PRG2 -- $C64_BAS2
$PETCAT -w2 -o $VIC20_PRG -- $VIC20_BAS
ca65 $C64_ASM3 -o $C64_OBJ3
ld65 -C c64.cfg $C64_OBJ3 -o $C64_PRG3
$C1541 -format "$DISK_HDR" d64 $DISK_NAME -write $C64_PRG $C64_OUT -write $C64_PRG2 $C64_OUT2 -write $C64_PRG3 $C64_OUT3 -write $VIC20_PRG $VIC20_OUT >> /dev/null

echo "Created disk image: $DISK_NAME"