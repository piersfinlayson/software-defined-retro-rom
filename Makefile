# Top level Makefile for SDRR project
#
# Running this Makefile will generate the SDRR firmware binary in sdrr/build
# according to the settings below.
#
# You can override the settings in this file by creating a config file which
# sets them, and including it when running make like this:
#
# CONFIG=configs/your_config.mk make
#
# You can also override some of the below settings in a config file, and others
# on the command line, like this:
#
# STM=f411rc CONFIG=configs/your_config.mk make
#

# Allow specifying config file to override the below settings
CONFIG ?= configs/blank.mk
-include $(CONFIG)

#
# Settings
#
# Use these to specify your hardware variant and the ROM images and
# configuration.
#

# STM types
# f446rc - F446 256KB flash version
# f446re - F446 512KB flash version
# f411rc - F411 256KB flash version
# f411re - F411 512KB flash version
# f405rg - F405 1024KB flash version
# f401re - F401 512KB flash version
# f401rb - F401 128KB flash version
# f401rc - F401 256KB flash version
# f103r8 - F103 64KB flash version (no longer supported)
# f103rb - F103 128KB flash version (no longer supported)

# STM ?= f446rc
# STM ?= f446re
# STM ?= f411rc
# STM ?= f411re
# STM ?= f405rg
# STM ?= f401re
# STM ?= f401rb
# STM ?= f401rc
STM ?= f411re

# HW revision
#
# F103 boards - a, b, c 
# F4xx boards - d, e, f

# HW_REV ?= a (no longer supported)
# HW_REV ?= b (no longer supported)
# HW_REV ?= c (no longer supported)
# HW_REV ?= d
# HW_REV ?= e
# HW_REV ?= f
HW_REV ?= d

# ROM configurations - each ROM can have its own type and CS settings
#
# Format: file=path_or_url,type=XXXX,cs1=X[,cs2=X][,cs3=X][,dup|pad]
#
# dup - used where the provided image is two small for the specified ROM type,
#       and the image should be duplicated to fill the ROM
# pad - used where the provided image is too large for the specified ROM type,
#       and the image should be padded (after the image) to fill the ROM
#
# Examples:
#   Single 2364 ROM: file=images/rom1.rom,type=2364,cs1=0
#   Single 2332 ROM: file=images/rom2.rom,type=2332,cs1=1,cs2=0  
#   Single 2316 ROM: file=images/rom3.rom,type=2316,cs1=0,cs2=1,cs3=0
#   Multiple ROMs: file=images/rom1.rom,type=2364,cs1=0 file=images/rom2.rom,type=2332,cs1=1,cs2=0
#   URL: file=https://example.com/rom1.rom,type=2364,cs1=0
#
# See `config` for more extensive examples.

ROM_CONFIGS ?= \
	file=images/test/0_63_4096.rom,type=2332,cs1=0,cs2=1

#
# Development and debug settings
#
# It is recommended to leave these settings as is unless you are developing or
# debugging the SDRR firmware.
#
# However, if you are having problems with the SDRR firmware, you may want to
# disable any enabled features.
#

# SWD - Serial Wire Debug protocol
#
# This is used to communicate with the STM32 chip for debugging and
# programming.
#
# If you are using an STM32F4xx series chip it is recommended to leave SWD
# enabled, as it makes it easier to reprogram the chip with new images.
#
# If you are using an STM32F1xx series chip, you will want to disable SWD
# unless you are debugging, because the SWD pins are shared with the ROM's
# data pins, hence two of the data lines won't work, and invalid data will be
# read from the ROM.
#
# SWD is required for the logging options to work.

# SWD ?= 0
SWD ?= 1

# Boot logging configuration
#
# Outputs diagnostics boot messages via SWD during the boot process.
#
# This is generally safe to leave enabled, but it slows down the boot process.
# Boot takes ~300us (STM32F405) without this enabled, and ~2.5ms with it.
# If you are using the ROM as a boot ROM in a machine with a very short reset
# timer, you may want to disable this.
#
# SWD is required for this option to work.

# BOOT_LOGGING ?= 0
BOOT_LOGGING ?= 1

# Main loop logging configuration
#
# This option outputs logs after every byte is retrieved from the ROM.  While
# the main loop that processes the chip select and retrieves ROM data remanins
# functional, there is a gap between the CS line being released, and the ROM
# detecting the next CS activation - to make these logs.
#
# Therefore, with this option, the ROM will not meet its timing requirements,
# and it is not recommended to use this option unless you are debugging the
# ROM code.
#
# SWD and BOOT_LOGGING are required for this option to work.

MAIN_LOOP_LOGGING ?= 0
#MAIN_LOOP_LOGGING ?= 1

# Debug logging
#
# More extensive logging than the boot and main loop logging options.
#
# May overwhelm the SWD interface, meaning logs are dropped.
#
# This is not recommended for normal operation, but may be useful for debugging

DEBUG_LOGGING ?= 0
#DEBUG_LOGGING ?= 1

# MCO - Microcontroller Clock Output
#
# The MCO options output STM32 clocks on specific pins:
# - MCO (STM32F1xx)/MCO1 (STM32F4xx) is output on pin PA8.
# - MCO2 (STM32F4xx) is output on pin PC9.
#
# This is useful to debugging the clock configuration of the STM32, and 
# ensuring it is being clocked at the correct frequency.
#
# MCO is used for MCO on STM32F1xx series chips, and MCO1 on STM32F4xx series
# chips.
# MCO2 is used for MCO2 on STM32F4xx series chips.
#
# On the STM32F1xx chip, PA8 is used for one of the ROM data lines, so enabling
# MCO should only be done during debugging.
#
# On the STM32F4xx chip, PA8 is not used for ROM data, so MCO can be enabled
# relatively safely - although having a high frequency signal on the board may
# cause interference with other sigansl.
#
# MCO2 (STM32F4xx only) uses PC9, which is an address line, so enabling this
# should only be done during debugging.
#
# MCO2 may only be enabled when MCO is enabled.

MCO ?= 0
# MCO ?= 1
MCO2 ?= 0
# MCO2 ?= 1

# Oscillator
# 
# Which oscillator to use.  The options are:
# - HSI (High Speed Internal) - the internal oscillator
# - HSE (High Speed External) - an external crystal or clock source (no longer supported)
#
# SDRR has been designed to work primrily with the internal oscillator, in
# order to reduce component count and cost (and a more stable clock source is
# not required for this application).  Hence HSI operation is recommended.

OSC ?= HSI
# OSC ?= HSE

# Frequency
#
# Target frequency of the processor - only valid for STM32F4xx series chips.
# Leave blank to use the max frequency of the chip:
# - F401: 84MHz
# - F411: 100MHz
# - F405: 168MHz
# - F446: 180MHz
#
# Some guidance on speeds required to run stably:
# - PET 4032 character ROM >= 26MHz
# - PET 4032 kernal ROM >= 26MHz
# - C64 PAL kernal ROM >= 75MHz
# - C64 PAL character ROM >= 98MHz
# - VIC 20 PAL kernal ROM >= 37MHz
# - VIC 20 PAL basic ROM >= 37MHz
# - VIC 20 PAL character ROM >= 72MHz
#
# Your mileage may vary.  Faster machines will require the device to operate
# at higher frequencies.  Lower clock frequencies will cause SDRR to draw less
# power.  The F405 draws around 30mA at 30MHz and >60mA at 168MHz.  The SDRR
# voltage regulator also consumes some power.
#
# It is suggested to leave this blank (i.e. use the maximum frequency) for most
# applications, and only dial it down if you are trying to reduce power
# consumption.

# FREQ ?= 30

# Overclocking
#
# Can be used to allow the STM32F4xx chip to be run at a higher frequency than
# officially supported.  Use with caution, as this may damage the chip.
#
# You must manually set the desired frequency using the FREQ variable.

OVERCLOCK ?= 0
# OVERCLOCK ?= 1

# Status LED
#
# The status LED is used to indicate the status of the SDRR device.
# Only supported from HW revision e onwards.
#
# You can disable it in order to reduce the power consumption of the device.
# With a 1K resistor, this is likely to save around 5-10mW of power.
#
# 1 is enabled, 0 disabled.

STATUS_LED ?= 0
# STATUS_LED ?= 1

# Bootloader
#
# When enabled, the STM32 device will enter its built in bootloader on boot
# when all the select jumpers are closed (high).
#
# This may be useful for reprogramming the device if it has locked up and SWD
# has become unresponsive - although it shouldn't.

BOOTLOADER ?= 0
# BOOTLOADER ?= 1

# Disable preload to RAM
#
# By default, the ROM image to be served is preloaded to RAM.  Setting this
# option disables that.  It speeds up startup by a fraction of a ms, at the
# cost of (presumably) serving the ROM data taking longer, as it has to be
# looked up from the flash, instead of RAM - and the flash lookup will have
# wait states, and the prefetch cache won't help, as this access is random.
# It might even slow down the instruction prefetch.
#
# Put a different way, you probably don't want to enable this.

DISABLE_PRELOAD_TO_RAM ?= 0
# DISABLE_PRELOAD_TO_RAM ?= 1

# Byte serving algorithm
#
# Byte default, the SDRR firmware checks for CS line changes twice as often as
# addr lines - due to the asymetric timings for these lines.
#
# When serving mulit-ROM sets, the address lines are only checked when CS goes
# active - as SDRR doesn't know which ROM image to pull the byte from until
# that happens.
#
# Another algorithm can be chosen for the single image/set case.  It has no
# effect in the mutli image set case.
#
# default = a
# a = check CS twice as often as address lines
# b = check CS and address as frequently as each other

SERVE_ALG ?= default
#SERVE_ALG ?= a
#SERVE_ALG ?= b

# Output directory
#
# Directory to store intermediate files which are autogenerated during the
# build process.
GEN_OUTPUT_DIR ?= output

#
# End of settings
#

COLOUR_YELLOW := $(shell echo -e '\033[33m')
COLOUR_RESET := $(shell echo -e '\033[0m')
COLOUR_RED := $(shell echo -e '\033[31m')

$(info ==========================================)
$(info SDRR Makefile env settings:)

ifneq ($(CONFIG),)
  $(info - CONFIG=$(CONFIG))
endif

# Set hardware revision flag
ifneq ($(HW_REV),)
  $(info - HW_REV=$(HW_REV))
  HW_REV_FLAG = --hw-rev $(HW_REV)
else
  $(info - $(COLOUR_RED)HW_REV not set$(COLOUR_RESET) - please set it to a valid value, e.g. d or e)
  exit 1
endif

# Set swd flag based on SWD variable
ifeq ($(SWD), 1)
  $(info - SWD=$(SWD))
  SWD_FLAG = --swd
else
  SWD_FLAG =
endif

# Set boot logging flag based on BOOT_LOGGING variable
ifeq ($(BOOT_LOGGING), 1)
  $(info - BOOT_LOGGING=$(BOOT_LOGGING))
  BOOT_LOGGING_FLAG = --boot-logging
else
  BOOT_LOGGING_FLAG = 
endif

# Set main loop logging flag based on MAIN_LOOP_LOGGING variable
ifeq ($(MAIN_LOOP_LOGGING), 1)
  $(info - MAIN_LOOP_LOGGING=$(MAIN_LOOP_LOGGING))
  MAIN_LOOP_LOGGING_FLAG = --main-loop-logging
else
  MAIN_LOOP_LOGGING_FLAG = 
endif

# Set debug logging flag based on DEBUG_LOGGING variable
ifeq ($(DEBUG_LOGGING), 1)
  $(info - DEBUG_LOGGING=$(DEBUG_LOGGING))
  DEBUG_LOGGING_FLAG = --debug-logging
else
  DEBUG_LOGGING_FLAG =
endif

# Set mco flag based on MCO variable
ifeq ($(MCO), 1)
  $(info - MCO=$(MCO))
  MCO_FLAG = --mco
else
  MCO_FLAG = 
endif

# Set mco2 flag based on MCO2 variable
ifeq ($(MCO2), 1)
  $(info - MCO2=$(MCO2))
  MCO2_FLAG = --mco2
else
  MCO2_FLAG =
endif

# Set oscillator flag based on OSC variable
ifeq ($(OSC), HSE)
  $(info - OSC=$(OSC))
  OSC_FLAG = --hse
else
  OSC_FLAG = --hsi
endif

# Set FREQ_FLAG based on FREQ variable
ifeq ($(FREQ),)
  FREQ_FLAG =
else
  $(info - FREQ=$(FREQ))
  FREQ_FLAG = --freq $(FREQ)
endif

# Set OVERCLOCK_FLAG based on OVERCLOCK variable
ifeq ($(OVERCLOCK), 1)
  $(info - OVERCLOCK=$(OVERCLOCK))
  OVERCLOCK_FLAG = --overclock
else
  OVERCLOCK_FLAG =
endif

# Set STATUS_LED_FLAG based on STATUS_LED variable
ifeq ($(STATUS_LED), 1)
  $(info - STATUS_LED=$(STATUS_LED))
  STATUS_LED_FLAG = --status-led
else
  STATUS_LED_FLAG = 
endif

# Set BOOTLOADER_FLAG based on BOOTLOADER variable
ifeq ($(BOOTLOADER), 1)
  $(info - BOOTLOADER=$(BOOTLOADER))
  BOOTLOADER_FLAG = --bootloader
else
  BOOTLOADER_FLAG =
endif

# Set DISABLE_PRELOAD_TO_RAM_FLAG based on DISABLE_PRELOAD_TO_RAM variable
ifeq ($(DISABLE_PRELOAD_TO_RAM), 1)
  $(info - "DISABLE_PRELOAD_TO_RAM=$(DISABLE_PRELOAD_TO_RAM)")
  DISABLE_PRELOAD_TO_RAM_FLAG = --disable-preload-to-ram
else
  DISABLE_PRELOAD_TO_RAM_FLAG =
endif

# Set SERVE_ALG_FLAG based on SERVE_ALG variable
ifeq ($(SERVE_ALG), b)
  $(info - SERVE_ALG=$(SERVE_ALG))
  SERVE_ALG_FLAG = --serve-alg b
else ifeq ($(SERVE_ALG), a)
  $(info - SERVE_ALG=$(SERVE_ALG))
  SERVE_ALG_FLAG = --serve-alg a
else ifeq ($(SERVE_ALG), default)
  $(info - SERVE_ALG=$(SERVE_ALG))
  SERVE_ALG_FLAG =
else
  $(info - $(COLOUR_RED)Invalid SERVE_ALG value$(COLOUR_RESET) - please set it to a valid value, e.g. a or b)
  exit 1
endif

ifneq ($(ARGS),)
$(info - ARGS=$(ARGS))
endif

ifeq ($(ROM_CONFIGS),)
  $(info - $(COLOUR_RED)ROM_CONFIGS not set$(COLOUR_RESET) - please set it to a valid value, or use the CONFIG variable to include a config file)
  exit 1
else
  $(info - ROM_CONFIGS=$(ROM_CONFIGS))
endif

$(info ==========================================)

WARNINGS=0

$(info SDRR Build Warnings: )
ifneq ($(SWD), 1)
  $(info - $(COLOUR_YELLOW)SWD is disabled$(COLOUR_RESET) - this will prevent you from reprogramming the device via SWD after flashing with this image)
  WARNINGS += 1
endif

ifeq ($(WARNINGS),0)
  $(info - None)
endif

$(info ==========================================)

# Convert ROM_CONFIGS to --rom arguments, adjusting local file paths only
#
# HTTP/HTTPS URLs are used as-is
# Absolute file paths (starting with /) are used as-is  
# Relative file paths get ../ prepended
define process_rom_config
$(if $(findstring http://,$(1)),
    $(1),
    $(if $(findstring https://,$(1)),
        $(1),
        $(if $(filter /%,$(patsubst file=%,%,$(1))),
            $(1),
            $(subst file=,file=../,$(1))
        )
    )
)
endef
ROM_ARGS = $(foreach config,$(ROM_CONFIGS),--rom $(strip $(call process_rom_config,$(config))))
$(info ROM_ARGS = $(ROM_ARGS))

.PHONY: all clean clean-firmware clean-firmware-build clean-gen clean-sdrr-gen sdrr-gen gen firmware run test

all: firmware

sdrr-gen:
	cd sdrr-gen && cargo build

gen: sdrr-gen
	mkdir -p $(GEN_OUTPUT_DIR)
	cd sdrr-gen && cargo run -- --stm $(STM) $(HW_REV_FLAG) $(OSC_FLAG) $(ROM_ARGS) $(SWD_FLAG) $(BOOT_LOGGING_FLAG) $(MAIN_LOOP_LOGGING_FLAG) $(DEBUG_LOGGING_FLAG) $(MCO_FLAG) $(MCO2_FLAG) $(FREQ_FLAG) $(OVERCLOCK_FLAG) $(STATUS_LED_FLAG) $(BOOTLOADER_FLAG) $(DISABLE_PRELOAD_TO_RAM_FLAG) $(SERVE_ALG_FLAG) $(ARGS) --overwrite --output ../$(GEN_OUTPUT_DIR)

firmware: gen
	+cd sdrr && make OUTPUT_DIR=../$(GEN_OUTPUT_DIR)
	ls -ltr sdrr/build/*.bin

# Call make run-actual - this causes a new instance of make to be invoked and generated.mk exists, so it can load PROBE_RS_CHIP_ID
run: firmware
	make run-actual

test: firmware
	+cd test && make
	ROM_CONFIGS="$(ROM_CONFIGS)" test/build/image-test

-include $(GEN_OUTPUT_DIR)/generated.mk

run-actual:
	probe-rs run --chip $(PROBE_RS_CHIP_ID) sdrr/build/sdrr-stm32$(STM).elf

clean-firmware-build:
	+cd sdrr && make clean-build

clean-firmware:
	+cd sdrr && make clean

clean-gen:
	rm -fr $(GEN_OUTPUT_DIR)

clean-sdrr-gen:
	cd sdrr-gen && cargo clean

clean: clean-firmware clean-gen clean-sdrr-gen
