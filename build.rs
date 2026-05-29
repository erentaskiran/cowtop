fn main() {
    cc::Build::new()
        .files([
            "csrc/cowsys.c",
            "csrc/proc_reader.c",
            "csrc/cow_net.c",
            "csrc/cow_disk.c",
        ])
        .include("csrc")
        .std("c11")
        .flag_if_supported("-pthread")
        // Names/paths longer than the fixed FFI buffers are intentionally
        // truncated by snprintf; that is safe, so quiet the noise.
        .flag_if_supported("-Wno-format-truncation")
        .warnings(true)
        .compile("cowsys");

    // Explicitly link the C static library - some cc crate / Cargo combinations
    // need the directive emitted from build.rs proper rather than via cc internals.
    println!("cargo:rustc-link-lib=static=cowsys");
    println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=csrc");
}
