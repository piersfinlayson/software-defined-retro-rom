# Docker

This directory contains Dockerfiles for building One ROM in a consistent environment.

* `Dockerfile.build` - creates `sdrr-build` container for building One ROM.
* `Dockerfile` - creates `sdrr` tooling and firmware, using `sdrr-build` container.

Uses:

* Ubuntu 24.04
* arm-gnu-toolchain-14.3.rel1-x86_64-arm-none
* Latest Rust and probe-rs versions

## Building One ROM

Either build via the [`sdrr` container](#building-sdrr-via-sdrr-container) or use the `sdrr-build` container directly, like this:

```bash
docker run --name sdrr-builder piersfinlayson/sdrr-build:latest sh -c '
git clone https://github.com/piersfinlayson/software-defined-retro-rom.git && \
cd software-defined-retro-rom && \
STM=f401re HW_REV=24-f CONFIG=config/vic20-pal.mk make test info-detail
'
docker cp sdrr-builder:/home/build/software-defined-retro-rom/sdrr/build/sdrr-stm32f401re.elf /tmp/
docker rm sdrr-builder
```

You can build additional configurations using the same container (i.e. not running `docker rm sdrr-builder`).  This avoids re-cloning the repo, and also avoids rebuilding all of the Rust tooling, which takes most of the build time.

## Building `sdrr-build` Container

To create the sdrr-build container, from the repo root:

```bash
docker buildx build \
    --platform linux/amd64 \
    --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
    --build-arg VERSION=$(git describe --tags --always --dirty 2>/dev/null || echo "dev") \
    --build-arg VCS_REF=$(git rev-parse HEAD 2>/dev/null || echo "unknown") \
    -t piersfinlayson/sdrr-build:latest \
    -f ci/docker/Dockerfile.build .
```

## Building One ROM via `sdrr` Container

To build One ROM, create the sdrr container, from the repo root:

```bash
docker build \
    -f ci/docker/Dockerfile \
    -t piersfinlayson/sdrr:latest .
```

To use non-default build configuration, pass in build arguments.  For example:

```bash
docker build \
    --build-arg STM=f401re \
    --build-arg CONFIG=vic20-pal.mk \
    --build-arg HW_REV=24-f \
    -f ci/docker/Dockerfile \
    -t piersfinlayson/sdrr-f401re-24-f-vic20-pal:latest .
```
