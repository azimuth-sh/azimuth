from http.server import BaseHTTPRequestHandler, HTTPServer
import os

from azimuth_annotations import realizes


@realizes("polyglot/identity", "python-identifies")
def identity() -> str:
    return "python"


class Handler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        if self.path != "/identity":
            self.send_error(404)
            return
        body = f"{identity()}\n".encode()
        self.send_response(200)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


if __name__ == "__main__":
    HTTPServer(("127.0.0.1", int(os.getenv("PORT", "8084"))), Handler).serve_forever()
