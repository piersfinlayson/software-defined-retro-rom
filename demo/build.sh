PETCAT="petcat"
C1541="c1541"

C64_BAS="intro-c64.bas"
C64_PRG="intro-c64.prg"
C64_OUT="sdrrc64"
VIC20_BAS="intro-vic20.bas"
VIC20_PRG="intro-vic20.prg"
VIC20_OUT="sdrrvic"
DISK_HDR="piers.rocks,01"
DISK_NAME="sdrr.d64"

rm *.prg *.d64 2> /dev/null
$PETCAT -w2 -o $C64_PRG -- $C64_BAS
$PETCAT -w2 -o $VIC20_PRG -- $VIC20_BAS
$C1541 -format "$DISK_HDR" d64 $DISK_NAME -write $C64_PRG $C64_OUT -write $VIC20_PRG $VIC20_OUT > /dev/null

echo "Created disk image: $DISK_NAME"