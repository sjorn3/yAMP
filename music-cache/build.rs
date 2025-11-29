fn main() {
    println!("cargo:rerun-if-changed=tests/ffi_shim.c");
    println!("cargo:rerun-if-changed=music_cache.h");

    cc::Build::new()
        .file("tests/ffi_shim.c")
        .include(".")
        .compile("ffi_shim");
}
