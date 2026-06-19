# nui vocabulary (current)

The DSL is intentionally small and semantic. Platform emitters resolve intent into
SwiftUI / Compose idioms. See [`examples/counter.nui`](../examples/counter.nui)
for a living sample that exercises most widgets.

## Component structure

```nui
component Name {
    state { field: Type = default }
    event event_name

    // single root element (often Scroll or Column)
    Column { ... }
}
```

## State types

| Type | Notes |
|------|-------|
| `Int` | |
| `Bool` | defaults: `true`, `false` |
| `String` | string literals in defaults |
| `Double` | |

## Layout

| Widget | Args | Notes |
|--------|------|-------|
| `Column` | `spacing`, `padding`, `align: center` | → `VStack` / Compose `Column` |
| `Row` | `spacing`, `padding`, `align: center` | → `HStack` / Compose `Row` |
| `Scroll` | — | scrollable container |
| `Card` | — | grouped content with surface styling |
| `Spacer` | — | flexible space |
| `Divider` | — | horizontal rule |

## Content

| Widget | Args | Modifiers |
|--------|------|-----------|
| `Text("…\(field)…")` | string with `\(...)` interpolation | `.font(size:, weight:)`, `.color(secondary)`, `.padding(n)` |
| `Icon` | `icon:`, `tint:`, `size:` | decorative icon (not tappable) |
| `Button("Label")` | label string | `.style(bordered)` → `-> event` |
| `IconButton` | `icon: plus\|minus\|…`, `tint: red\|green\|…` | `-> event` |
| `Progress` | `bind: field`, `max:` | read-only bar; clamps when value exceeds `max` |

## Input (two-way via events)

Bind widgets read state and emit events **with payload** `{"value": …}`:

| Widget | Args | Event |
|--------|------|-------|
| `Switch` | `label:`, `bind: field` | `-> set_field` |
| `Slider` | `label:`, `bind:`, `min:`, `max:` | `-> set_field` |
| `TextField` | `label:`, `bind: field` | `-> set_field` |

Logic handlers receive the payload dict and return updated state (MVU — UI never
mutates state directly).

## Conditional

```nui
If(show_label) {
    Text("when true")
} else {
    Text("when false")
}
```

Positional arg is the `Bool` state field name. The `else` branch is optional.

## Events

- Simple: `event increment` → `bridge.send("increment")`
- With payload: declared as `event set_title`; generated model method
  `set_title(value)` sends `{"value": value}`

## Colors & semantics

Unitless numbers in layout (`spacing: 28`) — emitters attach `.dp` / points.

Semantic colors: `secondary`, `primary`, `red`, `green`, …

Icons: `plus`, `minus`, `check`, `close`, `refresh` (mapped per platform).
