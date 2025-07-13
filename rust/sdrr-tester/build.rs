//! This build script handles:
//! - Building our assembly code
//! - Copying `memory.x` to the output directory to allow the firmware to be
//!   created.
//!

// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT licensed - see https://opensource.org/license/MIT

// memory.x handling sourced from from embassy-rs examples.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Re-run this build script if anything in git changes.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");

    //
    // Handle asm.c
    //

    // Tell cargo to recompile if assembly changes
    println!("cargo:rerun-if-changed=src/asm.s");
    
    // Compile assembly file
    cc::Build::new()
        .file("src/asm.s")
        .flag("-mcpu=cortex-m0plus")
        .flag("-mthumb")
        .compile("sdr-tester-asm");

    // Re-run this build script of DEFMT_LOG changes.
    println!("cargo:rerun-if-env-changed=DEFMT_LOG");

    //
    // Handle memory.x
    //

    let memory_x = {
        println!("cargo:rerun-if-changed=memory.x");
        include_bytes!("memory.x")
    };

    // Put `memory.x` in our output directory and ensure it's on the linker
    // search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memory_x)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // Set embassy linker arguments for the binary.
    println!("cargo:rustc-link-arg=-v");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tdevice.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
}