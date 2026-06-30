package com.rucraft;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Locale;

final class NativeLoader {
    private static boolean loaded;

    private NativeLoader() {
    }

    static synchronized void load() {
        if (loaded) {
            return;
        }

        String platform = platform();
        String libraryName = System.mapLibraryName("rucraft_native");
        String resourcePath = "/natives/" + platform + "/" + libraryName;

        try (InputStream input = NativeLoader.class.getResourceAsStream(resourcePath)) {
            if (input == null) {
                throw new UnsatisfiedLinkError("Native library not found in mod jar: " + resourcePath);
            }

            Path tempDirectory = Files.createTempDirectory("rucraft-natives");
            Path nativeLibrary = tempDirectory.resolve(libraryName);

            Files.copy(input, nativeLibrary, StandardCopyOption.REPLACE_EXISTING);

            tempDirectory.toFile().deleteOnExit();
            nativeLibrary.toFile().deleteOnExit();

            System.load(nativeLibrary.toAbsolutePath().toString());
            loaded = true;
        } catch (IOException exception) {
            UnsatisfiedLinkError error = new UnsatisfiedLinkError("Failed to extract native library: " + resourcePath);
            error.initCause(exception);
            throw error;
        }
    }

    private static String platform() {
        return osName() + "-" + archName();
    }

    private static String osName() {
        String os = System.getProperty("os.name").toLowerCase(Locale.ROOT);

        if (os.contains("win")) {
            return "windows";
        }

        if (os.contains("mac")) {
            return "macos";
        }

        if (os.contains("linux")) {
            return "linux";
        }

        throw new UnsupportedOperationException("Unsupported OS: " + os);
    }

    private static String archName() {
        String arch = System.getProperty("os.arch").toLowerCase(Locale.ROOT);

        if (arch.equals("amd64") || arch.equals("x86_64")) {
            return "x86_64";
        }

        if (arch.equals("aarch64") || arch.equals("arm64")) {
            return "aarch64";
        }

        throw new UnsupportedOperationException("Unsupported architecture: " + arch);
    }
}
