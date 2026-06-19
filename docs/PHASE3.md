# Phase 3 — embed the logic in-process

Goal: prove the whole concept works as a **real, self-contained app** on both
platforms — the Python logic running *inside* the app, with **no socket and no
desktop process** — before growing the DSL vocabulary.

Phases 0–2 used a Python process on your Mac over a TCP socket. That's fine for
dev, not shippable. Phase 3 removes the cheat.

## What changed

Almost nothing in the parts that matter, which is the whole point.

```
            Phases 0–2                         Phase 3
   ┌───────────────────────┐         ┌───────────────────────────┐
   │  native UI (generated)│         │  native UI (generated)    │   ← unchanged
   ├───────────────────────┤         ├───────────────────────────┤
   │  Bridge: onState/send │         │   Bridge: onState/send    │   ← same API
   │      via TCP socket    │   →    │   via in-process Python   │   ← new transport
   ├───────────────────────┤         ├───────────────────────────┤
   │  Python (your Mac)    │         │  Python (embedded in app) │
   └───────────────────────┘         └───────────────────────────┘
```

- Generated views (`CounterView.swift` / `CounterView.kt`): **unchanged.**
- `Bridge`'s public surface (`onState`, `send`, `connect`): **identical.** Only
  the implementation swapped.
- The logic (`counter.py`): **the same file**, now loaded in-process.

## The logic is now transport-free

`logic/counter.py` is the pure core — state + handlers, no transport. It exposes:

- `initial_json() -> str`
- `dispatch_json(event, payload_json) -> str`

Hosted three ways from the **same source**:

| Host | How |
|------|-----|
| `logic/server.py` | Dev socket transport (optional) |
| iOS app | CPython C API via BeeWare xcframework |
| Android app | Chaquopy Gradle plugin |

The in-process boundary is strings in / strings out: the same JSON contract as
`PROTOCOL.md`, but as function calls instead of a socket.

---

## iOS

### Wiring

1. **CPython runtime** — `Python.xcframework` (BeeWare
   [Python-Apple-support](https://github.com/beeware/Python-Apple-support), 3.13,
   [PEP 730](https://peps.python.org/pep-0730/)).
2. **C API → Swift** — bridging header `ios/Sources/PythonBridging.h`.
3. **Standard library** — `ios/scripts/install_python_stdlib.sh` copies stdlib into
   the bundle; `PythonRuntime.swift` sets `PYTHONHOME` / `PYTHONPATH`.
4. **Logic** — pre-build step copies `logic/counter.py` into `Resources/app/`.
5. **Calls** — `Bridge.swift` on a serial queue, GIL-guarded.

### Simulator vs device

On **device**, each stdlib `.so` is repackaged as a signed `.framework` (PEP 730
`.fwork`). On **simulator**, plain `.so` modules load directly.

### Run (iOS)

```bash
./run.sh   # simulator
```

Physical device:

```bash
DEV=$(xcrun devicectl list devices | awk '/available/{print $(NF-3); exit}')
xcodebuild -project ios/NuiCounter.xcodeproj -scheme NuiCounter \
  -configuration Debug -destination 'generic/platform=iOS' \
  -derivedDataPath ios/.build -allowProvisioningUpdates build
xcrun devicectl device install app --device "$DEV" \
  ios/.build/Build/Products/Debug-iphoneos/NuiCounter.app
xcrun devicectl device process launch --device "$DEV" dev.nui.counter
```

Dev autodrive (simulator): `xcrun simctl launch "iPhone 17" dev.nui.counter -nui-autodrive`

Verified on iPhone 16 Pro Max (real device).

---

## Android

### Wiring

1. **Chaquopy 17.0** — Gradle plugin; supports AGP 8.9 and Python 3.13.
2. **Startup** — `NuiApplication` calls `Python.start(AndroidPlatform(this))`.
3. **Logic** — `preBuild` syncs `logic/counter.py` → `app/src/main/python/`.
4. **Calls** — `Bridge.kt` on a single-thread executor; `getModule("counter")`
   then `callAttr("initial_json")` / `callAttr("dispatch_json", event)`.

### Build requirements

Chaquopy needs **Python 3.13 on the build machine** (major.minor must match the
app). Install via Homebrew: `brew install python@3.13`. The Gradle config tries
common Homebrew paths, then falls back to `python3.13` on `PATH`.

### Run (Android)

No `logic/server.py` needed.

```bash
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
cd android && gradle :app:installDebug --no-daemon
$ANDROID_HOME/platform-tools/adb shell am start -n dev.nui.counter/dev.nui.MainActivity
```

Dev autodrive:

```bash
adb shell am start -n dev.nui.counter/dev.nui.MainActivity --ez nui-autodrive true
```

Verified on Pixel_5 emulator (arm64): logcat shows `[logic] increment -> {'count': 5}`
with no socket server running.

---

## What this proves (and what it doesn't)

**Proves:**
- Logic ships **inside** the app on iOS and Android — no external process.
- The state/event contract is transport-agnostic.
- Generated UI never needs to know how logic is hosted.

**Does not yet address:**
- Phase 4: a second logic language (Rust) and a real FFI ABI instead of JSON strings.
- Growing the `.nui` vocabulary.

`logic/server.py` remains useful for quick dev experiments or driving a UI over
the network, but neither mobile app needs it anymore.
