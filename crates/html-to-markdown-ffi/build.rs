#![allow(missing_docs)]

//! Build script for generating C bindings.
//!
//! Uses cbindgen to generate C header files from Rust FFI definitions.

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir).join("html_to_markdown.h");

    cbindgen::generate(&crate_dir)
        .expect("Unable to generate C bindings")
        .write_to_file(&output_file);

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
