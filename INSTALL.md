# INSTALL

Get hold of a [supported microcontroller](README.md#supported-stm32-microcontrollers) and the [PCB](sdrr-pcb) and solder the PCB up.

Get rev E from here:

[![Order from OSH Park](https://oshpark.c   om/assets/badge-5b7ec47045b78aef6eb9d83b3bac6b1920de805e9a0c227658eac6e19a045b9c.png)](https://oshpark.com/shared_projects/9TJoAirm)

At this point, you can choose to install the dependencies locally, or use the [Docker container](ci/docker/README.md) to build the SDRR firmware.

1. Clone the repository:

    ```bash
    git clone https://github.com/piersfinlayson/software-defined-retro-rom.git
    cd software-defined-retro-rom
    ```

2. Install the required [ARM GNU toolchain](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads) for AArch32 bare-metal target (arm-none-eabi).

    Recommended approach - download the toolchain from ARM's developer site (this is quite large, so may take a while):

    ```bash
    wget https://developer.arm.com/-/media/Files/downloads/gnu/14.3.rel1/binrel/arm-gnu-toolchain-14.3.rel1-x86_64-arm-none-eabi.tar.xz
    tar -xvf arm-gnu-toolchain-14.3.rel1-x86_64-arm-none-eabi.tar.xz
    sudo mv arm-gnu-toolchain-14.3.rel1-x86_64-arm-none-eabi /opt/
    ```

    Or install it via your package manager, e.g., on Debian/Ubuntu:

    ```bash
    sudo apt install gcc-arm-none-eabi
    ```

    If you install using the package manager, you will need to update the `TOOLCHAIN` environment variable or variable in the [Makefile](sdrr/Makefile) to point to the correct compiler binary directory.  It should probably `/usr/bin` or similar.

3. Install curl, zip and json-x development packages (required for tests):

    ```bash
    sudo apt install libcurl4-openssl-dev libzip-dev libjson-c-dev
    ```

    If you are using a different package manager, the package name may vary slightly, e.g., `libcurl-devel` on Fedora.

4. Install [Rust](https://www.rust-lang.org/tools/install):

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
    ```

5. Install [probe-rs](https://probe.rs/) for flashing the firmware to the SDRR device.

    ```bash
    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
    probe-rs complete install
    ```

6. Connect up the SDRR device to your [programmer](README.md#programmer).

At this point you can follow the instructions in [Quick Start](README.md#quick-start) to build and flash the SDRR firmware.
