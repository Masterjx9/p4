# Python SDK

This package is a ctypes bridge over the PP2P Rust C ABI.

## Build native library first

Windows:
```powershell
.\scripts\build_pp2p_core.ps1
```

Linux/macOS:
```bash
./scripts/build_pp2p_core_unix.sh
```

## Install the Python SDK package (local)

```powershell
pip install -e .\bindings\python
```

## Example

```python
from pp2p_core import Pp2pCore

core = Pp2pCore()
alice = core.generate_identity()
bob = core.generate_identity()

env = core.sign_envelope(
    private_key_b64=alice["private_key_b64"],
    sender_peer_id=alice["peer_id"],
    recipient_peer_id=bob["peer_id"],
    payload={"type": "hello", "text": "hi"},
    nonce="n1",
)
core.verify_envelope(env, signer_public_key_b64=alice["public_key_b64"])
print("ok")
```

If the library is not in `dist/pp2p_core/...`, set `PP2P_CORE_LIB` to the absolute path.
