from http.server import HTTPServer, SimpleHTTPRequestHandler, test
import sys
import ssl


class RequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        SimpleHTTPRequestHandler.end_headers(self)


if __name__ == '__main__':
    # settings.py
    import mimetypes
    mimetypes.add_type("text/javascript", ".js", True)
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    httpd = HTTPServer(('127.0.0.1', port), RequestHandler)
    httpd.socket = ssl.wrap_socket(
        httpd.socket, certfile='../cert.pem', keyfile='../key.pem', server_side=True)
    httpd.serve_forever()
