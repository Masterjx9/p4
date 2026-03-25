package io.github.masterjx9.p4;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Locale;
import java.util.Objects;

public final class P4Core {
    private static final String ENV_LIBRARY_PATH = "P4_CORE_LIB";

    private interface NativeApi extends Library {
        Pointer p4_generate_identity_json();
        Pointer p4_peer_id_from_public_key_b64(String publicKeyB64);
        Pointer p4_sign_envelope_json(
            String privateKeyB64,
            String senderPeerId,
            String recipientPeerId,
            String payloadJson,
            long timestampMs,
            String nonce
        );
        byte p4_verify_envelope_json(
            String envelopeJson,
            String signerPublicKeyB64,
            long nowMs,
            long maxSkewMs
        );
        Pointer p4_last_error_message();
        void p4_free_string(Pointer ptr);
    }

    private final NativeApi api;

    public P4Core() {
        this(resolveDefaultLibraryPath());
    }

    public P4Core(String libraryPath) {
        Objects.requireNonNull(libraryPath, "libraryPath");
        this.api = Native.load(libraryPath, NativeApi.class);
    }

    private String takeString(Pointer ptr) {
        if (ptr == null) {
            throw new RuntimeException(lastError());
        }
        return readOwnedString(ptr, "unknown error");
    }

    public String lastError() {
        return readOwnedString(api.p4_last_error_message(), "unknown error");
    }

    public String generateIdentityJson() {
        return takeString(api.p4_generate_identity_json());
    }

    public String peerIdFromPublicKeyB64(String publicKeyB64) {
        return takeString(api.p4_peer_id_from_public_key_b64(publicKeyB64));
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
            api.p4_sign_envelope_json(
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
        byte ok = api.p4_verify_envelope_json(envelopeJson, signerPublicKeyB64, nowMs, maxSkewMs);
        if (ok == 1) {
            return true;
        }
        throw new RuntimeException(lastError());
    }

    private String readOwnedString(Pointer ptr, String fallback) {
        if (ptr == null) {
            return fallback;
        }
        try {
            return ptr.getString(0, "UTF-8");
        } finally {
            api.p4_free_string(ptr);
        }
    }

    private static String resolveDefaultLibraryPath() {
        String envPath = System.getenv(ENV_LIBRARY_PATH);
        if (envPath != null && !envPath.isBlank()) {
            return envPath;
        }

        PlatformTarget target = PlatformTarget.detect();
        String resourcePath = "/native/p4_core/" + target.nativeDir + "/" + target.fileName;
        String bundledPath = extractBundledLibrary(resourcePath, target.fileName);
        if (bundledPath != null) {
            return bundledPath;
        }

        Path repoNative = Path.of(
            System.getProperty("user.dir"),
            "native",
            "p4_core",
            target.nativeDir,
            target.fileName
        );
        if (Files.exists(repoNative)) {
            return repoNative.toAbsolutePath().toString();
        }

        throw new RuntimeException(
            "P⁴ native library not found for " + target.osLabel + "/" + target.archLabel +
            ". Set P4_CORE_LIB or use a build that includes bundled native binaries."
        );
    }

    private static String extractBundledLibrary(String resourcePath, String fileName) {
        try (InputStream input = P4Core.class.getResourceAsStream(resourcePath)) {
            if (input == null) {
                return null;
            }
            Path tempDir = Files.createTempDirectory("p4-core-jna-");
            Path tempLib = tempDir.resolve(fileName);
            Files.copy(input, tempLib, StandardCopyOption.REPLACE_EXISTING);
            tempLib.toFile().deleteOnExit();
            tempDir.toFile().deleteOnExit();
            return tempLib.toAbsolutePath().toString();
        } catch (IOException e) {
            throw new RuntimeException("Failed to extract bundled native library: " + e.getMessage(), e);
        }
    }

    private static final class PlatformTarget {
        private final String osLabel;
        private final String archLabel;
        private final String nativeDir;
        private final String fileName;

        private PlatformTarget(String osLabel, String archLabel, String nativeDir, String fileName) {
            this.osLabel = osLabel;
            this.archLabel = archLabel;
            this.nativeDir = nativeDir;
            this.fileName = fileName;
        }

        private static PlatformTarget detect() {
            String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
            String arch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
            if (os.contains("win")) {
                if (arch.contains("64")) {
                    return new PlatformTarget("windows", arch, "win32-x64", "p4_core.dll");
                }
            } else if (os.contains("mac") || os.contains("darwin")) {
                if (arch.contains("aarch64") || arch.contains("arm64")) {
                    return new PlatformTarget("darwin", arch, "darwin-arm64", "libp4_core.dylib");
                }
                if (arch.contains("x86_64") || arch.contains("amd64")) {
                    return new PlatformTarget("darwin", arch, "darwin-x64", "libp4_core.dylib");
                }
            } else if (os.contains("linux")) {
                if (arch.contains("x86_64") || arch.contains("amd64")) {
                    return new PlatformTarget("linux", arch, "linux-x64", "libp4_core.so");
                }
            }
            throw new RuntimeException("Unsupported platform: os=" + os + ", arch=" + arch);
        }
    }
}


