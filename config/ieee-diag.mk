# Contains IEEE-488 disk drive diagnostics ROMs from:
# https://github.com/piersfinlayson/cbm-ieee-disk-diag-rom
#
# Currently contains diagnostics ROM v0.2.1.
#
# Images:
# 0 - 2040/3040/4040 $F000 4KB diagnostics ROM
# 1 - 2040/3040/4040 $D000 4KB diagnostics ROM
# 2 - 8050/8250 $E000 8KB diagnostics ROM

ROM_CONFIGS = \
	file=https://github.com/piersfinlayson/cbm-ieee-disk-diag-rom/releases/download/v0.2.1/xx40_ieee_diag_f000.bin,type=2332,cs1=0,cs2=1 \
	file=https://github.com/piersfinlayson/cbm-ieee-disk-diag-rom/releases/download/v0.2.1/xx40_ieee_diag_d000.bin,type=2332,cs1=0,cs2=1 \
	file=https://github.com/piersfinlayson/cbm-ieee-disk-diag-rom/releases/download/v0.2.1/8x50_ieee_diag_e000.bin,type=2364,cs1=0
