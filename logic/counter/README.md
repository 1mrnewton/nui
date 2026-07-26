# counter-logic

The Rust side of the nui counter example: the pure, typed functions declared
in the `.nui` logic block, exposed to Swift through UniFFI and shipped as an
XCFramework. No globals, no lifecycle — nui owns the state; this crate owns
the logic.

The interface checks (`src/generated.rs`) and the Swift adapter
(`swift/RustCounterLogic.swift`) are generated from the same `.nui` file as
the UI, so nothing can drift — a missing or misdeclared function fails the
build:

```sh
# from the repo root, after changing examples/counter.nui:
nui build examples/counter.nui -o CounterView.swift
nui build examples/counter.nui --target rust -o logic/counter/src/generated.rs
nui build examples/counter.nui --target swift-bridge -o logic/counter/swift/RustCounterLogic.swift
```

The only handwritten code is the function bodies in `src/lib.rs`:

```rust
#[uniffi::export]
pub fn counter_increment(count: i64) -> i64 {
    count.saturating_add(1)
}
```

```sh
cargo test               # logic is testable headless, no simulator needed
./build-xcframework.sh   # → CounterLogic.xcframework + generated/counter_logic.swift
```

## Running the counter on an iPhone — all from the command line

```sh
# from the repo root:
nui build examples/counter.nui -o CounterView.swift   # 1. generate the UI
logic/counter/build-xcframework.sh                    # 2. build the Rust logic

cd logic/counter/app
xcodegen generate                                     # 3. create the Xcode project

# 4. build, install, launch in a simulator (pick any iPhone UDID from
#    `xcrun simctl list devices available`):
xcodebuild -project CounterApp.xcodeproj -scheme CounterApp \
    -destination 'platform=iOS Simulator,name=iPhone 17' \
    -derivedDataPath build build
xcrun simctl boot <UDID>
xcrun simctl install <UDID> build/Build/Products/Debug-iphonesimulator/CounterApp.app
xcrun simctl launch <UDID> dev.nui.CounterApp
```

Or open `CounterApp.xcodeproj` in Xcode and hit Run. The buttons you tap are
SwiftUI; the arithmetic happens in Rust.

The end-to-end loop (tap → event → Rust via FFI → new state → re-render) is
covered by a UI test:

```sh
xcodebuild test -project CounterApp.xcodeproj -scheme CounterApp \
    -destination 'platform=iOS Simulator,name=iPhone 17' -derivedDataPath build
```

## Layout

```
src/lib.rs                 the handwritten logic: function bodies + tests
src/generated.rs           (nui-generated) expected signatures + build checks
src/bin/uniffi-bindgen.rs  UniFFI's binding generator CLI
build-xcframework.sh       iOS device + simulator build, bindings, packaging
swift/CounterApp.swift     host-app entry point
swift/RustCounterLogic.swift  (nui-generated) UI↔logic adapter
app/project.yml            XcodeGen spec for the demo app
app/UITests/               taps the buttons, asserts on Rust-computed state
generated/                 (build output) Swift bindings
CounterLogic.xcframework   (build output) compiled Rust for iOS
app/CounterApp.xcodeproj   (generated) by xcodegen
```

Types here are prefixed `CounterLogic*` so the UniFFI-generated Swift can
live in the same app target as the nui-generated `CounterState` (and
SwiftUI's `State`) without name collisions. `src/generated.rs` and the
bridge are regenerated from the `.nui` file, so the two sides cannot drift.
