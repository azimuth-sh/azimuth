#include "identity.hpp"
#include <cstdlib>
#include <cstring>
#include <netinet/in.h>
#include <string>
#include <sys/socket.h>
#include <unistd.h>

int main() {
    const char* configured = std::getenv("PORT");
    int port = configured == nullptr ? 8087 : std::stoi(configured);
    int server = socket(AF_INET, SOCK_STREAM, 0);
    sockaddr_in address{};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    address.sin_port = htons(static_cast<uint16_t>(port));
    if (bind(server, reinterpret_cast<sockaddr*>(&address), sizeof(address)) != 0
        || listen(server, 8) != 0) {
        return 1;
    }
    while (true) {
        int client = accept(server, nullptr, nullptr);
        char request[1024]{};
        ssize_t count = read(client, request, sizeof(request));
        bool found = count > 0 && std::strncmp(request, "GET /identity ", 14) == 0;
        std::string body = found ? std::string(identity()) + "\n" : "";
        std::string response = std::string("HTTP/1.1 ")
            + (found ? "200 OK" : "404 Not Found")
            + "\r\nContent-Length: " + std::to_string(body.size()) + "\r\n\r\n" + body;
        write(client, response.data(), response.size());
        close(client);
    }
}
