# nui

> Just another UI framework — write your UI once in a declarative DSL (`.nui`),
> write your logic in any language. Compiles to **native** SwiftUI (iOS) and
> Jetpack Compose (Android).

This is an exploration / learning project. It does **not** yet promise a usable
UI framework. Things will change. Nothing here is stable.

## The idea

```
State  ──►  View = f(State)  ──►  user interacts  ──►  Event  ──►  Logic  ──►  new State  ──►  (loop)
```

nui is built on the **MVU** (Model–View–Update) model, which is also how SwiftUI
and Compose work internally:

- **Logic owns the state.** It is the single source of truth.
- **The UI is a pure projection** of that state. It never holds business state;
  it only renders and emits events.
- **Logic runs off the UI thread**, always.

### Why "transpile to SwiftUI/Compose"?

SwiftUI and Compose are themselves declarative + reactive. By generating *their*
source instead of building our own renderer (like Slint) or our own reconciler
(like React Native's Fabric / Lynx), we **delegate diffing & reconciliation to
the platform**. Apple and Google already built the best reconcilers in the
world; nui just feeds them state and routes events back.

The genuinely hard part that is *ours*: mapping one `.nui` description onto two
different declarative dialects (SwiftUI vs Compose). That's the research
frontier of this project — not the parser.

## Architecture: three layers, one contract

```
.nui DSL  ──compiles to──►  native UI (SwiftUI / Compose)   [the projection]
                                     │
                              the bridge (contract)
                              events ↓     state ↑
                                     │
                            logic (any language)             [source of truth]
```

The **contract** is a language-agnostic protocol of typed *State* + *Events*.
Anything that can speak it can be a logic backend.

## Roadmap

- **Phase 0 — Prove the loop. [done]** A real SwiftUI counter on the iOS
  simulator, driven by a Python process over a local socket. No DSL, no codegen.
  Validated that "platform-reconciled UI + off-thread logic over a thin
  contract" feels good. See [`docs/PHASE0.md`](./docs/PHASE0.md).
- **Phase 1 — `.nui` DSL + SwiftUI codegen. [done]** The `nuic` compiler
  (Rust) parses [`examples/counter.nui`](./examples/counter.nui) and emits
  SwiftUI that runs on the iOS simulator. Design designed backward from output
  in [`docs/PHASE1-mapping.md`](./docs/PHASE1-mapping.md).
- **Phase 2 — Compose codegen. [done]** Same `.nui`, second emitter, runs as
  native Jetpack Compose on the Android emulator. One DSL → SwiftUI + Compose,
  both driven by the same logic.
- **Phase 3 — Embed the logic in-process. [done]** CPython 3.13 embedded on
  **iOS** (simulator + real device via BeeWare) and **Android** (emulator via
  Chaquopy). Same `counter.py`, same `Bridge` API, no socket. See
  [`docs/PHASE3.md`](./docs/PHASE3.md).
- **Phase 4** — Add a second logic language (Rust) to prove the contract is
  truly language-agnostic.
- **Vocabulary** — growing; see [`docs/VOCABULARY.md`](./docs/VOCABULARY.md).
  Current widgets include Scroll, Card, Divider, If/else, Icon, Progress, Switch,
  Slider, TextField, plus Phase 1 layout/content primitives.

## Layout

```
examples/counter.nui   # the UI, written once in the nui DSL
compiler/              # `nuic` — the Rust compiler (lexer, parser, emitters)
logic/counter.py       # the logic core: owns state, transport-free (embeddable)
logic/server.py        # dev transport: a TCP socket host around the core
ios/                   # SwiftUI app; embeds CPython in-process (Phase 3)
android/               # Compose app; embeds Python via Chaquopy (Phase 3)
PROTOCOL.md            # the wire contract
run.sh                 # iOS: build + boot simulator + launch helper
```

### Regenerate the views from the DSL

```bash
cargo build --release --manifest-path compiler/Cargo.toml
./compiler/target/release/nuic examples/counter.nui --target swift   -o ios/Sources/Generated/CounterView.swift
./compiler/target/release/nuic examples/counter.nui --target compose -o android/app/src/main/java/dev/nui/generated/CounterView.kt
```

See [`PROTOCOL.md`](./PROTOCOL.md) for the wire format and
[`docs/PHASE0.md`](./docs/PHASE0.md) for how to run it.
