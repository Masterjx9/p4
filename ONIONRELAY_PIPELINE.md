# OnionRelay Build Pipeline

This repository now includes a cross-platform pipeline that builds a slimmed Tor-based
binary named `onionrelay` for:

1. Linux x86_64
2. macOS Intel (`macos-13`)
3. macOS Apple Silicon (`macos-14`)

## CI Workflow

Workflow file:

- `.github/workflows/build-onionrelay-unix.yml`

Trigger:

1. Manually via `workflow_dispatch`
2. Pushes to `main` touching:
   - workflow file
   - `scripts/build_onionrelay_unix.sh`
   - `tor_win_min_src/**`

Output artifacts:

1. `onionrelay-linux-x86_64.tar.gz`
2. `onionrelay-macos-intel.tar.gz`
3. `onionrelay-macos-apple-silicon.tar.gz`

Each tarball contains:

1. `onionrelay`
2. `LICENSE`

## Local Build Script (Linux/macOS)

Script:

- `scripts/build_onionrelay_unix.sh`

Example:

```bash
chmod +x scripts/build_onionrelay_unix.sh
scripts/build_onionrelay_unix.sh ./dist onionrelay
```

Build flags used for slim profile:

1. `--disable-asciidoc`
2. `--disable-module-relay`
3. `--disable-module-dirauth`
4. `--disable-module-pow`
