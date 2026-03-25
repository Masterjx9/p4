# PHP SDK (FFI)

Composer package metadata and wrapper class:
- [Pp2pCore.php](/c:/Users/RKerrigan/Projects/pp2p/bindings/php/src/Pp2pCore.php)

## Install

```bash
composer require masterjx9/pp2p-core-sdk
```

## Runtime requirements

- PHP 8.1+ with `ffi` enabled
- Bundled native binary is auto-loaded for:
  - Windows x64
  - Linux x64
  - macOS Intel (x64)
  - macOS Apple Silicon (arm64)

Enable `ffi` in `php.ini`, then:

```bash
composer install
php bindings/php/example.php
```

Optional override:
- set `PP2P_CORE_LIB` to an absolute path to your own native library.
