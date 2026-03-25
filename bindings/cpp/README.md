# C++ SDK Wrapper

This directory has:
- `include/p4_core_cpp.hpp` thin C++ wrapper API
- `src/p4_core_cpp.cpp` implementation over `p4_core.h`
- `CMakeLists.txt` for build/link

The C++ wrapper dynamically loads the P4 native library and auto-resolves
bundled binaries under `native/p4_core/<platform>/`.

## Build example

```bash
cd bindings/cpp
cmake -S . -B build
cmake --build build --config Release
```

Optional override:
- set `P4_CORE_LIB` to an absolute path to your own native library.


