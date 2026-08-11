#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("system_bridge.h");

        fn platform_name() -> String;
        fn cpu_architecture() -> String;
    }
}

/// Returns the operating-system identifier from a tiny, auditable C++ bridge.
pub fn platform_name() -> String {
    ffi::platform_name()
}

/// Returns the target CPU architecture from the native bridge.
pub fn cpu_architecture() -> String {
    ffi::cpu_architecture()
}

