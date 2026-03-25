<?php

declare(strict_types=1);

namespace Pp2p\Core;

use FFI;
use RuntimeException;

final class Pp2pCore
{
    private FFI $ffi;

    public function __construct(?string $libraryPath = null)
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

    private function resolveLibraryPath(?string $libraryPath): string
    {
        if ($libraryPath !== null && $libraryPath !== '') {
            return $libraryPath;
        }

        $envPath = getenv('PP2P_CORE_LIB');
        if ($envPath !== false && $envPath !== '') {
            return $envPath;
        }

        [$platformDir, $fileName] = $this->platformTarget();
        $repoRoot = dirname(__DIR__, 3);
        $bundled = $repoRoot . DIRECTORY_SEPARATOR . 'native' . DIRECTORY_SEPARATOR . 'pp2p_core' .
            DIRECTORY_SEPARATOR . $platformDir . DIRECTORY_SEPARATOR . $fileName;

        if (is_file($bundled)) {
            return $bundled;
        }

        throw new RuntimeException(
            'PP2P native library not found for this platform. ' .
            'Set PP2P_CORE_LIB or use a package build that includes native binaries.'
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
                return ['win32-x64', 'pp2p_core.dll'];
            }
        } elseif ($family === 'Darwin') {
            if ($arch === 'arm64' || $arch === 'aarch64') {
                return ['darwin-arm64', 'libpp2p_core.dylib'];
            }
            if ($arch === 'x86_64' || $arch === 'amd64') {
                return ['darwin-x64', 'libpp2p_core.dylib'];
            }
        } elseif ($family === 'Linux') {
            if ($arch === 'x86_64' || $arch === 'amd64') {
                return ['linux-x64', 'libpp2p_core.so'];
            }
        }

        throw new RuntimeException("Unsupported platform for PP2P native library: {$family}/{$arch}");
    }
}
