# PP2P (Persistent Peer-to-Peer Protocol)

PP2P keeps normal P2P data paths (WebRTC/DataChannel) and adds persistent onion rendezvous for rediscovery and automatic reconnect after disconnects.

This repository is a monorepo containing:
- Runtime reference CLI: `pp2p.py`
- Rust core + C ABI: `rust/pp2p-core`, `rust/pp2p-ffi`, `include/pp2p_core.h`
- SDK bindings: `bindings/`
- Minimal onion relay source/build tree: `tor_win_min_src/`

## Why PP2P

Use PP2P when you want direct encrypted peer communication without making a central app server the permanent source of truth.

Example use cases:
- Personal file sync (Dropbox alternative)
- Password manager cross-device sync
- Notes/tasks syncing (offline-first apps)
- Home automation control (phone to local hub)
- Game state sync (local-host multiplayer)
- Camera/IoT monitoring without cloud
- Clipboard sharing between devices
- Local-first databases (CRDT peer sync)
- Encrypted chat without central servers
- Dev tools: logs/metrics between local machines

## What Is Supported

Protocol/runtime support:
- Signed envelope identity/authentication via Rust core crypto
- Peer reconnect loop after channel drop
- Persistent onion rendezvous identity for rediscovery
- WebRTC DataChannel messaging
- STUN by default (`stun:stun.l.google.com:19302`)
- TURN optional (recommended for stricter NAT environments)

Platforms with bundled native core binaries:
- Windows x64
- Linux x64
- macOS Intel (x64)
- macOS Apple Silicon (arm64)

## SDK Packages (Install First)

Python:
- Package: `pp2p_core`
- PyPI: `https://pypi.org/project/pp2p_core/`
- Install: `pip install pp2p_core`
- Legacy compatibility package: `pp2p-core-sdk` (`https://pypi.org/project/pp2p-core-sdk/`)
- Note: the official package name is `pp2p_core`.

JavaScript/TypeScript:
- Package: `@pythonicit/pp2p-core-sdk`
- npm: `https://www.npmjs.com/package/@pythonicit/pp2p-core-sdk`
- Install: `npm i @pythonicit/pp2p-core-sdk`

Java:
- Coordinates: `io.github.masterjx9:pp2p-core-sdk:0.2.0`
- Maven Central: `https://central.sonatype.com/artifact/io.github.masterjx9/pp2p-core-sdk`

PHP:
- Package: `masterjx9/pp2p-core-sdk`
- Packagist: `https://packagist.org/packages/masterjx9/pp2p-core-sdk`
- Install: `composer require masterjx9/pp2p-core-sdk`

C++:
- Wrapper lives in this repo: `bindings/cpp`
- Uses bundled native payload under `native/pp2p_core/<platform>/`

## Source Install (Secondary)

Use source installs only when developing/contributing to this monorepo:
- Python: `pip install -e .\bindings\python`
- JS/TS: `npm install .\bindings\javascript`
- Java: `mvn -f bindings\java\pom.xml package`
- PHP: `composer install` in repo root
- C++: `cmake -S bindings/cpp -B bindings/cpp/build && cmake --build bindings/cpp/build --config Release`

## Abstract Device Requirements

For any device/platform/language implementation of PP2P:
- A stable local identity keypair persisted on disk
- Ability to run the PP2P native crypto core for that OS/arch
- Network access to onion relay network for rendezvous signaling
- Network access for WebRTC ICE (STUN, optionally TURN)
- Local storage for state (identity, known peers, onion service metadata)
- Reasonably correct system clock for envelope freshness checks
- Ability to open local loopback/listener ports for runtime signaling paths

## Python Test (Package-First)

This test uses `pip install pp2p_core` first and does not require editable/source install.

1. Create env and install dependencies:

```powershell
py -3 -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
pip install pp2p_core aiortc cryptography
```

2. Build onion relay binary (Windows):

```powershell
powershell -ExecutionPolicy Bypass -File .\build_tor_subset_windows.ps1
```

3. Initialize two peers:

```powershell
python pp2p.py init --state-dir .\human_test\peerA
python pp2p.py init --state-dir .\human_test\peerB
```

4. Start peer A (terminal 1):

```powershell
python pp2p.py run --state-dir .\human_test\peerA --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe
```

5. Start peer B (terminal 2):

```powershell
python pp2p.py run --state-dir .\human_test\peerB --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe
```

6. In each `pp2p>` prompt:
- Run `/invite` and exchange JSON
- Add the other peer invite: `/add-json <invite-json>` or `/add-file <path>`
- Verify with `/peers`
- Send message: `/send <peer_id> hello`
- Reconnect test: `/drop <peer_id>` then wait for auto reconnect and send again

## Architecture Summary

Rust core:
- `rust/pp2p-core`: identity, peer id derivation, envelope sign/verify
- `rust/pp2p-ffi`: C ABI
- `include/pp2p_core.h`: ABI contract

Python runtime:
- `pp2p.py` is a reference node/CLI and consumes `pp2p_core` package.

Onion relay:
- Windows subset build via `build_tor_subset_windows.ps1`
- Linux/macOS build pipeline via `.github/workflows/build-onionrelay-unix.yml`

## Contributing

1. Fork and create a feature branch from `main`.
2. Keep changes scoped (runtime, core, or one binding per PR when possible).
3. Run the relevant local checks before pushing:
- Python: import + runtime smoke where touched
- JS: package install smoke where touched
- Java: `mvn -f bindings/java/pom.xml package`
- PHP: `php -l` and runtime smoke where touched
- C++: CMake build + example run
4. Update docs/README for any behavior or install flow change.
5. Open PR with:
- what changed
- how you tested
- platform(s) tested

## Security Notes

- Onion is used for rendezvous/signaling persistence.
- App data still runs over direct P2P channels when ICE succeeds.
- TURN is optional but recommended for higher reliability through strict NAT/firewall conditions.
- You can override native lib path in all SDKs with `PP2P_CORE_LIB`.
