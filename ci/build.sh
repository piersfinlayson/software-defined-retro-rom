#!/bin/bash

#
# build.sh - Build all MCU variant and config combinations for One ROM project
#
# This script automates building all combinations of MCU variants and configuration
# files for the software-defined-retro-rom project. It supports both CI builds
# (for automated testing), specific test builds (for build system validation), 
# and release builds (with packaging).
#
# Usage:
#   ci/build.sh ci              - Build all image configs
#   ci/build.sh test            - Test option combinations
#   ci/build.sh release v1.2.3  - Build and package for release
#   ci/build.sh clean           - Delete builds/ directory
#
# Configuration files:
#   ci/mcu-variants.txt         - List of MCU variants to build (one per line)
#   ci/blacklist-config.txt     - Configs to skip building (one per line)
#   ci/size-incompatible.txt    - Config+MCU combinations that don't fit in flash
#   ci/tests.txt                - Specific option combinations to test
#   config/*.mk                 - Individual config files to build
#
# Output structure:
#   builds/ci/                  - CI builds
#   builds/test/                - Test builds
#   builds/v1.2.3/              - Release builds
#

set -e  # Exit immediately on any command failure

# Determine script and project directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BLACKLIST_FILENAME="blacklist-config.txt"
MCU_VARIANTS_FILENAME="mcu-variants.txt"
SIZE_INCOMPATIBLE_FILENAME="size-incompatible.txt"
TESTS_FILENAME="tests.txt"
BLACKLIST_FILE="${SCRIPT_DIR}/${BLACKLIST_FILENAME}"
MCU_VARIANTS_FILE="${SCRIPT_DIR}/${MCU_VARIANTS_FILENAME}"
SIZE_INCOMPATIBLE_FILE="${SCRIPT_DIR}/${SIZE_INCOMPATIBLE_FILENAME}"
TESTS_FILE="${SCRIPT_DIR}/${TESTS_FILENAME}"

#
# Display usage information and exit
#
usage() {
    echo "Usage: $0 <command> [args]"
    echo "Commands:"
    echo "  ci                - Build firmware for all image configs"
    echo "  test              - Test option combinations"
    echo "  release <version> - Build and package for release (e.g. v1.2.3)"
    echo "  clean             - Delete builds/ directory"
    echo ""
    echo "Configuration files:"
    echo "  ci/${MCU_VARIANTS_FILENAME}      - MCU variants to build"
    echo "  ci/${BLACKLIST_FILENAME}  - Configs to skip"
    echo "  ci/${SIZE_INCOMPATIBLE_FILENAME} - Size-incompatible combinations"
    echo "  ci/${TESTS_FILENAME}             - Specific test combinations"
    exit 1
}

#
# Remove the entire builds/ directory
#
clean_builds() {
    echo "Cleaning builds directory..."
    rm -rf "${PROJECT_ROOT}/builds"
    echo "Done."
}

#
# Get the correct binary prefix for an MCU variant
# Args: mcu_variant
# Returns: correct binary prefix (e.g. "sdrr-stm32f401re" or "sdrr-rp2350")
#
get_bin_prefix() {
    local mcu="$1"
    if [[ "$mcu" =~ ^f.* ]]; then
        echo "sdrr-stm32${mcu}"
    else
        echo "sdrr-${mcu}"
    fi
}

#
# Load MCU variants from mcu-variants.txt file
# Returns: Array of variant names (one per line)
#
load_mcu_variants() {
    local variants=()
    if [[ ! -f "${MCU_VARIANTS_FILE}" ]]; then
        echo "ERROR: MCU variants file not found: ${MCU_VARIANTS_FILE}"
        exit 1
    fi
    
    while IFS= read -r line; do
        # Skip empty lines and comments (lines starting with #)
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
        variants+=("$line")
    done < "${MCU_VARIANTS_FILE}"
    
    if [[ ${#variants[@]} -eq 0 ]]; then
        echo "ERROR: No MCU variants found in ${MCU_VARIANTS_FILE}"
        exit 1
    fi
    
    printf '%s\n' "${variants[@]}"
}

#
# Parse MCU variant line to extract MCU name and environment variables
# Args: mcu_variant_line (format: "mcu_name" or "mcu_name:VAR1=val1,VAR2=val2")
# Returns: Prints "mcu_name" on first line, then env vars on subsequent lines
#
parse_mcu_variant() {
    local variant_line="$1"
    
    if [[ "$variant_line" =~ : ]]; then
        # Parse MCU name and env vars
        IFS=':' read -r mcu_name env_vars <<< "$variant_line"
        echo "$mcu_name"
        
        if [[ -n "$env_vars" ]]; then
            IFS=',' read -ra var_pairs <<< "$env_vars"
            for var_pair in "${var_pairs[@]}"; do
                echo "$var_pair"
            done
        fi
    else
        # Just MCU name, no env vars
        echo "$variant_line"
    fi
}

#
# Load blacklisted configs from blacklist-config.txt file
# Returns: Array of config names to skip building
#
load_blacklist() {
    local blacklist=()
    if [[ -f "${BLACKLIST_FILE}" ]]; then
        while IFS= read -r line; do
            # Skip empty lines and comments (lines starting with #)
            [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
            blacklist+=("$line")
        done < "${BLACKLIST_FILE}"
    fi
    printf '%s\n' "${blacklist[@]}"
}

#
# Load size-incompatible combinations from size-incompatible.txt file
# Returns: Lines in format "config_name mcu1 mcu2 ..."
#
load_size_incompatible() {
    local incompatible=()
    if [[ -f "${SIZE_INCOMPATIBLE_FILE}" ]]; then
        while IFS= read -r line; do
            # Skip empty lines and comments (lines starting with #)
            [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
            incompatible+=("$line")
        done < "${SIZE_INCOMPATIBLE_FILE}"
    fi
    printf '%s\n' "${incompatible[@]}"
}

#
# Check if a config+MCU combination is size-incompatible
# Args: config_name, mcu_variant, incompatible_lines_array...
# Returns: 0 if incompatible, 1 if compatible
#
is_size_incompatible() {
    local config="$1"
    local mcu="$2"
    local incompatible_lines=("${@:3}")
    
    for line in "${incompatible_lines[@]}"; do
        # Parse line: first word is config, rest are MCU variants
        read -ra parts <<< "$line"
        local line_config="${parts[0]}"
        local line_mcus=("${parts[@]:1}")
        
        # Check if this line matches our config
        if [[ "$config" == "$line_config" ]]; then
            # Check if our MCU variant is in the incompatible list
            for incompatible_mcu in "${line_mcus[@]}"; do
                if [[ "$mcu" == "$incompatible_mcu" ]]; then
                    return 0  # Found incompatible combination
                fi
            done
        fi
    done
    return 1  # Not incompatible
}

#
# Check if a config is in the blacklist
# Args: config_name, blacklist_array...
# Returns: 0 if blacklisted, 1 if not
#
is_blacklisted() {
    local config="$1"
    local blacklist_array=("${@:2}")
    
    for blacklisted in "${blacklist_array[@]}"; do
        if [[ "$config" == "$blacklisted" ]]; then
            return 0  # Found in blacklist
        fi
    done
    return 1  # Not blacklisted
}

#
# Discover all available config files in config/ directory
# Returns: Array of config names (without .mk extension)
#
discover_configs() {
    local configs=()
    for mk_file in "${PROJECT_ROOT}/config"/*.mk; do
        if [[ -f "$mk_file" ]]; then
            config_name=$(basename "$mk_file" .mk)
            configs+=("$config_name")
        fi
    done
    
    if [[ ${#configs[@]} -eq 0 ]]; then
        echo "ERROR: No config files found in ${PROJECT_ROOT}/config/"
        exit 1
    fi
    
    printf '%s\n' "${configs[@]}"
}

#
# Load test combinations from tests.txt file
# Returns: Lines in format "test_name:MCU=variant,VAR1=val1,CONFIG=config/file.mk"
#
load_tests() {
    local tests=()
    if [[ ! -f "${TESTS_FILE}" ]]; then
        echo "ERROR: Tests file not found: ${TESTS_FILE}"
        exit 1
    fi
    
    while IFS= read -r line; do
        # Skip empty lines and comments (lines starting with #)
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
        tests+=("$line")
    done < "${TESTS_FILE}"
    
    if [[ ${#tests[@]} -eq 0 ]]; then
        echo "ERROR: No tests found in ${TESTS_FILE}"
        exit 1
    fi
    
    printf '%s\n' "${tests[@]}"
}

#
# Build a single MCU variant + config combination
# Args: mcu_variant, config_name
# Returns: 0 on success, 1 on failure
#
build_combination() {
    local mcu_line="$1"
    local config="$2"
    local config_file="config/${config}.mk"
    
    # Parse MCU variant line
    mapfile -t parsed < <(parse_mcu_variant "$mcu_line")
    local mcu="${parsed[0]}"
    local env_vars=("${parsed[@]:1}")
    
    echo "Building MCU=${mcu} CONFIG=${config_file}"
    if [[ ${#env_vars[@]} -gt 0 ]]; then
        echo "  with env vars: ${env_vars[*]}"
    fi
    
    # Clean firmware and generated files, but preserve Rust generator to avoid rebuild
    make clean-firmware-build clean-gen > /dev/null 2>&1 || true

    # Retry logic: Attempt the build up to 2 times if it fails
    local attempt=1
    local max_attempts=2
    local success=0

    while [[ $attempt -le $max_attempts ]]; do
        echo "    - Attempt $attempt: Building MCU=${mcu} CONFIG=${config_file}"
        
        # Build make command with env vars
        local make_cmd="MCU=\"$mcu\" CONFIG=\"$config_file\""
        for env_var in "${env_vars[@]}"; do
            make_cmd="$env_var $make_cmd"
        done
        make_cmd="$make_cmd make"
        
        if eval "$make_cmd" > /dev/null; then
            success=1
            break
        else
            echo "Build failed on attempt $attempt"
        fi
        attempt=$((attempt + 1))
    done

    if [[ $success -eq 0 ]]; then
        echo "ERROR: Build failed after $max_attempts attempts"
        return 1
    fi

    # Verify that all expected output files were created
    local build_dir="${PROJECT_ROOT}/sdrr/build"
    local base_name="$(get_bin_prefix "$mcu")"
    
    for ext in bin dis elf map; do
        if [[ ! -f "${build_dir}/${base_name}.${ext}" ]]; then
            echo "ERROR: Expected file ${base_name}.${ext} not found"
            return 1
        fi
    done
    
    return 0
}

#
# Execute a single test combination
# Args: test_line (format: "test_name:MCU=variant,VAR1=val1,CONFIG=config/file.mk")
# Returns: 0 on success, 1 on failure
#
execute_test() {
    local test_line="$1"
    local test_num="$2"

    # Parse test name and variables
    IFS=':' read -r test_name test_vars <<< "$test_line"
    
    # Parse environment variables
    local env_vars=()
    local mcu_variant=""
    local config_param=""
    
    if [[ -n "$test_vars" ]]; then
        IFS=',' read -ra var_pairs <<< "$test_vars"
        for var_pair in "${var_pairs[@]}"; do
            if [[ "$var_pair" =~ ^MCU= ]]; then
                mcu_variant="${var_pair#MCU=}"
            elif [[ "$var_pair" =~ ^CONFIG= ]]; then
                config_param="$var_pair"
            else
                env_vars+=("$var_pair")
            fi
        done
    fi
    
    # Validate MCU variant is specified
    if [[ -z "$mcu_variant" ]]; then
        echo "ERROR: Test '$test_name' missing MCU variant"
        return 1
    fi
    
    echo "- Running test $test_num: $test_name (MCU=$mcu_variant)"
    
    # Clean previous build artifacts
    make clean-firmware-build clean-gen > /dev/null 2>&1 || true
    
    # Build command with environment variables
    local make_cmd="MCU=$mcu_variant"
    for env_var in "${env_vars[@]}"; do
        make_cmd="$env_var $make_cmd"
    done
    if [[ -n "$config_param" ]]; then
        make_cmd="$make_cmd $config_param"
    fi
    echo "  - Command: $make_cmd make test"
    make_cmd="$make_cmd make test"
    
    # Retry logic: Attempt the test up to 2 times if it fails
    local attempt=1
    local max_attempts=2
    local success=0

    while [[ $attempt -le $max_attempts ]]; do
        echo "    - Attempt $attempt: Executing test '$test_name'"
        if eval "$make_cmd" > /dev/null; then
            success=1
            break
        else
            echo "Test '$test_name' failed on attempt $attempt"
        fi
        attempt=$((attempt + 1))
    done

    if [[ $success -eq 0 ]]; then
        echo "ERROR: Test '$test_name' failed after $max_attempts attempts"
        return 1
    fi

    return 0
}

#
# Move build artifacts from sdrr/build/ to organized directory structure
# Args: mcu_variant, config_name, target_base_dir
#
organize_files() {
    local mcu="$1"
    local config="$2"
    local target_dir="$3"
    
    local build_dir="${PROJECT_ROOT}/sdrr/build"
    local base_name="$(get_bin_prefix "$mcu")"
    local dest_dir="${target_dir}/${mcu}/${config}"
    
    # Create destination directory structure
    mkdir -p "$dest_dir"
    
    # Copy all build artifacts to organized location
    for ext in bin dis elf map; do
        cp "${build_dir}/${base_name}.${ext}" "$dest_dir/"
    done
}

#
# Create zip files for release distribution
# Args: version_string
#
create_zips() {
    local version="$1"
    local builds_dir="${PROJECT_ROOT}/builds/${version}"
    
    echo "Creating zip files..."
    
    # Create firmware and full-output directories
    mkdir -p "${builds_dir}/firmware"
    mkdir -p "${builds_dir}/full-output"
    
    # Load MCU variants
    mapfile -t mcu_variants < <(load_mcu_variants)

    for mcu_line in "${mcu_variants[@]}"; do
        # Parse MCU name from variant line
        mapfile -t parsed_mcu < <(parse_mcu_variant "$mcu_line")
        local mcu="${parsed_mcu[0]}"
        
        if [[ -d "${builds_dir}/${mcu}" ]]; then
            # Create firmware-only zip (just .elf files, renamed with config names)
            cd "${builds_dir}/${mcu}"
            for config_dir in */; do
                config_name=$(basename "$config_dir")
                local bin_prefix="$(get_bin_prefix "$mcu")"
                if [[ -f "${config_dir}${bin_prefix}.elf" ]]; then
                    zip -j "${builds_dir}/firmware/elf-${mcu}.zip" "${config_dir}${bin_prefix}.elf" > /dev/null
                    # Rename the file in the zip to use config name
                    (cd /tmp && unzip -q "${builds_dir}/firmware/elf-${mcu}.zip" "${bin_prefix}.elf" && mv "${bin_prefix}.elf" "${config_name}-${mcu}.elf" && zip -u "${builds_dir}/firmware/elf-${mcu}.zip" "${config_name}-${mcu}.elf" && zip -d "${builds_dir}/firmware/elf-${mcu}.zip" "${bin_prefix}.elf")                fi
            done
            echo "Created firmware/elf-${mcu}.zip"

            # Create bin-only zip (just .bin files, renamed with config names)
            for config_dir in */; do
                config_name=$(basename "$config_dir")
                if [[ -f "${config_dir}${bin_prefix}.bin" ]]; then
                    zip -j "${builds_dir}/firmware/bin-${mcu}.zip" "${config_dir}${bin_prefix}.bin" > /dev/null
                    # Rename the file in the zip to use config name
                    (cd /tmp && unzip -q "${builds_dir}/firmware/bin-${mcu}.zip" "${bin_prefix}.bin" && mv "${bin_prefix}.bin" "${config_name}-${mcu}.bin" && zip -u "${builds_dir}/firmware/bin-${mcu}.zip" "${config_name}-${mcu}.bin" && zip -d "${builds_dir}/firmware/bin-${mcu}.zip" "${bin_prefix}.bin")                fi
            done
            echo "Created firmware/bin-${mcu}.zip"
            
            # Create complete zip with full structure  
            cd "${builds_dir}"
            zip -r "full-output/full-${mcu}.zip" "${mcu}" > /dev/null
            echo "Created full-output/full-${mcu}.zip"
        fi
    done
    
    # Return to project root
    cd "${PROJECT_ROOT}"
}

#
# Main script execution logic
#
main() {
    # Require at least one argument
    [[ $# -lt 1 ]] && usage
    
    local command="$1"
    
    # Handle different command modes
    case "$command" in
        clean)
            clean_builds
            exit 0
            ;;
        ci)
            local target_dir="${PROJECT_ROOT}/builds/ci"
            local mode="ci"
            ;;
        test)
            local target_dir="${PROJECT_ROOT}/builds/test"
            local mode="test"
            ;;
        release)
            # Release mode requires version argument
            [[ $# -ne 2 ]] && usage
            local version="$2"
            local target_dir="${PROJECT_ROOT}/builds/${version}"
            local mode="release"
            ;;
        *)
            usage
            ;;
    esac
    
    # Change to project root directory for make commands
    cd "${PROJECT_ROOT}"
    
    # Execute based on mode
    if [[ "$mode" == "test" ]]; then
        # Test mode: Execute specific test combinations and organize files for verification

        # Initial clean to ensure we start with a clean slate
        echo "Performing initial clean..."
        make clean > /dev/null 2>&1 || true
    
        mkdir -p "$target_dir"
        
        mapfile -t tests < <(load_tests)
        
        echo "Found tests: ${#tests[@]}"
        echo "First test will take longer as it builds sdrr-gen"
        echo ""
        
        local total_tests=0
        local successful_tests=0
        
        for test_line in "${tests[@]}"; do
            total_tests=$((total_tests + 1))
            
            if execute_test "$test_line" "$total_tests"; then
                # Test succeeded, organize the output files for verification
                # Extract MCU variant from test line for organizing files
                IFS=':' read -r test_name test_vars <<< "$test_line"
                local mcu_variant=""
                if [[ -n "$test_vars" ]]; then
                    IFS=',' read -ra var_pairs <<< "$test_vars"
                    for var_pair in "${var_pairs[@]}"; do
                        if [[ "$var_pair" =~ ^MCU= ]]; then
                            mcu_variant="${var_pair#MCU=}"
                            break
                        fi
                    done
                fi
                organize_files "$mcu_variant" "$test_name" "$target_dir"
                successful_tests=$((successful_tests + 1))
            else
                echo "ERROR: Test failed, stopping"
                exit 1
            fi
        done
        
        # Display test results summary
        echo ""
        echo "Test summary: ${successful_tests}/${total_tests} successful"
        echo "Test build validation complete"
        echo "Files organized in: $target_dir"
        
    elif [[ "$mode" == "release" ]]; then
        # Release mode: Package existing CI build
        local ci_dir="${PROJECT_ROOT}/builds/ci"
        
        # Verify CI build exists
        if [[ ! -d "$ci_dir" ]]; then
            echo "ERROR: No CI build found at ${ci_dir}. Run 'ci/build.sh ci' first."
            exit 1
        fi
        
        # Remove existing release directory and copy from CI
        echo "Copying CI build to release directory..."
        rm -rf "$target_dir"

        # Verify CI build exists
        if [[ ! -d "$ci_dir" ]]; then
            echo "ERROR: No CI build found at ${ci_dir}. Run 'ci/build.sh ci' first."
            exit 1
        fi
        cp -r "$ci_dir" "$target_dir"
        
        # Create zip files
        create_zips "$version"
        echo "Release ${version} packaging complete"
        echo "Firmware zips: ${target_dir}/firmware/*-firmware.zip"
        echo "Complete zips: ${target_dir}/full-output/*-all.zip"
        
    else
        # CI mode: Build all combinations

        # Initial clean to ensure we start with a clean slate
        echo "Performing initial clean..."
        make clean > /dev/null 2>&1 || true

        # Ensure target directory exists
        mkdir -p "$target_dir"
        
        # Load configuration data
        mapfile -t blacklist < <(load_blacklist)
        mapfile -t configs < <(discover_configs)
        mapfile -t mcu_variants < <(load_mcu_variants)
        mapfile -t size_incompatible < <(load_size_incompatible)
        
        # Display configuration summary
        echo "Found MCU variants: ${mcu_variants[*]}"
        echo "Found configs: ${configs[*]}"
        if [[ ${#blacklist[@]} -gt 0 ]]; then
            echo "Blacklisted configs: ${blacklist[*]}"
        fi
        echo ""
        
        # Build all valid combinations
        local total_combinations=0
        local successful_builds=0
        local blacklisted_skips=0
        local size_incompatible_skips=0
        
        for mcu_line in "${mcu_variants[@]}"; do
            # Parse MCU name from variant line
            mapfile -t parsed_mcu < <(parse_mcu_variant "$mcu_line")
            local mcu="${parsed_mcu[0]}"
            
            for config in "${configs[@]}"; do
                total_combinations=$((total_combinations + 1))
                
                # Skip blacklisted configurations
                if is_blacklisted "$config" "${blacklist[@]}"; then
                    echo "Skipping blacklisted config: $config (MCU: $mcu)"
                    blacklisted_skips=$((blacklisted_skips + 1))
                    continue
                fi
                
                # Skip size-incompatible combinations
                if is_size_incompatible "$config" "$mcu" "${size_incompatible[@]}"; then
                    echo "Skipping size-incompatible combination: $config + $mcu"
                    size_incompatible_skips=$((size_incompatible_skips + 1))
                    continue
                fi
                
                # Attempt to build this combination
                if build_combination "$mcu_line" "$config"; then
                    # Build succeeded, organize the output files
                    organize_files "$mcu" "$config" "$target_dir"
                    successful_builds=$((successful_builds + 1))
                else
                    echo "ERROR: Build failed for MCU=${mcu} CONFIG=${config}"
                    exit 1
                fi
            done
        done
        
        # Display build results summary
        local attempted_builds=$((total_combinations - blacklisted_skips - size_incompatible_skips))
        echo ""
        echo "Build summary:"
        echo "  Total combinations: ${total_combinations}"
        echo "  Blacklisted skips: ${blacklisted_skips}"
        echo "  Size-incompatible skips: ${size_incompatible_skips}"
        echo "  Attempted builds: ${attempted_builds}"
        echo "  Successful builds: ${successful_builds}"
        echo "CI build complete"
        echo "Files organized in: $target_dir"
    fi
}

# Execute main function with all script arguments
main "$@"