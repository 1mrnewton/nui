# Phase 2 — Compose codegen (Android)

The same [`examples/counter.nui`](../examples/counter.nui) compiles to native
Jetpack Compose and runs on Android. One DSL, two native frameworks.

> **Phase 3 update:** the Android app now embeds Python in-process via Chaquopy.
> You no longer need `logic/server.py` or the `10.0.2.2:7000` socket. See
> [`PHASE3.md`](./PHASE3.md) for the current run instructions.

## What's here

| Path | Role |
|------|------|
| `compiler/src/emit_kotlin.rs` | The Compose emitter (second backend). |
| `android/` | Gradle Compose app with Chaquopy (Phase 3). |
| `android/app/.../runtime/Bridge.kt` | In-process Python bridge (same API as iOS). |
| `android/app/.../generated/CounterView.kt` | **Generated** from `counter.nui`. |
| `android/app/.../MainActivity.kt` | Hosts the generated view. |

## Build & run (Phase 3 — in-process)

Requires Python 3.13 on the build machine (`brew install python@3.13`).

```bash
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"

# emulator (if needed)
$ANDROID_HOME/emulator/emulator -avd Pixel_5 -no-snapshot &

# build + install + launch — no logic server
cd android && gradle :app:installDebug --no-daemon
$ANDROID_HOME/platform-tools/adb shell am start -n dev.nui.counter/dev.nui.MainActivity
```

## Legacy: socket transport (Phase 0–2)

For dev experiments, you can still run `python3 logic/server.py` and point an
old socket-based `Bridge` at `10.0.2.2:7000`. The committed app no longer uses
this path.

## Toolchain notes

- Chaquopy 17.0, Gradle 8.13, AGP 8.9.1, Kotlin 2.1.0, Compose BOM 2024.12.01,
  compileSdk 36, embedded Python 3.13.
- `android.suppressUnsupportedCompileSdk=36` silences the new-SDK warning.
- State decoding uses `kotlinx.serialization`; generated `CounterState` is
  `@Serializable` and `Bridge.onState<T>` is reified.
