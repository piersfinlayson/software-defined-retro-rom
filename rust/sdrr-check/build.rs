// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT licence

fn main() {
    // Re-run this build script if anything in git changes.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");

    // Set up STM32 linking
    println!("cargo:rustc-link-arg=-v");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    // Re-run this build script of DEFMT_LOG changes.
    println!("cargo:rerun-if-env-changed=DEFMT_LOG");

    // Required for defmt
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
