/*
 * Node.js bridge for the Rust P4 C ABI.
 */

const ffi = require("ffi-napi");
const ref = require("ref-napi");
const fs = require("fs");
const path = require("path");

function defaultLibPath() {
  if (process.env.P4_CORE_LIB) {
    return process.env.P4_CORE_LIB;
  }

  let bundled = null;
  if (process.platform === "win32" && process.arch === "x64") {
    bundled = path.resolve(__dirname, "native", "win32-x64", "p4_core.dll");
  } else if (process.platform === "darwin" && process.arch === "x64") {
    bundled = path.resolve(__dirname, "native", "darwin-x64", "libp4_core.dylib");
  } else if (process.platform === "darwin" && process.arch === "arm64") {
    bundled = path.resolve(__dirname, "native", "darwin-arm64", "libp4_core.dylib");
  } else if (process.platform === "linux" && process.arch === "x64") {
    bundled = path.resolve(__dirname, "native", "linux-x64", "libp4_core.so");
  }

  if (bundled && fs.existsSync(bundled)) {
    return bundled;
  }

  let rel = null;
  if (process.platform === "win32") rel = "dist/p4_core/windows-x64/p4_core.dll";
  else if (process.platform === "darwin") rel = "dist/p4_core/macos/libp4_core.dylib";
  else rel = "dist/p4_core/linux-x64/libp4_core.so";

  const candidates = [
    path.resolve(process.cwd(), rel),
    path.resolve(__dirname, "..", "..", rel),
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  throw new Error(
    `P4 native library not found for ${process.platform}/${process.arch}. ` +
      `Set P4_CORE_LIB or install a package build that includes native binaries.`
  );
}

class P4Core {
  constructor(libPath = defaultLibPath()) {
    this.lib = ffi.Library(libPath, {
      p4_generate_identity_json: ["pointer", []],
      p4_peer_id_from_public_key_b64: ["pointer", ["string"]],
      p4_sign_envelope_json: ["pointer", ["string", "string", "string", "string", "uint64", "string"]],
      p4_verify_envelope_json: ["uchar", ["string", "string", "uint64", "uint64"]],
      p4_last_error_message: ["pointer", []],
      p4_free_string: ["void", ["pointer"]],
    });
  }

  _takeString(ptr) {
    if (ref.isNull(ptr)) {
      throw new Error(this.lastError());
    }
    try {
      return ref.readCString(ptr, 0);
    } finally {
      this.lib.p4_free_string(ptr);
    }
  }

  lastError() {
    return this._takeString(this.lib.p4_last_error_message());
  }

  generateIdentity() {
    return JSON.parse(this._takeString(this.lib.p4_generate_identity_json()));
  }

  peerIdFromPublicKeyB64(publicKeyB64) {
    return this._takeString(this.lib.p4_peer_id_from_public_key_b64(publicKeyB64));
  }

  signEnvelope({ privateKeyB64, senderPeerId, recipientPeerId, payload, timestampMs, nonce }) {
    const ts = typeof timestampMs === "number" ? timestampMs : Date.now();
    const ptr = this.lib.p4_sign_envelope_json(
      privateKeyB64,
      senderPeerId,
      recipientPeerId,
      JSON.stringify(payload),
      ts,
      nonce
    );
    return JSON.parse(this._takeString(ptr));
  }

  verifyEnvelope({ envelope, signerPublicKeyB64, maxSkewMs = 60000, nowMs }) {
    const now = typeof nowMs === "number" ? nowMs : Date.now();
    const ok = this.lib.p4_verify_envelope_json(
      JSON.stringify(envelope),
      signerPublicKeyB64,
      now,
      maxSkewMs
    );
    if (ok === 1) {
      return true;
    }
    throw new Error(this.lastError());
  }
}

module.exports = { P4Core };


