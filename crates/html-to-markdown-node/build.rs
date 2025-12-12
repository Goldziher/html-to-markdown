fn main() {
    napi_build::setup();

    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=advapi32");
}
