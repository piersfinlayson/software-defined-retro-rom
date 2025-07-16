# Docker

This directory contains Dockerfiles for building the SDRR project in a consistent environment.

* `Dockerfile.build` - creates `sdrr-build` container for building the SDRR project.
* `Dockerfile` - creates `sdrr` tooling and firmware, using `sdrr-build` container.

Uses:

* Ubuntu 24.04
* arm-gnu-toolchain-14.3.rel1-x86_64-arm-none
* Latest Rust and probe-rs versions

## `sdrr-build` Container

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

## `sdrr` Container

To build SDRR, create the sdrr container, from the repo root:

```bash
docker build \
    --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
    --build-arg VERSION=$(git describe --tags --always --dirty 2>/dev/null || echo "dev") \
    --build-arg VCS_REF=$(git rev-parse HEAD 2>/dev/null || echo "unknown") \
    -f ci/docker/Dockerfile \
    -t piersfinlayson/sdrr:latest .
```

To use non-default build configuration, pass in build arguments.  For example:

```bash
docker build \
    --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
    --build-arg VERSION=$(git describe --tags --always --dirty 2>/dev/null || echo "dev") \
    --build-arg VCS_REF=$(git rev-parse HEAD 2>/dev/null || echo "unknown") \
    --build-arg STM=f401re \
    --build-arg CONFIG=vic20-pal.mk \
    --build-arg HW_REV=24-f \
    -f ci/docker/Dockerfile \
    -t piersfinlayson/sdrr-f401re-24-f-vic20-pal:latest .
```
