<?php

declare(strict_types=1);

namespace Pp2p\Core;

use FFI;
use RuntimeException;

final class Pp2pCore
{
    private FFI $ffi;

    public function __construct(string $libraryPath)
    {
        $cdef = <<<CDEF
char *pp2p_generate_identity_json(void);
char *pp2p_peer_id_from_public_key_b64(const char *public_key_b64);
char *pp2p_sign_envelope_json(
    const char *private_key_b64,
    const char *sender_peer_id,
    const char *recipient_peer_id,
    const char *payload_json,
    uint64_t timestamp_ms,
    const char *nonce
);
unsigned char pp2p_verify_envelope_json(
    const char *envelope_json,
    const char *signer_public_key_b64,
    uint64_t now_ms,
    uint64_t max_skew_ms
);
char *pp2p_last_error_message(void);
void pp2p_free_string(char *ptr);
CDEF;
        $this->ffi = FFI::cdef($cdef, $libraryPath);
    }

    private function takeString($ptr): string
    {
        if ($ptr === null) {
            throw new RuntimeException($this->lastError());
        }

        try {
            return FFI::string($ptr);
        } finally {
            $this->ffi->pp2p_free_string($ptr);
        }
    }

    public function lastError(): string
    {
        return $this->takeString($this->ffi->pp2p_last_error_message());
    }

    public function generateIdentityJson(): string
    {
        return $this->takeString($this->ffi->pp2p_generate_identity_json());
    }

    public function peerIdFromPublicKeyB64(string $publicKeyB64): string
    {
        return $this->takeString($this->ffi->pp2p_peer_id_from_public_key_b64($publicKeyB64));
    }

    public function signEnvelopeJson(
        string $privateKeyB64,
        string $senderPeerId,
        string $recipientPeerId,
        string $payloadJson,
        int $timestampMs,
        string $nonce
    ): string {
        return $this->takeString(
            $this->ffi->pp2p_sign_envelope_json(
                $privateKeyB64,
                $senderPeerId,
                $recipientPeerId,
                $payloadJson,
                $timestampMs,
                $nonce
            )
        );
    }

    public function verifyEnvelopeJson(
        string $envelopeJson,
        string $signerPublicKeyB64,
        int $nowMs,
        int $maxSkewMs = 60000
    ): bool {
        $ok = $this->ffi->pp2p_verify_envelope_json(
            $envelopeJson,
            $signerPublicKeyB64,
            $nowMs,
            $maxSkewMs
        );
        if ((int)$ok === 1) {
            return true;
        }
        throw new RuntimeException($this->lastError());
    }
}
