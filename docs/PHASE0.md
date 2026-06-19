# Phase 0 — prove the loop

Goal: a real SwiftUI counter on the iOS simulator whose `count` is owned by a
**Python** process, communicating over a local socket. This de-risks the core
bet of nui — *platform-reconciled UI + off-thread logic over a thin contract* —
before any DSL or codegen exists.

## What's here

| Path | Role |
|------|------|
| `logic/counter.py` | The logic core. Owns state, transport-free. |
| `logic/server.py` | Dev transport: a TCP socket host around the core. |
| `ios/Sources/Bridge.swift` | Thin client: mirrors state down, sends events up. |
| `ios/Sources/ContentView.swift` | The UI projection (real SwiftUI). |
| `PROTOCOL.md` | The wire contract. |

> Historical note: as of **Phase 3** the iOS `Bridge` embeds Python in-process
> (no socket). The socket transport below now lives in `logic/server.py` and is
> still the simplest way to drive a UI from a separate process during dev.

## Run it

Two terminals.

**Terminal 1 — the logic (start this first):**

```bash
python3 logic/server.py
```

**Terminal 2 — the app:**

```bash
./run.sh
```

`run.sh` generates the Xcode project, boots the `iPhone 17` simulator, builds,
installs, and launches. Override the device with `SIM_NAME="iPhone 17 Pro" ./run.sh`.

Tap **+** / **−** in the simulator. The number changes because Python changed
it — the tap becomes an event, Python mutates the source-of-truth state, and the
new snapshot flows back to SwiftUI, which re-renders.

### Try this to feel the architecture

Launch the app on **two** simulators at once. Tap **+** on one — the other
updates too, because there is only one source of truth (the Python process), and
every state change is broadcast to all connected UIs.

## What this proves (and what it doesn't)

Proves:
- The UI can be a pure projection of externally-owned state.
- Logic on its own thread/process keeps the UI thread free.
- A single language-agnostic contract is enough to wire them together.

Does **not** yet address (later phases):
- The `.nui` DSL and SwiftUI/Compose codegen.
- On-device, in-process logic (no socket).
- Latency-critical gestures handled natively (Lynx-style escape hatch).
