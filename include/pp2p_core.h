#ifndef PP2P_CORE_H
#define PP2P_CORE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Returns identity JSON:
 * {"private_key_b64":"...","public_key_b64":"...","peer_id":"..."}
 */
char *pp2p_generate_identity_json(void);

/**
 * Returns peer_id (first 24 hex chars of sha256(public_key_raw)).
 */
char *pp2p_peer_id_from_public_key_b64(const char *public_key_b64);

/**
 * Signs a protocol envelope and returns JSON:
 * {
 *   "protocol_version":1,
 *   "sender_peer_id":"...",
 *   "recipient_peer_id":"...",
 *   "timestamp_ms":123,
 *   "nonce":"...",
 *   "payload":{...},
 *   "signature_b64":"..."
 * }
 */
char *pp2p_sign_envelope_json(
    const char *private_key_b64,
    const char *sender_peer_id,
    const char *recipient_peer_id,
    const char *payload_json,
    uint64_t timestamp_ms,
    const char *nonce
);

/**
 * Returns 1 on valid envelope signature+peer_id+timestamp, otherwise 0.
 */
unsigned char pp2p_verify_envelope_json(
    const char *envelope_json,
    const char *signer_public_key_b64,
    uint64_t now_ms,
    uint64_t max_skew_ms
);

/**
 * Returns last error string from the current process.
 * Always returns a heap string (possibly empty); free with pp2p_free_string.
 */
char *pp2p_last_error_message(void);

/**
 * Frees strings returned by this library.
 */
void pp2p_free_string(char *ptr);

#ifdef __cplusplus
}
#endif

#endif

