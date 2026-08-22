package sh.azimuth.emit;

import sh.azimuth.Azimuth;
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
                import sh.azimuth.Azimuth;
                public final class Identity {
                    @Azimuth.Realizes(spec="polyglot/identity", scenario="java-identifies")
                    public static String identity() { return "java"; }
                    @Azimuth.ImplementsCheck("polyglot/java-identity")
                    public static void identityTest() {}
                    @Azimuth.ImplementsCheck("polyglot/java-identity")
                    public static void identityContractTest() { int observed = 1; }
                    @Azimuth.ImplementsMechanism(
                            spec="polyglot/identity", mechanism="guard-string")
                    public static void guard(String value) {}
                    @Azimuth.ImplementsMechanism(
                            spec="polyglot/identity", mechanism="guard-integer")
                    public static void guard(int value) {}
                    public static void ordinaryTest() {}
                    public static final class Nested {
                        @Azimuth.ImplementsMechanism(
                                spec="polyglot/identity", mechanism="nested-guard")
                        public static void guard(java.util.UUID value) {}
                    }
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
                root.resolve("manifest.json"), root,
                java.util.List.of(sourceRoot), java.util.List.of(classes)));
        String repeated = Main.emit(new Main.Options(
                root.resolve("manifest.json"), root,
                java.util.List.of(sourceRoot), java.util.List.of(classes)));
        String relocated = Main.emit(new Main.Options(
                root.resolve("relocated.json"), sourceRoot,
                java.util.List.of(sourceRoot), java.util.List.of(classes)));

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
        String stringSite = "fixture.Identity.guard(Ljava/lang/String;)V";
        String integerSite = "fixture.Identity.guard(I)V";
        String nestedSite = "fixture.Identity$Nested.guard(Ljava/util/UUID;)V";
        for (String site : java.util.List.of(stringSite, integerSite, nestedSite)) {
            if (!manifest.contains("\"site\":\"" + site + "\"")) {
                throw new AssertionError(
                        "missing qualified mechanism site " + site + "\n" + manifest);
            }
            if (!manifest.contains("\"binding\":\"java-symbol:" + site + "\"")) {
                throw new AssertionError("missing typed binding for " + site + "\n" + manifest);
            }
            if (!manifest.contains("{\"id\":\"java-symbol:" + site
                    + "\",\"kind\":\"java-symbol\",\"file\":\"src/fixture/Identity.java\"}")) {
                throw new AssertionError("missing exact companion for " + site + "\n" + manifest);
            }
        }
        String strictImplementation = "\\{\\\"spec\\\":\\\"polyglot/identity\\\","
                + "\\\"mechanism\\\":\\\"guard-string\\\","
                + "\\\"site\\\":\\\"fixture\\.Identity\\.guard\\(Ljava/lang/String;\\)V\\\","
                + "\\\"binding\\\":\\\"java-symbol:fixture\\.Identity\\.guard"
                + "\\(Ljava/lang/String;\\)V\\\","
                + "\\\"file\\\":\\\"src/fixture/Identity\\.java\\\","
                + "\\\"lang\\\":\\\"java\\\","
                + "\\\"source_fingerprint\\\":\\\"sha256:[0-9a-f]{64}\\\"\\}";
        if (!Pattern.compile(strictImplementation).matcher(manifest).find()) {
            throw new AssertionError(
                    "mechanism implementation is not the exact strict shape\n" + manifest);
        }
        String baselineObject = mechanismObject(manifest, "guard-string");
        String relocatedObject = mechanismObject(relocated, "guard-string");
        if (!baselineObject.replace("src/fixture/Identity.java", "fixture/Identity.java")
                .equals(relocatedObject)) {
            throw new AssertionError("relocation changed semantic mechanism identity");
        }
        if (baselineObject.equals(relocatedObject)) {
            throw new AssertionError("relocation did not change the accountable file locator");
        }
        Path collisionRoot = root.resolve("collision");
        Path collisionSources = collisionRoot.resolve("src");
        Path collisionClasses = collisionRoot.resolve("classes");
        Files.createDirectories(collisionSources.resolve("collision"));
        Files.createDirectories(collisionClasses);
        Path collisionSource = collisionSources.resolve("collision/Site.java");
        Files.writeString(collisionSource, """
                package collision;
                import sh.azimuth.Azimuth;
                public final class Site {
                    @Azimuth.ImplementsMechanism(spec="alpha", mechanism="first")
                    @Azimuth.ImplementsMechanism(spec="alpha", mechanism="second")
                    public static void guard() {}
                }
                """);
        int collisionCompiled = ToolProvider.getSystemJavaCompiler().run(
                null, null, null, "-cp", System.getProperty("java.class.path"),
                "-d", collisionClasses.toString(), collisionSource.toString());
        if (collisionCompiled != 0) throw new AssertionError("collision fixture did not compile");
        try {
            Main.emit(new Main.Options(
                    collisionRoot.resolve("manifest.json"), collisionRoot,
                    java.util.List.of(collisionSources), java.util.List.of(collisionClasses)));
            throw new AssertionError("ambiguous mechanism target was accepted");
        } catch (IllegalArgumentException expected) {
            if (!expected.getMessage().contains("cannot implement several mechanisms")) {
                throw expected;
            }
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

    private static String mechanismObject(String manifest, String mechanism) {
        Matcher matcher = Pattern.compile(
                "\\{\\\"spec\\\":\\\"[^\\\"]+\\\",\\\"mechanism\\\":\\\""
                        + Pattern.quote(mechanism) + "\\\"[^}]+\\}")
                .matcher(manifest);
        if (!matcher.find()) throw new AssertionError("missing mechanism " + mechanism);
        return matcher.group();
    }
}
