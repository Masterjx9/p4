package io.github.masterjx9.pp2p;

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

public final class Pp2pCore {
    private static final String ENV_LIBRARY_PATH = "PP2P_CORE_LIB";

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

    public Pp2pCore() {
        this(resolveDefaultLibraryPath());
    }

    public Pp2pCore(String libraryPath) {
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
        return readOwnedString(api.pp2p_last_error_message(), "unknown error");
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

    private String readOwnedString(Pointer ptr, String fallback) {
        if (ptr == null) {
            return fallback;
        }
        try {
            return ptr.getString(0, "UTF-8");
        } finally {
            api.pp2p_free_string(ptr);
        }
    }

    private static String resolveDefaultLibraryPath() {
        String envPath = System.getenv(ENV_LIBRARY_PATH);
        if (envPath != null && !envPath.isBlank()) {
            return envPath;
        }

        PlatformTarget target = PlatformTarget.detect();
        String resourcePath = "/native/pp2p_core/" + target.nativeDir + "/" + target.fileName;
        String bundledPath = extractBundledLibrary(resourcePath, target.fileName);
        if (bundledPath != null) {
            return bundledPath;
        }

        Path repoNative = Path.of(
            System.getProperty("user.dir"),
            "native",
            "pp2p_core",
            target.nativeDir,
            target.fileName
        );
        if (Files.exists(repoNative)) {
            return repoNative.toAbsolutePath().toString();
        }

        throw new RuntimeException(
            "PP2P native library not found for " + target.osLabel + "/" + target.archLabel +
            ". Set PP2P_CORE_LIB or use a build that includes bundled native binaries."
        );
    }

    private static String extractBundledLibrary(String resourcePath, String fileName) {
        try (InputStream input = Pp2pCore.class.getResourceAsStream(resourcePath)) {
            if (input == null) {
                return null;
            }
            Path tempDir = Files.createTempDirectory("pp2p-core-jna-");
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
                    return new PlatformTarget("windows", arch, "win32-x64", "pp2p_core.dll");
                }
            } else if (os.contains("mac") || os.contains("darwin")) {
                if (arch.contains("aarch64") || arch.contains("arm64")) {
                    return new PlatformTarget("darwin", arch, "darwin-arm64", "libpp2p_core.dylib");
                }
                if (arch.contains("x86_64") || arch.contains("amd64")) {
                    return new PlatformTarget("darwin", arch, "darwin-x64", "libpp2p_core.dylib");
                }
            } else if (os.contains("linux")) {
                if (arch.contains("x86_64") || arch.contains("amd64")) {
                    return new PlatformTarget("linux", arch, "linux-x64", "libpp2p_core.so");
                }
            }
            throw new RuntimeException("Unsupported platform: os=" + os + ", arch=" + arch);
        }
    }
}
