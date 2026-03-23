# Tor Minimal Calls For PP2P SDK Porting

This document defines the smallest Tor surface PP2P needs.

## Goal

Use Tor only for rendezvous discovery/reconnect signaling.
Do not depend on Tor internal C APIs.
Build a small Tor client runtime per target OS/arch (for example Windows `tor.exe`
from local source subset), then call only the interfaces below.

## Required Tor Capabilities

1. Run a Tor client process.
2. Expose a SOCKS5 port.
3. Expose a control port.
4. Create/remove a v3 onion service through control commands.
5. Connect to remote `.onion` endpoints through SOCKS5.

Note: TURN is not a Tor capability. TURN is part of ICE/WebRTC media fallback and is optional.

## Control Port Commands Used

1. `AUTHENTICATE`
- command:
```text
AUTHENTICATE
```

2. `ADD_ONION`
- new key (first run):
```text
ADD_ONION NEW:ED25519-V3 Flags=Detach Port=80,127.0.0.1:<LOCAL_SIGNAL_PORT>
```
- existing key (subsequent runs):
```text
ADD_ONION ED25519-V3:<KEY_BLOB> Flags=Detach Port=80,127.0.0.1:<LOCAL_SIGNAL_PORT>
```
- expected reply fields:
  - `ServiceID=<56-char-id>`
  - optionally `PrivateKey=ED25519-V3:<KEY_BLOB>` when key is newly generated

3. `GETINFO status/bootstrap-phase`
- used to wait until Tor reaches `PROGRESS=100`.

4. `DEL_ONION`
- command:
```text
DEL_ONION <SERVICE_ID>
```

## SOCKS5 Calls Used

1. Open TCP to local SOCKS port.
2. No-auth negotiation: `0x05 0x01 0x00`.
3. `CONNECT` with domain address type (`ATYP=0x03`) and host=`<peer>.onion`.

No Tor-specific library is needed for this, only SOCKS framing.

## Persisted Onion Identity

PP2P persists Tor rendezvous identity in app-owned files:

1. `onion_v3_key_blob.txt`
- stores `ED25519-V3` key blob returned by `ADD_ONION`.

2. `onion_service_id.txt`
- stores onion service id (`<service_id>` without `.onion`).

This keeps identity ownership in the SDK layer, not in Tor hidden-service directories.

## Where This Maps In Tor Source

If you need to trace behavior in the downloaded Tor repo:

1. `ADD_ONION` parser/handler:
- `tor/src/feature/control/control_cmd.c`

2. Hidden service registration path:
- `tor/src/feature/hs/hs_service.c`

These are references only. PP2P runtime does not link against these internals.

## SDK Porting Contract

Any language SDK (Swift/Kotlin/Rust/Go/etc.) needs to provide this interface:

1. `start_tor_client(config) -> handles`
2. `tor_control_authenticate(handles.control)`
3. `tor_add_onion(handles.control, key_blob?, local_port) -> {service_id, key_blob}`
4. `tor_wait_bootstrap(handles.control)`
5. `tor_socks_connect(handles.socks, onion_host, port) -> stream`
6. `tor_del_onion(handles.control, service_id)` (optional cleanup)

PP2P protocol logic (identity pinning, signed envelopes, reconnect loop) remains transport-agnostic above this layer.
