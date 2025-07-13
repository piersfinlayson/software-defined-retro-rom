# SDRR firmware Makefile

# To build, use the top-level /Makefile, which generates the required files and
# configuration to build SDRR, then calls this Makefile.

# sdrr-gen Output directory
GEN_OUTPUT_DIR ?= output
OUTPUT_DIR := ../$(GEN_OUTPUT_DIR)

# Include the auto-generated Makefile fragment
# Only include generated.mk if we're not doing a clean target
ifeq ($(filter clean clean-% distclean,$(MAKECMDGOALS)),)
  ifneq ($(wildcard $(OUTPUT_DIR)/generated.mk),)
    include $(OUTPUT_DIR)/generated.mk
  else
$(error sdrr-gen generated.mk Makefile fragment not found. Please run the sdrr-gen tool first.)
  endif
endif

# Default variant - this is set in generated.mk
# VARIANT := stm32f411xb

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
SDRR_CONFIG_SRC := $(OUTPUT_DIR)/sdrr_config.c
SDRR_CONFIG_OBJ := $(BUILD_DIR)/sdrr_config.o

# Output files
BIN_PREFIX := sdrr-$(VARIANT)
ELF := $(BUILD_DIR)/$(BIN_PREFIX).elf
BIN := $(BUILD_DIR)/$(BIN_PREFIX).bin
MAP := $(BUILD_DIR)/$(BIN_PREFIX).map
DISASM := $(BUILD_DIR)/$(BIN_PREFIX).dis

# Only set these variables if we're not doing a clean target
ifeq ($(filter clean clean-% distclean,$(MAKECMDGOALS)),)

# Linker script selection
LDSCRIPT=$(OUTPUT_DIR)/linker.ld

# Flags
ifeq ($(findstring stm32f4,$(VARIANT)),stm32f4)
	COMMON_FLAGS := -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -ffast-math
#	COMMON_FLAGS := -mcpu=cortex-m4 -mthumb -mfloat-abi=soft
else
$(error Unsupported variant: $(VARIANT))
endif

endif

SEGGER_RTT_FLAGS := -DBUFFER_SIZE_UP=4096
CFLAGS := $(COMMON_FLAGS) $(SEGGER_RTT_FLAGS) -Werror -I include -I $(OUTPUT_DIR) -I segger-rtt/RTT -I segger-rtt/Config -g -nostdlib -O3 -ffunction-sections -fdata-sections -Wall -Wextra -MMD -MD -MP
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
	@echo "- Compiling sdrr/$<"
	@$(CC) $(CFLAGS) -c $< -o $@

# roms.o is built from roms.c from a different directory
$(ROMS_OBJ): $(ROMS_SRC) ../Makefile 
	@echo "- Compiling sdrr/$<"
	@$(CC) $(CFLAGS) -c $< -o $@

# sdrr_config.o is built from sdrr_config.c from a different directory
$(SDRR_CONFIG_OBJ): $(SDRR_CONFIG_SRC) ../Makefile 
	@echo "- Compiling sdrr/$<"
	@$(CC) $(CFLAGS) -DSDRR_VERSION_MAJOR=$(VERSION_MAJOR) -DSDRR_VERSION_MINOR=$(VERSION_MINOR) -DSDRR_VERSION_PATCH=$(VERSION_PATCH) -DSDRR_BUILD_NUMBER=$(BUILD_NUMBER) -DSDRR_GIT_COMMIT=\"$(GIT_COMMIT)\" -c $< -o $@

# Explicit rules for SEGGER files
$(BUILD_DIR)/SEGGER_RTT.o: segger-rtt/RTT/SEGGER_RTT.c ../Makefile  | $(BUILD_DIR)
	@echo "- Compiling sdrr/$<"
	@$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/SEGGER_RTT_printf.o: segger-rtt/RTT/SEGGER_RTT_printf.c ../Makefile | $(BUILD_DIR)
	@echo "- Compiling sdrr/$<"
	@$(CC) $(CFLAGS) -c $< -o $@

segger-rtt/RTT/SEGGER_RTT.c segger-rtt/RTT/SEGGER_RTT_printf.c: segger-rtt

# Link
$(ELF): $(OBJS) $(ROMS_OBJ) $(SDRR_CONFIG_OBJ) $(SEGGER_OBJS) $(LDSCRIPT) | $(BUILD_DIR)
	@echo "- Linking object files to create ELF -> sdrr/$@"
	@$(CC) $(LDFLAGS) -T $(LDSCRIPT) $(OBJS) $(SEGGER_OBJS) $(ROMS_OBJ) $(SDRR_CONFIG_OBJ) -o $@

# Generate binary
$(BIN): $(ELF)
	@echo "- Generating binary from sdrr/$< -> sdrr/$@"
	@$(OBJCOPY) -O binary $< $@
	@$(OBJDUMP) -D -S -t -h $< > $(DISASM)
	@echo "----"
	@echo "Build complete:"

# Size information
size: $(ELF)
	@$(SIZE) $(ELF)
	@ls -ltr $(ELF) 
	@ls -ltr $(BIN) 

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
