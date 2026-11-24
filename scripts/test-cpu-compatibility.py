#!/usr/bin/env python3
"""Send a loopback POST while emulating a baseline x86-64 CPU."""

import argparse
import json
import os
import resource
import subprocess
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from threading import Thread


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    binary = parser.parse_args().binary.resolve()
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

    requests = []

    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            self.connection.settimeout(5)
            body = self.rfile.read(int(self.headers["Content-Length"]))
            requests.append((self.path, json.loads(body)))
            self.send_response(204)
            self.end_headers()

        def log_message(self, *_args):
            pass

    with HTTPServer(("127.0.0.1", 0), Handler) as server:
        worker = Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            result = subprocess.run(
                [
                    "qemu-x86_64", "-cpu", "qemu64", str(binary),
                    "--url", f"http://127.0.0.1:{server.server_port}/webhook",
                ],
                input="cpu compatibility probe\n",
                capture_output=True,
                text=True,
                timeout=30,
                env={**os.environ, "NO_PROXY": "127.0.0.1", "no_proxy": "127.0.0.1"},
            )
        finally:
            server.shutdown()
            worker.join()

    if result.returncode != 0:
        raise SystemExit(
            f"CPU compatibility check failed (exit {result.returncode}):\n{result.stderr}"
        )
    if requests != [("/webhook", {"content": "cpu compatibility probe\n", "args": ""})]:
        raise SystemExit(f"Unexpected webhook requests: {requests!r}")
    print(f"CPU compatibility check passed: {binary}")


if __name__ == "__main__":
    main()
