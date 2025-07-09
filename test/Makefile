# Quick test harness for ROM data validation
CC = gcc
CFLAGS = -Werror -Wall -Wextra -g -I include -I../sdrr/include -I../output
TARGET = image-test
BUILD_DIR = build
SOURCES = src/main.c src/check-roms.c src/load-config.c src/query-roms.c ../output/roms.c

# Create build directory and build target
$(BUILD_DIR)/$(TARGET): $(SOURCES) include/roms-test.h ../output/roms.h ../output/sdrr_config.h | $(BUILD_DIR)
	$(CC) $(CFLAGS) -o $@ $(SOURCES) -lcurl -lzip

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Convenience target
$(TARGET): $(BUILD_DIR)/$(TARGET)

# Clean up
clean:
	rm -rf $(BUILD_DIR)

# Run tests
test: $(BUILD_DIR)/$(TARGET)
	./$

# Force rebuild if output files change
../output/roms.c ../output/roms.h:
	@echo "ROM files missing - run sdrr-gen first"
	@false

.PHONY: clean test $(TARGET)
