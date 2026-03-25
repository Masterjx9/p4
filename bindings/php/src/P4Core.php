<?php

declare(strict_types=1);

namespace P4\Core;

use FFI;
use RuntimeException;

final class P4Core
{
    private FFI $ffi;

    public function __construct(?string $libraryPath = null)
    {
        $cdef = <<<CDEF
char *p4_generate_identity_json(void);
char *p4_peer_id_from_public_key_b64(const char *public_key_b64);
char *p4_sign_envelope_json(
    const char *private_key_b64,
    const char *sender_peer_id,
    const char *recipient_peer_id,
    const char *payload_json,
    uint64_t timestamp_ms,
    const char *nonce
);
unsigned char p4_verify_envelope_json(
    const char *envelope_json,
    const char *signer_public_key_b64,
    uint64_t now_ms,
    uint64_t max_skew_ms
);
char *p4_last_error_message(void);
void p4_free_string(char *ptr);
CDEF;
        $libraryPath = $this->resolveLibraryPath($libraryPath);
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
            $this->ffi->p4_free_string($ptr);
        }
    }

    public function lastError(): string
    {
        return $this->takeString($this->ffi->p4_last_error_message());
    }

    public function generateIdentityJson(): string
    {
        return $this->takeString($this->ffi->p4_generate_identity_json());
    }

    public function peerIdFromPublicKeyB64(string $publicKeyB64): string
    {
        return $this->takeString($this->ffi->p4_peer_id_from_public_key_b64($publicKeyB64));
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
            $this->ffi->p4_sign_envelope_json(
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
        $ok = $this->ffi->p4_verify_envelope_json(
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

    private function resolveLibraryPath(?string $libraryPath): string
    {
        if ($libraryPath !== null && $libraryPath !== '') {
            return $libraryPath;
        }

        $envPath = getenv('P4_CORE_LIB');
        if ($envPath !== false && $envPath !== '') {
            return $envPath;
        }

        [$platformDir, $fileName] = $this->platformTarget();
        $repoRoot = dirname(__DIR__, 3);
        $bundled = $repoRoot . DIRECTORY_SEPARATOR . 'native' . DIRECTORY_SEPARATOR . 'p4_core' .
            DIRECTORY_SEPARATOR . $platformDir . DIRECTORY_SEPARATOR . $fileName;

        if (is_file($bundled)) {
            return $bundled;
        }

        throw new RuntimeException(
            'P⁴ native library not found for this platform. ' .
            'Set P4_CORE_LIB or use a package build that includes native binaries.'
        );
    }

    /**
     * @return array{0:string,1:string}
     */
    private function platformTarget(): array
    {
        $family = PHP_OS_FAMILY;
        $arch = strtolower((string)php_uname('m'));

        if ($family === 'Windows') {
            if (str_contains($arch, '64')) {
                return ['win32-x64', 'p4_core.dll'];
            }
        } elseif ($family === 'Darwin') {
            if ($arch === 'arm64' || $arch === 'aarch64') {
                return ['darwin-arm64', 'libp4_core.dylib'];
            }
            if ($arch === 'x86_64' || $arch === 'amd64') {
                return ['darwin-x64', 'libp4_core.dylib'];
            }
        } elseif ($family === 'Linux') {
            if ($arch === 'x86_64' || $arch === 'amd64') {
                return ['linux-x64', 'libp4_core.so'];
            }
        }

        throw new RuntimeException("Unsupported platform for P⁴ native library: {$family}/{$arch}");
    }
}

