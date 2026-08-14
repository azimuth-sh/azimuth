package polyglot

import com.sun.net.httpserver.HttpServer
import dev.drim.azimuth.Azimuth
import java.net.InetSocketAddress

object IdentityService {
    @JvmStatic
    @Azimuth.Realizes(spec = "polyglot/identity", scenario = "kotlin-identifies")
    fun identity(): String = "kotlin"

    @JvmStatic
    fun main(arguments: Array<String>) {
        val port = System.getenv("PORT")?.toInt() ?: 8083
        val server = HttpServer.create(InetSocketAddress(port), 0)
        server.createContext("/identity") { exchange ->
            val body = "${identity()}\n".toByteArray()
            exchange.sendResponseHeaders(200, body.size.toLong())
            exchange.responseBody.use { it.write(body) }
        }
        server.start()
    }
}
