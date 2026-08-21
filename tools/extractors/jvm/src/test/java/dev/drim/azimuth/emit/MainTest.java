package dev.drim.azimuth.emit;

import dev.drim.azimuth.Azimuth;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import javax.tools.ToolProvider;

public final class MainTest {
    private MainTest() {}

    public static void main(String[] arguments) throws Exception {
        Path root = Files.createTempDirectory("azimuth-jvm-");
        Path sourceRoot = root.resolve("src");
        Path classes = root.resolve("classes");
        Files.createDirectories(sourceRoot.resolve("fixture"));
        Files.createDirectories(sourceRoot.resolve("another"));
        Files.createDirectories(classes);
        Path source = sourceRoot.resolve("fixture/Identity.java");
        Files.writeString(source, """
                package fixture;
                import dev.drim.azimuth.Azimuth;
                public final class Identity {
                    @Azimuth.Realizes(spec="polyglot/identity", scenario="java-identifies")
                    public static String identity() { return "java"; }
                    @Azimuth.ImplementsCheck("polyglot/java-identity")
                    public static void identityTest() {}
                    @Azimuth.ImplementsCheck("polyglot/java-identity")
                    public static void identityContractTest() { int observed = 1; }
                    public static void ordinaryTest() {}
                }
                """);
        Path sameNameInAnotherPackage = sourceRoot.resolve("another/Identity.java");
        Files.writeString(sameNameInAnotherPackage, """
                package another;
                public final class Identity {}
                """);
        int compiled = ToolProvider.getSystemJavaCompiler().run(
                null, null, null, "-cp", System.getProperty("java.class.path"),
                "-d", classes.toString(), source.toString(), sameNameInAnotherPackage.toString());
        if (compiled != 0) throw new AssertionError("fixture did not compile");

        String manifest = Main.emit(new Main.Options(
                root.resolve("manifest.json"), root, java.util.List.of(sourceRoot), java.util.List.of(classes)));
        String repeated = Main.emit(new Main.Options(
                root.resolve("manifest.json"), root, java.util.List.of(sourceRoot), java.util.List.of(classes)));

        if (!manifest.contains("\"lang\":\"java\"")) throw new AssertionError(manifest);
        if (!manifest.contains("fixture.Identity.identity")) throw new AssertionError(manifest);
        if (!manifest.equals(repeated)) throw new AssertionError("manifest was not deterministic");
        if (count(manifest, "\"check\":\"polyglot/java-identity\"") != 2) {
            throw new AssertionError(manifest);
        }
        if (manifest.contains("\"site\":\"fixture.Identity.ordinaryTest\"")) {
            throw new AssertionError(manifest);
        }
        if (!manifest.contains("\"check_implementations\"")) throw new AssertionError(manifest);
        if (manifest.contains("\"covers\"") || manifest.contains("\"mechanism_covers\"")
                || manifest.contains("\"observations\"")) {
            throw new AssertionError(manifest);
        }
        Matcher sourceFingerprints = Pattern.compile(
                "\\\"source_fingerprint\\\":\\\"([^\\\"]+)\\\"").matcher(manifest);
        int sourceFingerprintCount = 0;
        while (sourceFingerprints.find()) {
            sourceFingerprintCount++;
            if (!sourceFingerprints.group(1).matches("sha256:[0-9a-f]{64}")) {
                throw new AssertionError(manifest);
            }
        }
        if (sourceFingerprintCount < 3) throw new AssertionError(manifest);
        Matcher checkFingerprints = Pattern.compile(
                "\\\"check\\\":\\\"polyglot/java-identity\\\"[^}]+"
                        + "\\\"source_fingerprint\\\":\\\"(sha256:[0-9a-f]{64})\\\"")
                .matcher(manifest);
        if (!checkFingerprints.find()) throw new AssertionError(manifest);
        String first = checkFingerprints.group(1);
        if (!checkFingerprints.find() || first.equals(checkFingerprints.group(1))) {
            throw new AssertionError("implementation sites did not have distinct exact hashes");
        }
        if (Arrays.stream(Azimuth.class.getDeclaredClasses())
                .map(Class::getSimpleName)
                .anyMatch(name -> name.equals("Covers") || name.equals("CoversMechanism")
                        || name.equals("Coverage") || name.equals("MechanismCoverage"))) {
            throw new AssertionError("alpha 1 annotations remain public");
        }
    }

    private static int count(String source, String value) {
        int count = 0;
        for (int index = 0; (index = source.indexOf(value, index)) >= 0;
                index += value.length()) {
            count++;
        }
        return count;
    }
}
