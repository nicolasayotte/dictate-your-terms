fn main() {
    // whisper-rs-sys emits cargo:rustc-link-lib=libopenblas on Windows but does not
    // emit a corresponding cargo:rustc-link-search for the vcpkg lib directory.
    // We add it here so the Rust linker can resolve libopenblas.lib.
    #[cfg(target_os = "windows")]
    if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
        println!(
            "cargo:rustc-link-search=native={}/installed/x64-windows/lib",
            vcpkg_root
        );
    }
}
