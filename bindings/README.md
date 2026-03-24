# Multi-Language SDK Bindings

All SDKs call the same native ABI in [pp2p_core.h](/c:/Users/RKerrigan/Projects/pp2p/include/pp2p_core.h).

## Native library build

- Windows: `.\scripts\build_pp2p_core.ps1`
- Linux/macOS: `./scripts/build_pp2p_core_unix.sh`

Output:
- Windows: `dist/pp2p_core/windows-x64/pp2p_core.dll`
- Linux: `dist/pp2p_core/linux-x64/libpp2p_core.so`
- macOS: `dist/pp2p_core/macos/libpp2p_core.dylib`

## SDK packages

- Python: `bindings/python` (`pyproject.toml`, module `pp2p_core.py`)
- JavaScript/TypeScript: `bindings/javascript` (`package.json`)
- Java: `bindings/java` (`pom.xml`, JNA wrapper)
- C++: `bindings/cpp` (`CMakeLists.txt`, wrapper static lib)
- PHP: repo root `composer.json` (autoload -> `bindings/php/src`)

Maven namespace in this repo:
- `io.github.masterjx9`

Quick package commands:
- Python: `cd bindings/python && python -m build`
- JS/TS: `cd bindings/javascript && npm pack`
- Java: `cd bindings/java && mvn package`
- C++: `cd bindings/cpp && cmake -S . -B build && cmake --build build`
- PHP: `composer install`

Each binding README has language-specific usage examples.
