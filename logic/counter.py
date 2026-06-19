"""nui — the logic core (reference implementation, Python).

Transport-free: embeddable on iOS (CPython) and Android (Chaquopy), or hosted
by `logic/server.py` over a socket during dev.
"""

from __future__ import annotations

import json
import threading
from typing import Any, Callable


class App:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self.state: dict[str, Any] = {
            "count": 0,
            "step": 1,
            "show_label": True,
            "title": "Counter",
        }
        self.handlers: dict[str, Callable[[dict, dict], dict]] = {}

    def on(self, event: str):
        def register(fn: Callable[[dict, dict], dict]):
            self.handlers[event] = fn
            return fn

        return register

    def dispatch(self, event: str, payload: dict) -> dict:
        with self._lock:
            handler = self.handlers.get(event)
            if handler is None:
                print(f"[logic] no handler for event {event!r}")
                return dict(self.state)
            self.state = handler(self.state, payload)
            print(f"[logic] {event} -> {self.state}")
            return dict(self.state)

    def snapshot(self) -> dict:
        with self._lock:
            return dict(self.state)


app = App()


@app.on("increment")
def increment(state: dict, _payload: dict) -> dict:
    return {**state, "count": state["count"] + state["step"]}


@app.on("decrement")
def decrement(state: dict, _payload: dict) -> dict:
    return {**state, "count": state["count"] - state["step"]}


@app.on("reset")
def reset(state: dict, _payload: dict) -> dict:
    return {**state, "count": 0}


@app.on("set_step")
def set_step(state: dict, payload: dict) -> dict:
    value = int(payload.get("value", state["step"]))
    return {**state, "step": max(1, min(10, value))}


@app.on("set_show_label")
def set_show_label(state: dict, payload: dict) -> dict:
    return {**state, "show_label": bool(payload.get("value", state["show_label"]))}


@app.on("set_title")
def set_title(state: dict, payload: dict) -> dict:
    title = payload.get("value", state["title"])
    return {**state, "title": str(title) if title else "Counter"}


def initial_json() -> str:
    return json.dumps(app.snapshot())


def dispatch_json(event: str, payload_json: str = "") -> str:
    payload = json.loads(payload_json) if payload_json else {}
    return json.dumps(app.dispatch(event, payload))
