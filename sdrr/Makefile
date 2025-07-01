# STM32F103 Makefile

# To specify the target variant, use:
# ```bash
# make VARIANT=stm32f103x8
# ```
#
# Available variants:
# - stm32f103xb (default)
# - stm32f103x8

# sdrr-gen Output directory
OUTPUT_DIR := ../output

# Include the auto-generated Makefile fragment
# Only include generated.mk if we're not doing a clean target
ifeq ($(filter clean clean-% distclean,$(MAKECMDGOALS)),)
  ifneq ($(wildcard $(OUTPUT_DIR)/generated.mk),)
    include $(OUTPUT_DIR)/generated.mk
  else
$(error sddr-gen generated.mk Makefile fragment not found. Please run the sddr-gen tool first.)
  endif
endif

# Default variant - this is set in generated.mk
# VARIANT := stm32f103xb

# Build directory
BUILD_DIR := build

# Toolchain
TOOLCHAIN ?= /opt/arm-gnu-toolchain-14.2.rel1-x86_64-arm-none-eabi/bin
CC := $(TOOLCHAIN)/arm-none-eabi-gcc
OBJDUMP := $(TOOLCHAIN)/arm-none-eabi-objdump
OBJCOPY := $(TOOLCHAIN)/arm-none-eabi-objcopy
SIZE := $(TOOLCHAIN)/arm-none-eabi-size

# Source files
SRCS := $(wildcard src/*.c)
OBJS := $(addprefix $(BUILD_DIR)/,$(notdir $(SRCS:.c=.o)))

SEGGER_SRCS := segger-rtt/RTT/SEGGER_RTT.c segger-rtt/RTT/SEGGER_RTT_printf.c
SEGGER_OBJS := $(BUILD_DIR)/SEGGER_RTT.o $(BUILD_DIR)/SEGGER_RTT_printf.o

ROMS_SRC := $(OUTPUT_DIR)/roms.c
ROMS_OBJ := $(BUILD_DIR)/roms.o

# Output files
BIN_PREFIX := sdrr-$(VARIANT)
ELF := $(BUILD_DIR)/$(BIN_PREFIX).elf
BIN := $(BUILD_DIR)/$(BIN_PREFIX).bin
MAP := $(BUILD_DIR)/$(BIN_PREFIX).map
DISASM := $(BUILD_DIR)/$(BIN_PREFIX).dis

# Only set these variables if we're not doing a clean target
ifeq ($(filter clean clean-% distclean,$(MAKECMDGOALS)),)

# Linker script selection
ifeq ($(VARIANT),stm32f103r8)
    LDSCRIPT := stm32f103x8.ld
else ifeq ($(VARIANT),stm32f103rb)
    LDSCRIPT := stm32f103xb.ld
else ifeq ($(VARIANT),stm32f411rc)
	LDSCRIPT := stm32f411xc.ld
else ifeq ($(VARIANT),stm32f411re)
	LDSCRIPT := stm32f411xe.ld
else ifeq ($(VARIANT),stm32f401re)
	LDSCRIPT := stm32f401xe.ld
else ifeq ($(VARIANT),stm32f401rb)
	LDSCRIPT := stm32f401xb.ld
else ifeq ($(VARIANT),stm32f401rc)
	LDSCRIPT := stm32f401xc.ld
else ifeq ($(VARIANT),stm32f405rg)
	LDSCRIPT := stm32f405xg.ld
else
$(error Unsupported variant: $(VARIANT))
endif

# Flags
ifeq ($(findstring stm32f103,$(VARIANT)),stm32f103)
	COMMON_FLAGS := -mcpu=cortex-m3 -mthumb -mfloat-abi=soft 
else ifeq ($(findstring stm32f4,$(VARIANT)),stm32f4)
	COMMON_FLAGS := -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -ffast-math
#	COMMON_FLAGS := -mcpu=cortex-m4 -mthumb -mfloat-abi=soft
else
$(error Unsupported variant: $(VARIANT))
endif

endif

CFLAGS := $(COMMON_FLAGS) -Werror -I include -I $(OUTPUT_DIR) -I segger-rtt/RTT -I segger-rtt/Config -g -nostdlib -O3 -ffunction-sections -fdata-sections -Wall -Wextra -MMD -MD -MP
LDFLAGS := $(COMMON_FLAGS) -Werror -L link -nostdlib -specs=nosys.specs -specs=nano.specs -Wl,--no-gc-sections -Wl,--fatal-warnings -Wl,-Map=$(MAP)

# Phony targets
.PHONY: all clean clean-segger segger size flash run

# Default target
all: $(BIN) size

# Create build directory
$(BUILD_DIR):
	mkdir -p $@

# Clone the segger repo
segger-rtt:
	@if [ ! -d "$@" ]; then \
		git clone https://github.com/piersfinlayson/segger-rtt.git; \
	fi

# sdrr object files
$(BUILD_DIR)/%.o: src/%.c ../Makefile | $(BUILD_DIR) segger-rtt
	$(CC) $(CFLAGS) -c $< -o $@

# roms.o is built from roms.c from a different directory
$(ROMS_OBJ): $(OUTPUT_DIR)/roms.c ../Makefile 
	$(CC) $(CFLAGS) -c $< -o $@

# Explicit rules for SEGGER files
$(BUILD_DIR)/SEGGER_RTT.o: segger-rtt/RTT/SEGGER_RTT.c ../Makefile  | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/SEGGER_RTT_printf.o: segger-rtt/RTT/SEGGER_RTT_printf.c ../Makefile | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

segger-rtt/RTT/SEGGER_RTT.c segger-rtt/RTT/SEGGER_RTT_printf.c: segger-rtt

# Link
$(ELF): $(OBJS) $(ROMS_OBJ) $(SEGGER_OBJS) link/$(LDSCRIPT) | $(BUILD_DIR)
	$(CC) $(LDFLAGS) -T link/$(LDSCRIPT) $(OBJS) $(SEGGER_OBJS) $(ROMS_OBJ) -o $@

# Generate binary
$(BIN): $(ELF)
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -D -S -t -h $< > $(DISASM)
	@echo "Build complete: $@"

# Size information
size: $(ELF)
	$(SIZE) $(ELF)
	@ls -ltr $(BIN) 

# Flash target
flash: $(ELF)
	probe-rs download --chip STM32F103C8Tx $<

# Run target
run: $(ELF)
	probe-rs run --chip STM32F103C8Tx $<

# Clean segger
clean-segger-src:
	rm -rf segger-rtt/

# Clean build
clean-build:
	rm -fr $(BUILD_DIR)

# Clean
clean: clean-build clean-segger-src

# Include dependency files
-include $(BUILD_DIR)/*.d
