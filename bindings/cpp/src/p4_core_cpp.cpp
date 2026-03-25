#include "p4_core_cpp.hpp"

#include <cstdlib>
#include <filesystem>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>

#if defined(_WIN32)
#include <windows.h>
#else
#include <dlfcn.h>
#include <limits.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <mach-o/dyld.h>
#endif
#endif

namespace p4 {
namespace {

using fn_p4_generate_identity_json = char *(*)();
using fn_p4_peer_id_from_public_key_b64 = char *(*)(const char *);
using fn_p4_last_error_message = char *(*)();
using fn_p4_free_string = void (*)(char *);

std::mutex g_path_mutex;
std::string g_explicit_library_path;

std::string platform_dir() {
#if defined(_WIN32)
  return "win32-x64";
#elif defined(__APPLE__) && (defined(__aarch64__) || defined(__arm64__))
  return "darwin-arm64";
#elif defined(__APPLE__)
  return "darwin-x64";
#else
  return "linux-x64";
#endif
}

std::string library_name() {
#if defined(_WIN32)
  return "p4_core.dll";
#elif defined(__APPLE__)
  return "libp4_core.dylib";
#else
  return "libp4_core.so";
#endif
}

bool file_exists(const std::filesystem::path &path) {
  std::error_code ec;
  return std::filesystem::exists(path, ec) && !ec;
}

std::filesystem::path executable_dir() {
#if defined(_WIN32)
  char buffer[MAX_PATH];
  DWORD len = GetModuleFileNameA(nullptr, buffer, MAX_PATH);
  if (len == 0 || len >= MAX_PATH) {
    throw std::runtime_error("failed to determine executable path");
  }
  return std::filesystem::path(std::string(buffer, len)).parent_path();
#elif defined(__APPLE__)
  uint32_t size = 0;
  _NSGetExecutablePath(nullptr, &size);
  std::vector<char> buffer(size + 1, 0);
  if (_NSGetExecutablePath(buffer.data(), &size) != 0) {
    throw std::runtime_error("failed to determine executable path");
  }
  return std::filesystem::path(buffer.data()).parent_path();
#else
  char buffer[PATH_MAX];
  ssize_t len = readlink("/proc/self/exe", buffer, sizeof(buffer) - 1);
  if (len <= 0) {
    throw std::runtime_error("failed to determine executable path");
  }
  buffer[len] = '\0';
  return std::filesystem::path(buffer).parent_path();
#endif
}

std::string resolve_default_library_path() {
  {
    std::lock_guard<std::mutex> lock(g_path_mutex);
    if (!g_explicit_library_path.empty()) {
      return g_explicit_library_path;
    }
  }

  const char *env_path = std::getenv("P4_CORE_LIB");
  if (env_path != nullptr && env_path[0] != '\0') {
    return std::string(env_path);
  }

  const auto rel_native_path =
      std::filesystem::path("native") / "p4_core" / platform_dir() / library_name();

  std::vector<std::filesystem::path> candidates;
  candidates.push_back(std::filesystem::current_path() / rel_native_path);

#ifdef P4_NATIVE_ROOT
  candidates.push_back(std::filesystem::path(P4_NATIVE_ROOT) / platform_dir() / library_name());
#endif

  try {
    candidates.push_back(executable_dir() / rel_native_path);
  } catch (const std::exception &) {
    // Ignore executable path lookup failures; other candidates may still work.
  }

  for (const auto &candidate : candidates) {
    if (file_exists(candidate)) {
      return candidate.string();
    }
  }

  throw std::runtime_error(
      "P4 native library not found. Set P4_CORE_LIB or place native binaries under "
      "native/p4_core/<platform>/");
}

class NativeApi {
 public:
  NativeApi() {
    const std::string lib_path = resolve_default_library_path();
#if defined(_WIN32)
    handle_ = LoadLibraryA(lib_path.c_str());
    if (handle_ == nullptr) {
      throw std::runtime_error("failed to load native library: " + lib_path);
    }
    load_symbol(p4_generate_identity_json, "p4_generate_identity_json");
    load_symbol(p4_peer_id_from_public_key_b64, "p4_peer_id_from_public_key_b64");
    load_symbol(p4_last_error_message, "p4_last_error_message");
    load_symbol(p4_free_string, "p4_free_string");
#else
    handle_ = dlopen(lib_path.c_str(), RTLD_NOW);
    if (handle_ == nullptr) {
      throw std::runtime_error(std::string("failed to load native library: ") + dlerror());
    }
    load_symbol(p4_generate_identity_json, "p4_generate_identity_json");
    load_symbol(p4_peer_id_from_public_key_b64, "p4_peer_id_from_public_key_b64");
    load_symbol(p4_last_error_message, "p4_last_error_message");
    load_symbol(p4_free_string, "p4_free_string");
#endif
  }

  ~NativeApi() {
#if defined(_WIN32)
    if (handle_ != nullptr) {
      FreeLibrary(static_cast<HMODULE>(handle_));
    }
#else
    if (handle_ != nullptr) {
      dlclose(handle_);
    }
#endif
  }

  NativeApi(const NativeApi &) = delete;
  NativeApi &operator=(const NativeApi &) = delete;

  fn_p4_generate_identity_json p4_generate_identity_json = nullptr;
  fn_p4_peer_id_from_public_key_b64 p4_peer_id_from_public_key_b64 = nullptr;
  fn_p4_last_error_message p4_last_error_message = nullptr;
  fn_p4_free_string p4_free_string = nullptr;

 private:
  void *handle_ = nullptr;

  template <typename T>
  void load_symbol(T &out, const char *name) {
#if defined(_WIN32)
    FARPROC sym = GetProcAddress(static_cast<HMODULE>(handle_), name);
    if (sym == nullptr) {
      throw std::runtime_error(std::string("missing symbol in P4 core: ") + name);
    }
    out = reinterpret_cast<T>(sym);
#else
    void *sym = dlsym(handle_, name);
    if (sym == nullptr) {
      throw std::runtime_error(std::string("missing symbol in P4 core: ") + name);
    }
    out = reinterpret_cast<T>(sym);
#endif
  }
};

NativeApi &api() {
  static NativeApi instance;
  return instance;
}

std::string last_error() {
  char *err = api().p4_last_error_message();
  if (err == nullptr) {
    return "unknown error";
  }
  std::string message(err);
  api().p4_free_string(err);
  return message;
}

std::string take_string(char *ptr) {
  if (ptr == nullptr) {
    throw std::runtime_error(last_error());
  }
  std::string value(ptr);
  api().p4_free_string(ptr);
  return value;
}

}  // namespace

void set_library_path(const std::string &path) {
  std::lock_guard<std::mutex> lock(g_path_mutex);
  g_explicit_library_path = path;
}

std::string resolve_library_path() { return resolve_default_library_path(); }

std::string generate_identity_json() { return take_string(api().p4_generate_identity_json()); }

std::string peer_id_from_public_key_b64(const std::string &public_key_b64) {
  return take_string(api().p4_peer_id_from_public_key_b64(public_key_b64.c_str()));
}

}  // namespace p4


