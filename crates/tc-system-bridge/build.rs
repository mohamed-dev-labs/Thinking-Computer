fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("../../native/system-bridge/system_bridge.cc")
        .include("../../native/system-bridge")
        .flag_if_supported("-std=c++17")
        .compile("tc_system_bridge");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=../../native/system-bridge/system_bridge.h");
    println!("cargo:rerun-if-changed=../../native/system-bridge/system_bridge.cc");
}

