use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;
pub const PEER_ID_HEX_LEN: usize = 24;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid base64: {0}")]
    InvalidBase64(String),
    #[error("invalid key length for {0}: expected {1}, got {2}")]
    InvalidKeyLength(&'static str, usize, usize),
    #[error("invalid signature length: expected 64, got {0}")]
    InvalidSignatureLength(usize),
    #[error("json error: {0}")]
    Json(String),
    #[error("canonicalization error: {0}")]
    Canonicalization(String),
    #[error("protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolVersionMismatch { expected: u32, actual: u32 },
    #[error("peer id mismatch: expected {expected}, got {actual}")]
    PeerIdMismatch { expected: String, actual: String },
    #[error("signature verification failed")]
    SignatureVerificationFailed,
    #[error("timestamp outside allowed skew window")]
    TimestampSkewExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub private_key_b64: String,
    pub public_key_b64: String,
    pub peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub protocol_version: u32,
    pub sender_peer_id: String,
    pub recipient_peer_id: String,
    pub timestamp_ms: u64,
    pub nonce: String,
    pub payload: Value,
    pub signature_b64: String,
}

#[derive(Debug, Clone, Serialize)]
struct SignableEnvelope<'a> {
    protocol_version: u32,
    sender_peer_id: &'a str,
    recipient_peer_id: &'a str,
    timestamp_ms: u64,
    nonce: &'a str,
    payload: &'a Value,
}

pub fn generate_identity() -> Identity {
    let signing_key = SigningKey::generate(&mut OsRng);
    let private_key_b64 = B64.encode(signing_key.to_bytes());
    let public_key_b64 = B64.encode(signing_key.verifying_key().to_bytes());
    let peer_id = peer_id_from_public_key_bytes(signing_key.verifying_key().as_bytes());
    Identity {
        private_key_b64,
        public_key_b64,
        peer_id,
    }
}

pub fn peer_id_from_public_key_bytes(public_key: &[u8]) -> String {
    let digest = Sha256::digest(public_key);
    let hex = hex_lower(&digest);
    hex[..PEER_ID_HEX_LEN].to_string()
}

pub fn peer_id_from_public_key_b64(public_key_b64: &str) -> Result<String, CoreError> {
    let bytes = decode_fixed_len_b64(public_key_b64, 32, "public key")?;
    Ok(peer_id_from_public_key_bytes(&bytes))
}

pub fn signing_key_from_b64(private_key_b64: &str) -> Result<SigningKey, CoreError> {
    let bytes = decode_fixed_len_b64(private_key_b64, 32, "private key")?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(SigningKey::from_bytes(&arr))
}

pub fn verifying_key_from_b64(public_key_b64: &str) -> Result<VerifyingKey, CoreError> {
    let bytes = decode_fixed_len_b64(public_key_b64, 32, "public key")?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    VerifyingKey::from_bytes(&arr).map_err(|e| CoreError::InvalidBase64(e.to_string()))
}

pub fn sign_envelope(
    signing_key: &SigningKey,
    sender_peer_id: &str,
    recipient_peer_id: &str,
    timestamp_ms: u64,
    nonce: &str,
    payload: Value,
) -> Result<Envelope, CoreError> {
    let signable = SignableEnvelope {
        protocol_version: PROTOCOL_VERSION,
        sender_peer_id,
        recipient_peer_id,
        timestamp_ms,
        nonce,
        payload: &payload,
    };
    let canonical = serde_jcs::to_vec(&signable)
        .map_err(|e| CoreError::Canonicalization(e.to_string()))?;
    let signature = signing_key.sign(&canonical);
    Ok(Envelope {
        protocol_version: PROTOCOL_VERSION,
        sender_peer_id: sender_peer_id.to_string(),
        recipient_peer_id: recipient_peer_id.to_string(),
        timestamp_ms,
        nonce: nonce.to_string(),
        payload,
        signature_b64: B64.encode(signature.to_bytes()),
    })
}

pub fn verify_envelope(
    envelope: &Envelope,
    verifying_key: &VerifyingKey,
    now_ms: u64,
    max_skew_ms: u64,
) -> Result<(), CoreError> {
    if envelope.protocol_version != PROTOCOL_VERSION {
        return Err(CoreError::ProtocolVersionMismatch {
            expected: PROTOCOL_VERSION,
            actual: envelope.protocol_version,
        });
    }

    let expected_peer_id = peer_id_from_public_key_bytes(verifying_key.as_bytes());
    if envelope.sender_peer_id != expected_peer_id {
        return Err(CoreError::PeerIdMismatch {
            expected: expected_peer_id,
            actual: envelope.sender_peer_id.clone(),
        });
    }

    let lower = now_ms.saturating_sub(max_skew_ms);
    let upper = now_ms.saturating_add(max_skew_ms);
    if envelope.timestamp_ms < lower || envelope.timestamp_ms > upper {
        return Err(CoreError::TimestampSkewExceeded);
    }

    let signable = SignableEnvelope {
        protocol_version: envelope.protocol_version,
        sender_peer_id: &envelope.sender_peer_id,
        recipient_peer_id: &envelope.recipient_peer_id,
        timestamp_ms: envelope.timestamp_ms,
        nonce: &envelope.nonce,
        payload: &envelope.payload,
    };
    let canonical = serde_jcs::to_vec(&signable)
        .map_err(|e| CoreError::Canonicalization(e.to_string()))?;

    let sig_bytes = B64
        .decode(envelope.signature_b64.as_bytes())
        .map_err(|e| CoreError::InvalidBase64(e.to_string()))?;
    if sig_bytes.len() != 64 {
        return Err(CoreError::InvalidSignatureLength(sig_bytes.len()));
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(&canonical, &sig)
        .map_err(|_| CoreError::SignatureVerificationFailed)
}

pub fn envelope_to_json(envelope: &Envelope) -> Result<String, CoreError> {
    serde_json::to_string(envelope).map_err(|e| CoreError::Json(e.to_string()))
}

pub fn envelope_from_json(raw: &str) -> Result<Envelope, CoreError> {
    serde_json::from_str(raw).map_err(|e| CoreError::Json(e.to_string()))
}

#[derive(Debug, Clone)]
pub struct ReplayWindow {
    max_seen: usize,
    seen: HashMap<String, VecDeque<String>>,
    seen_set: HashMap<String, HashSet<String>>,
}

impl ReplayWindow {
    pub fn new(max_seen: usize) -> Self {
        Self {
            max_seen,
            seen: HashMap::new(),
            seen_set: HashMap::new(),
        }
    }

    pub fn seen_before(&mut self, peer_id: &str, nonce: &str) -> bool {
        let set = self.seen_set.entry(peer_id.to_string()).or_default();
        if set.contains(nonce) {
            return true;
        }

        set.insert(nonce.to_string());
        let queue = self.seen.entry(peer_id.to_string()).or_default();
        queue.push_back(nonce.to_string());

        while queue.len() > self.max_seen {
            if let Some(old) = queue.pop_front() {
                if let Some(set) = self.seen_set.get_mut(peer_id) {
                    set.remove(&old);
                }
            }
        }
        false
    }
}

fn decode_fixed_len_b64(
    text: &str,
    expected_len: usize,
    label: &'static str,
) -> Result<Vec<u8>, CoreError> {
    let bytes = B64
        .decode(text.as_bytes())
        .map_err(|e| CoreError::InvalidBase64(e.to_string()))?;
    if bytes.len() != expected_len {
        return Err(CoreError::InvalidKeyLength(label, expected_len, bytes.len()));
    }
    Ok(bytes)
}

fn hex_lower(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sign_and_verify_roundtrip() {
        let identity = generate_identity();
        let signing = signing_key_from_b64(&identity.private_key_b64).unwrap();
        let verifying = verifying_key_from_b64(&identity.public_key_b64).unwrap();

        let now = 1_700_000_000_000u64;
        let env = sign_envelope(
            &signing,
            &identity.peer_id,
            "peer_b",
            now,
            "n-1",
            json!({"type":"offer","sdp":"abc"}),
        )
        .unwrap();

        verify_envelope(&env, &verifying, now + 1_000, 30_000).unwrap();
    }

    #[test]
    fn replay_window_detects_duplicates() {
        let mut rw = ReplayWindow::new(2);
        assert!(!rw.seen_before("peer", "a"));
        assert!(rw.seen_before("peer", "a"));
        assert!(!rw.seen_before("peer", "b"));
        assert!(!rw.seen_before("peer", "c"));
        // "a" should have been evicted once max_seen exceeded.
        assert!(!rw.seen_before("peer", "a"));
    }

    #[test]
    fn verify_rejects_bad_peer_id() {
        let identity = generate_identity();
        let signing = signing_key_from_b64(&identity.private_key_b64).unwrap();
        let verifying = verifying_key_from_b64(&identity.public_key_b64).unwrap();
        let mut env = sign_envelope(
            &signing,
            &identity.peer_id,
            "peer_b",
            100,
            "n-1",
            json!({"x":1}),
        )
        .unwrap();
        env.sender_peer_id = "tampered".to_string();
        let err = verify_envelope(&env, &verifying, 100, 10).unwrap_err();
        assert!(matches!(err, CoreError::PeerIdMismatch { .. }));
    }
}

