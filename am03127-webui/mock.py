"""In-memory stand-in for the panel's REST API, for UI development without hardware.

Stores whatever JSON it is sent; it does not validate against the firmware models.
Usage: python3 mock.py [port]   (default 3000)
"""

import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

STATUS = {
    "version": "mock",
    "build_time": "00:00:00",
    "build_date": "2026-01-01",
    "uptime_secs": 93784,
    "rssi": -61,
    "heap_free": 53248,
    "heap_used": 49152,
    "panel_ok": True,
}
store = {"page": {}, "schedule": {}}


class Handler(BaseHTTPRequestHandler):
    def reply(self, code, body=None):
        data = b"" if body is None else json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def body(self):
        return self.rfile.read(int(self.headers.get("Content-Length", 0)))

    def route(self):
        # "/page/A" -> ("page", "A"), "/pages" -> ("pages", None)
        parts = self.path.strip("/").split("/")
        return parts[0], parts[1] if len(parts) > 1 else None

    def do_GET(self):
        kind, id = self.route()
        if kind == "status":
            return self.reply(200, STATUS)
        if kind in ("pages", "schedules"):
            return self.reply(200, list(store[kind[:-1]].values()))
        if kind in store and id in store[kind]:
            return self.reply(200, store[kind][id])
        self.reply(404)

    def do_POST(self):
        kind, id = self.route()
        body = self.body()
        if kind in store and id:
            store[kind][id] = json.loads(body)
        elif kind in ("pages", "schedules"):
            store[kind[:-1]] = {item["id"]: item for item in json.loads(body)}
        elif kind == "reset":
            for items in store.values():
                items.clear()
        elif kind != "clock":
            return self.reply(404)
        self.reply(200)

    def do_DELETE(self):
        kind, id = self.route()
        if kind in store and store[kind].pop(id, None) is not None:
            return self.reply(200)
        self.reply(404)

    def do_PUT(self):
        if self.route()[0] != "ota":
            return self.reply(404)
        print(f"OTA: received {len(self.body())} bytes")
        self.reply(200)


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
    print(f"Mock panel on http://127.0.0.1:{port}")
    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
