# JavaScript / TypeScript SDK

Node.js FFI wrapper for the PP2P Rust core.

## Install (local)

```bash
cd bindings/javascript
npm install
```

## Build native library first

From repo root:

```bash
./scripts/build_pp2p_core_unix.sh
```
or on Windows:
```powershell
.\scripts\build_pp2p_core.ps1
```

## Example

```javascript
const { Pp2pCore } = require("./bindings/javascript/pp2p_core");

const core = new Pp2pCore();
const a = core.generateIdentity();
const b = core.generateIdentity();

const env = core.signEnvelope({
  privateKeyB64: a.private_key_b64,
  senderPeerId: a.peer_id,
  recipientPeerId: b.peer_id,
  payload: { type: "offer" },
  nonce: "n1",
});

core.verifyEnvelope({ envelope: env, signerPublicKeyB64: a.public_key_b64 });
console.log("ok");
```
