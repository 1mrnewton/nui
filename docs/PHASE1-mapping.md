# Phase 1 — designing the DSL backward from its output

We are **not** writing a parser yet. First we answer: *"if `counter.nui` existed,
what is the exact SwiftUI and Compose it must generate?"* The DSL falls out of
the differences between those two targets.

Source under design: [`examples/counter.nui`](../examples/counter.nui).

---

## The `.nui` source

```
component Counter {
    state { count: Int = 0 }
    event increment
    event decrement
    event reset

    Column(spacing: 28, padding: 32, align: center) {
        Spacer()
        Text("\(count)").font(size: 96, weight: bold)
        Text("this number lives in your logic")
            .font(size: 16).color(secondary)
        Spacer()
        Row(spacing: 20) {
            IconButton(icon: minus, tint: red)  -> decrement
            IconButton(icon: plus,  tint: green) -> increment
        }
        Button("Reset") -> reset .style(bordered)
    }
}
```

---

## Target 1 — generated SwiftUI

`Bridge` (from Phase 0) stays as the generic runtime. The compiler generates a
**typed state struct**, a **view-model** (the reactivity adapter), and the
**view**.

```swift
// GENERATED from counter.nui — do not edit.
import SwiftUI

struct CounterState: Decodable {
    var count: Int = 0
}

@MainActor
final class CounterModel: ObservableObject {
    @Published private(set) var state = CounterState()
    private let bridge: Bridge

    init(bridge: Bridge) {
        self.bridge = bridge
        bridge.onState { [weak self] (new: CounterState) in self?.state = new }
    }

    func increment() { bridge.send(event: "increment") }
    func decrement() { bridge.send(event: "decrement") }
    func reset()     { bridge.send(event: "reset") }
}

struct CounterView: View {
    @StateObject private var model: CounterModel
    init(bridge: Bridge) { _model = StateObject(wrappedValue: CounterModel(bridge: bridge)) }

    var body: some View {
        VStack(spacing: 28) {                                   // Column(spacing:)
            Spacer()
            Text("\(model.state.count)")                        // "\(count)"
                .font(.system(size: 96, weight: .bold))
            Text("this number lives in your logic")
                .font(.system(size: 16))
                .foregroundStyle(.secondary)                    // color(secondary)
            Spacer()
            HStack(spacing: 20) {                               // Row(spacing:)
                Button(action: model.decrement) {              // -> decrement
                    Image(systemName: "minus")                 // icon: minus
                }.tint(.red)
                Button(action: model.increment) {
                    Image(systemName: "plus")
                }.tint(.green)
            }
            Button("Reset", action: model.reset)               // -> reset
                .buttonStyle(.bordered)                         // .style(bordered)
        }
        .padding(32)                                            // padding: 32
        .frame(maxWidth: .infinity)                            // align: center
    }
}
```

---

## Target 2 — generated Jetpack Compose

```kotlin
// GENERATED from counter.nui — do not edit.
package dev.nui.generated

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class CounterState(val count: Int = 0)

class CounterModel(private val bridge: Bridge) {
    var state by mutableStateOf(CounterState())               // reactivity adapter
        private set

    init { bridge.onState { new -> state = new } }

    fun increment() = bridge.send("increment")
    fun decrement() = bridge.send("decrement")
    fun reset()     = bridge.send("reset")
}

@Composable
fun CounterView(bridge: Bridge) {
    val model = remember { CounterModel(bridge) }
    Column(
        modifier = Modifier.padding(32.dp).fillMaxWidth(),     // padding:32 + align:center
        verticalArrangement = Arrangement.spacedBy(28.dp),     // spacing: 28
        horizontalAlignment = Alignment.CenterHorizontally     // align: center
    ) {
        Spacer(Modifier.weight(1f))                            // Spacer() (greedy!)
        Text(
            text = "${model.state.count}",                     // "\(count)" -> "$count"
            fontSize = 96.sp,                                  // font(size:weight:)
            fontWeight = FontWeight.Bold
        )
        Text(
            text = "this number lives in your logic",
            fontSize = 16.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant // color(secondary)
        )
        Spacer(Modifier.weight(1f))
        Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
            FilledIconButton(                                  // IconButton
                onClick = model::decrement,                    // -> decrement
                colors = IconButtonDefaults.filledIconButtonColors(containerColor = Color.Red)
            ) { Icon(Icons.Filled.Remove, contentDescription = "minus") }  // icon: minus
            FilledIconButton(
                onClick = model::increment,
                colors = IconButtonDefaults.filledIconButtonColors(containerColor = Color.Green)
            ) { Icon(Icons.Filled.Add, contentDescription = "plus") }
        }
        OutlinedButton(onClick = model::reset) { Text("Reset") } // Button .style(bordered)
    }
}
```

---

## The mapping table (this is the real work)

| nui | SwiftUI | Compose | the mismatch to abstract |
|---|---|---|---|
| `Column` | `VStack` | `Column` | — |
| `Row` | `HStack` | `Row` | — |
| `spacing: 28` | `VStack(spacing: 28)` | `Arrangement.spacedBy(28.dp)` | a param vs an *Arrangement* object |
| `padding: 32` | `.padding(32)` | `Modifier.padding(32.dp)` | trailing modifier vs Modifier arg |
| `align: center` | `.frame(maxWidth:.infinity)` | `horizontalAlignment + fillMaxWidth` | cross-axis alignment models differ |
| `Spacer()` | `Spacer()` (greedy) | `Spacer(Modifier.weight(1f))` | **SwiftUI Spacer is greedy; Compose needs `weight`** |
| `Text("\(count)")` | `Text("\(...)")` | `Text("$...")` | interpolation syntax differs |
| `font(size:96, weight:bold)` | `.font(.system(size:96, weight:.bold))` | `fontSize=96.sp, fontWeight=Bold` | one `Font` vs two params; **pt vs sp** |
| `color(secondary)` | `.foregroundStyle(.secondary)` | `colorScheme.onSurfaceVariant` | **no shared semantic palette** |
| `IconButton(icon: minus)` | `Button{ Image(systemName:"minus") }` | `FilledIconButton{ Icon(Icons.Filled.Remove) }` | **SF Symbols ≠ Material Icons** |
| `tint: red` | `.tint(.red)` | `colors = ...(containerColor=Color.Red)` | tinting APIs differ wildly |
| `Button("x") .style(bordered)` | `Button("x"){} .buttonStyle(.bordered)` | `OutlinedButton(...){ Text("x") }` | "style" = modifier vs *different widget* |
| `-> increment` | `action: model.increment` | `onClick = model::increment` | trailing closure vs `onClick` param |
| `count` (read) | `model.state.count` | `model.state.count` | identical at the use site |
| reactivity | `@Published`/`ObservableObject` | `mutableStateOf`/`by` | **totally different — but fully hidden in generated boilerplate** |

---

## What this tells us about the DSL

1. **The scary part doesn't leak.** The two reactive systems (`@Published` vs
   `mutableStateOf`) are completely different, yet they live *only* in generated
   boilerplate the author never sees. This is the strongest validation yet that
   "transpile to SwiftUI/Compose" is the right strategy: divergence is absorbed
   by codegen, not the DSL.

2. **The DSL must be unitless and semantic.** Express *intent* — `font(size: 96)`,
   `color(secondary)` — and let codegen resolve pt-vs-`sp`, `.secondary` vs
   `onSurfaceVariant`. The DSL never speaks platform units or platform palettes.

3. **Three things need their own nui namespaces** (the unavoidable "two dialects"
   tax, isolated to small lookup tables):
   - **Icons** — a curated `nui` icon set mapping to SF Symbols *and* Material
     Icons. (`minus` → `"minus"` / `Icons.Filled.Remove`.) No 1:1 set exists.
   - **Colors** — a semantic palette (`primary`, `secondary`, `danger`, …)
     mapped per design system.
   - **Component styles** — `style(bordered)` chooses a *modifier* on iOS but a
     *different widget* on Android (`OutlinedButton`). The DSL names the intent;
     codegen picks the construct.

4. **`Spacer` and layout need normalized semantics.** A `nui` `Spacer()` means
   "greedy flexible space" → bare `Spacer()` on iOS, `Spacer(Modifier.weight(1f))`
   on Android. Layout intent must be defined by nui, not inherited from either
   platform.

5. **Keep v1 vocabulary tiny and curated.** `Column, Row, Text, Button,
   IconButton, Spacer` + a handful of modifiers (`font, color, padding, spacing,
   align, tint, style`). A small surface keeps two-target codegen tractable;
   grow it deliberately.

---

## Proposed next step (Phase 1 build)

A three-stage compiler, smallest viable version:

```
counter.nui ──[parser]──► AST ──┬──[SwiftUI emitter]──► CounterView.swift
                                └──[Compose emitter]──► CounterView.kt
```

- **AST**: a handful of node types (`Component`, `StateDecl`, `EventDecl`,
  `Element{ name, args, modifiers, children, event? }`).
- **Two emitters**: pure functions `AST -> String`, each owning its platform's
  quirks + the icon/color/style lookup tables.
- **Validation target**: regenerate the Phase 0 SwiftUI from `counter.nui` and
  diff against the hand-written `ContentView.swift`. When they match, the
  SwiftUI emitter is correct. Then bring up the Compose side on an emulator.

Open question to decide before building: **what to write the compiler in?**
(Rust — fits the "logic compiled to native" story and is fast; or TypeScript —
fastest to iterate. Either works.)
