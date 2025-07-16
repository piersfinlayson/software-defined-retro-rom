FROM piersfinlayson/sdrr-build:latest

# Metadata
LABEL org.opencontainers.image.title="SDRR"
LABEL org.opencontainers.image.description="Software Defined Retro ROM firmware and tools"
LABEL org.opencontainers.image.authors="Piers Finlayson <piers@piers.rocks>"
LABEL org.opencontainers.image.url="https://piers.rocks"
LABEL org.opencontainers.image.source="https://github.com/piersfinlayson/software-defined-retro-rom"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.vendor="piers.rocks"

# Build-time metadata
ARG BUILD_DATE
ARG VERSION
ARG VCS_REF
LABEL org.opencontainers.image.created=${BUILD_DATE}
LABEL org.opencontainers.image.version=${VERSION}
LABEL org.opencontainers.image.revision=${VCS_REF}

# SDRR build args
ARG CONFIG
ARG STM
ARG HW_REV
ARG ROM_CONFIGS
ARG SWD
ARG BOOT_LOGGING
ARG MAIN_LOOP_LOGGING
ARG MAIN_LOOP_ONE_SHOT
ARG DEBUG_LOGGING
ARG MCO
ARG MCO2
ARG OSC
ARG FREQ
ARG OVERCLOCK
ARG STATUS_LED
ARG BOOTLOADER
ARG DISABLE_PRELOAD_TO_RAM
ARG SERVE_ALG
ARG GEN_OUTPUT_DIR
ARG CARGO_PROFILE

# Clone the repository
RUN git clone https://github.com/piersfinlayson/software-defined-retro-rom.git
COPY config/docker/ /home/build/software-defined-retro-rom/config/docker/

# Build SDRR
WORKDIR /home/build/software-defined-retro-rom
WORKDIR /home/build/software-defined-retro-rom
RUN ${CONFIG:+CONFIG=$CONFIG} \
    ${STM:+STM=$STM} \
    ${HW_REV:+HW_REV=$HW_REV} \
    ${ROM_CONFIGS:+ROM_CONFIGS="$ROM_CONFIGS"} \
    ${SWD:+SWD=$SWD} \
    ${BOOT_LOGGING:+BOOT_LOGGING=$BOOT_LOGGING} \
    ${MAIN_LOOP_LOGGING:+MAIN_LOOP_LOGGING=$MAIN_LOOP_LOGGING} \
    ${MAIN_LOOP_ONE_SHOT:+MAIN_LOOP_ONE_SHOT=$MAIN_LOOP_ONE_SHOT} \
    ${DEBUG_LOGGING:+DEBUG_LOGGING=$DEBUG_LOGGING} \
    ${MCO:+MCO=$MCO} \
    ${MCO2:+MCO2=$MCO2} \
    ${OSC:+OSC=$OSC} \
    ${FREQ:+FREQ=$FREQ} \
    ${OVERCLOCK:+OVERCLOCK=$OVERCLOCK} \
    ${STATUS_LED:+STATUS_LED=$STATUS_LED} \
    ${BOOTLOADER:+BOOTLOADER=$BOOTLOADER} \
    ${DISABLE_PRELOAD_TO_RAM:+DISABLE_PRELOAD_TO_RAM=$DISABLE_PRELOAD_TO_RAM} \
    ${SERVE_ALG:+SERVE_ALG=$SERVE_ALG} \
    ${GEN_OUTPUT_DIR:+GEN_OUTPUT_DIR=$GEN_OUTPUT_DIR} \
    ${CARGO_PROFILE:+CARGO_PROFILE=$CARGO_PROFILE} \
    make test info-detail

# Copy build artifacts to output directory
RUN cp -r sdrr/build /home/build/output/

# Output detailed firmware information
RUN ELF_FILE=$(basename $(find sdrr/build -name "sdrr-stm32*.elf" -type f)) && \
    /home/build/software-defined-retro-rom/rust/sdrr-info/target/release/sdrr-info info -d $ELF_FILE >> /home/build/output/sdrr-info.txt

# Default to staying in the project directory
WORKDIR /home/build/software-defined-retro-rom
VOLUME ["/home/build/output"]
