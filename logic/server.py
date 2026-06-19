"""nui dev transport — a TCP socket host around the logic core.

This is the Phase 0-2 development transport. It imports the SAME logic core
(`counter.py`) that the iOS/Android apps embed in-process in Phase 3, and simply
exposes it over a newline-delimited JSON socket. See PROTOCOL.md.

Run:  python3 logic/server.py
"""

from __future__ import annotations

import json
import socket
import threading

from counter import app

HOST = "127.0.0.1"
PORT = 7000


class Server:
    def __init__(self, host: str = HOST, port: int = PORT) -> None:
        self.host = host
        self.port = port
        self._clients: set[socket.socket] = set()
        self._clients_lock = threading.Lock()

    def _send(self, conn: socket.socket, message: dict) -> None:
        line = (json.dumps(message) + "\n").encode("utf-8")
        try:
            conn.sendall(line)
        except OSError:
            self._drop(conn)

    def _broadcast_state(self) -> None:
        message = {"type": "state", "state": app.snapshot()}
        with self._clients_lock:
            clients = list(self._clients)
        for conn in clients:
            self._send(conn, message)

    def _drop(self, conn: socket.socket) -> None:
        with self._clients_lock:
            self._clients.discard(conn)
        try:
            conn.close()
        except OSError:
            pass

    def _handle_client(self, conn: socket.socket, addr) -> None:
        print(f"[server] UI connected: {addr}")
        with self._clients_lock:
            self._clients.add(conn)
        self._send(conn, {"type": "state", "state": app.snapshot()})

        buffer = b""
        try:
            while True:
                chunk = conn.recv(4096)
                if not chunk:
                    break
                buffer += chunk
                while b"\n" in buffer:
                    raw, buffer = buffer.split(b"\n", 1)
                    if not raw.strip():
                        continue
                    self._handle_message(raw)
        except OSError:
            pass
        finally:
            print(f"[server] UI disconnected: {addr}")
            self._drop(conn)

    def _handle_message(self, raw: bytes) -> None:
        try:
            message = json.loads(raw)
        except json.JSONDecodeError:
            print(f"[server] bad message: {raw!r}")
            return
        if message.get("type") != "event":
            return
        name = message.get("name", "")
        payload = message.get("payload") or {}
        app.dispatch(name, payload)
        self._broadcast_state()

    def serve_forever(self) -> None:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            sock.bind((self.host, self.port))
            sock.listen()
            print(f"[server] nui logic listening on {self.host}:{self.port}")
            print("[server] waiting for the UI to connect...")
            while True:
                conn, addr = sock.accept()
                threading.Thread(
                    target=self._handle_client, args=(conn, addr), daemon=True
                ).start()


if __name__ == "__main__":
    try:
        Server().serve_forever()
    except KeyboardInterrupt:
        print("\n[server] bye")
