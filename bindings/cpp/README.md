# C++ SDK Wrapper

This directory has:
- `include/pp2p_core_cpp.hpp` thin C++ wrapper API
- `src/pp2p_core_cpp.cpp` implementation over `pp2p_core.h`
- `CMakeLists.txt` for build/link

The C++ wrapper dynamically loads the PP2P native library and auto-resolves
bundled binaries under `native/pp2p_core/<platform>/`.

## Build example

```bash
cd bindings/cpp
cmake -S . -B build
cmake --build build --config Release
```

Optional override:
- set `PP2P_CORE_LIB` to an absolute path to your own native library.
