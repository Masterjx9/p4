# PP2P (Persistent P2P) - Save Point

PP2P is a persistent peer-to-peer prototype:
- real P2P data path over WebRTC DataChannel
- onion rendezvous/signaling for rediscovery
- automatic renegotiation after disconnect
- pinned peer trust (Ed25519 identity)

## Current Status

Working now:
1. Two peers can connect and exchange messages.
2. Two peers auto-reconnect after session drop.
3. Reconnect still works after WAN region cutover (tested by moving live peerB between Fly regions while keeping the same identity/onion key).

## Protocol Flow

```text
   +------+        +----------------------------------+        +----------------------------------+
   | Dead |------->| Establish                        |------->| Authenticate                     |
   +------+   UP   | - direct/onion rendezvous select | OPENED | - verify pinned peer            |
      ^            | - signed SDP offer/answer        |        | - verify Ed25519 sig + nonce    |
      |            | - onion only for signaling       |        | - fail if trust check fails     |
      |            +----------------------------------+        +-------------------+--------------+
      |                         FAIL                                              SUCCESS/NONE |
      |                                                                                       |
      |            +----------------------------------+        CLOSING                        v
      +------------| Terminate                        |<--------------------------------+---------------+
           DOWN    | - close session / stop node      |                                 | Network       |
                   +----------------------------------+                                 | - WebRTC/ICE  |
                                                                                        | - DataChannel |
                                                                                        | - app traffic |
                                                                                        | - reconnect   |
                                                                                        |   -> Establish|
                                                                                        +---------------+
```

## Quick Start (Windows)

1. Create env and install deps:
```powershell
py -3 -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

2. Build onionrelay (native Windows):
```powershell
.\build_tor_subset_windows.ps1
```

3. Initialize peers:
```powershell
.\.venv\Scripts\python.exe pp2p.py init --state-dir .\human_test\peerA
.\.venv\Scripts\python.exe pp2p.py init --state-dir .\human_test\peerB
```

4. Run both peers (separate terminals):
```powershell
.\.venv\Scripts\python.exe pp2p.py run --state-dir .\human_test\peerA --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe
.\.venv\Scripts\python.exe pp2p.py run --state-dir .\human_test\peerB --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe
```

5. Exchange invites and add contacts:
- Use `/invite` inside each `pp2p>` runtime.
- On each side, add the other invite with `/add-json ...` or `/add-file ...`.

6. Test:
- `/peers`
- `/send <peer_id> hello`
- `/drop <peer_id>` and wait for auto reconnect.

## Useful Scripts

- `direct_smoketest.py`: local direct-mode smoke test
- `onion_smoketest.py`: onion-mode smoke test
- `scripts/build_onionrelay_unix.sh`: Linux/macOS onionrelay build

## Repo Notes

- `pp2p.py` is the core runtime and CLI.
- `tor_win_min_src/` is the local minimal onionrelay source/build tree.
- Fly test app files are intentionally local-only and ignored by `.gitignore` for this save point.
