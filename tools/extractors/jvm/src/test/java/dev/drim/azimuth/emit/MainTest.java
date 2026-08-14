package dev.drim.azimuth.emit;

import dev.drim.azimuth.Azimuth;
import java.nio.file.Files;
import java.nio.file.Path;
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
                    @Azimuth.Covers(spec="polyglot/identity", scenario="java-identifies",
                        scope=Azimuth.Scope.unit, quantification=Azimuth.Quantification.example)
                    public static void identityTest() {}
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

        if (!manifest.contains("\"lang\":\"java\"")) throw new AssertionError(manifest);
        if (!manifest.contains("fixture.Identity.identity")) throw new AssertionError(manifest);
        if (!manifest.contains("\"scope\":\"unit\"")) throw new AssertionError(manifest);
    }
}
