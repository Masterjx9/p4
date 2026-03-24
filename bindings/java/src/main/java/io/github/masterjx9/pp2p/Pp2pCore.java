package io.github.masterjx9.pp2p;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

public final class Pp2pCore {
    private interface NativeApi extends Library {
        Pointer pp2p_generate_identity_json();
        Pointer pp2p_peer_id_from_public_key_b64(String publicKeyB64);
        Pointer pp2p_sign_envelope_json(
            String privateKeyB64,
            String senderPeerId,
            String recipientPeerId,
            String payloadJson,
            long timestampMs,
            String nonce
        );
        byte pp2p_verify_envelope_json(
            String envelopeJson,
            String signerPublicKeyB64,
            long nowMs,
            long maxSkewMs
        );
        Pointer pp2p_last_error_message();
        void pp2p_free_string(Pointer ptr);
    }

    private final NativeApi api;

    public Pp2pCore(String libraryPath) {
        this.api = Native.load(libraryPath, NativeApi.class);
    }

    private String takeString(Pointer ptr) {
        if (ptr == null) {
            throw new RuntimeException(lastError());
        }
        try {
            return ptr.getString(0, "UTF-8");
        } finally {
            api.pp2p_free_string(ptr);
        }
    }

    public String lastError() {
        return takeString(api.pp2p_last_error_message());
    }

    public String generateIdentityJson() {
        return takeString(api.pp2p_generate_identity_json());
    }

    public String peerIdFromPublicKeyB64(String publicKeyB64) {
        return takeString(api.pp2p_peer_id_from_public_key_b64(publicKeyB64));
    }

    public String signEnvelopeJson(
        String privateKeyB64,
        String senderPeerId,
        String recipientPeerId,
        String payloadJson,
        long timestampMs,
        String nonce
    ) {
        return takeString(
            api.pp2p_sign_envelope_json(
                privateKeyB64,
                senderPeerId,
                recipientPeerId,
                payloadJson,
                timestampMs,
                nonce
            )
        );
    }

    public boolean verifyEnvelopeJson(
        String envelopeJson,
        String signerPublicKeyB64,
        long nowMs,
        long maxSkewMs
    ) {
        byte ok = api.pp2p_verify_envelope_json(envelopeJson, signerPublicKeyB64, nowMs, maxSkewMs);
        if (ok == 1) {
            return true;
        }
        throw new RuntimeException(lastError());
    }
}
