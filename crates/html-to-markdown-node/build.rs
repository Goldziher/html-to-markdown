fn main() {
    napi_build::setup();

    // On Windows, mimalloc requires advapi32 for privilege escalation functions
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=advapi32");
}
