package polyglot;

public final class IdentityServiceTest {
    private IdentityServiceTest() {}

    public static void main(String[] arguments) {
        if (!IdentityService.identity().equals("java")) {
            throw new AssertionError("Java identity changed");
        }
    }
}
