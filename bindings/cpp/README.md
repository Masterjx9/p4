# C++ SDK Wrapper

This directory has:
- `include/pp2p_core_cpp.hpp` thin C++ wrapper API
- `src/pp2p_core_cpp.cpp` implementation over `pp2p_core.h`
- `CMakeLists.txt` for build/link

## Build example

Build native core from repo root first, then:

```bash
cd bindings/cpp
cmake -S . -B build
cmake --build build --config Release
```
