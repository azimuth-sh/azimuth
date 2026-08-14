package polyglot;

import com.sun.net.httpserver.HttpServer;
import dev.drim.azimuth.Azimuth;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;

public final class IdentityService {
    private IdentityService() {}

    @Azimuth.Realizes(spec = "polyglot/identity", scenario = "java-identifies")
    public static String identity() {
        return "java";
    }

    public static void main(String[] arguments) throws Exception {
        int port = Integer.parseInt(System.getenv().getOrDefault("PORT", "8082"));
        HttpServer server = HttpServer.create(new InetSocketAddress(port), 0);
        server.createContext("/identity", exchange -> {
            byte[] body = (identity() + "\n").getBytes(StandardCharsets.UTF_8);
            exchange.sendResponseHeaders(200, body.length);
            exchange.getResponseBody().write(body);
            exchange.close();
        });
        server.start();
    }
}
