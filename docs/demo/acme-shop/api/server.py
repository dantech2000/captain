import http.server, json, time

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = json.dumps({"service": "acme-shop api", "time": time.time()}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(body)

print("api listening on :8000")
http.server.HTTPServer(("", 8000), Handler).serve_forever()
