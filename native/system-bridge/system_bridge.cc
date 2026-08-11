#include "system_bridge.h"

#include <thread>

rust::String platform_name() {
#if defined(_WIN32)
  return "windows";
#elif defined(__APPLE__)
  return "macos";
#elif defined(__linux__)
  return "linux";
#else
  return "unknown";
#endif
}

rust::String cpu_architecture() {
#if defined(__aarch64__) || defined(_M_ARM64)
  return "aarch64";
#elif defined(__x86_64__) || defined(_M_X64)
  return "x86_64";
#else
  return "unknown";
#endif
}

std::uint32_t logical_cpu_count() {
  const auto count = std::thread::hardware_concurrency();
  return count == 0 ? 1 : count;
}
