package polyglot;

import dev.drim.azimuth.Azimuth;

public final class IdentityServiceTest {
    private IdentityServiceTest() {}

    @Azimuth.Covers(
            spec = "polyglot/identity",
            scenario = "java-identifies",
            scope = Azimuth.Scope.unit,
            quantification = Azimuth.Quantification.example,
            oracle = Azimuth.Oracle.direct)
    public static void main(String[] arguments) {
        if (!IdentityService.identity().equals("java")) {
            throw new AssertionError("Java identity changed");
        }
    }
}
