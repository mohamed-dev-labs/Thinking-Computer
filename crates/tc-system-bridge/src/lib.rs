#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("system_bridge.h");

        fn platform_name() -> String;
        fn cpu_architecture() -> String;
        fn logical_cpu_count() -> u32;
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

/// Returns the native runtime's reported logical processor count.
pub fn logical_cpu_count() -> u32 {
    ffi::logical_cpu_count()
}
