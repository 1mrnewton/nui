# nui syntax explorations

> **Decision: Variant C, with block-valued actions.** Named properties and
> implicit children from C, plus the action written as a block —
> `on_click: { count = increment(count) }` — so a tap handler reads like a
> (future multi-statement) block, not a value. Style blocks use the colon
> form `style: { ... }` so every keyed entry looks the same. This is
> implemented; docs/GRAMMAR.md is the reference. The rest of this file is
> kept as the record of the alternatives considered.

Five candidate syntax families for the nui view layer. Variant A is what
the compiler accepted before the decision; the others were proposals.
State and `logic` declarations are identical in every variant
(they're not what's being decided), so each example only varies the view
tree.

Every variant expresses the same two components:

- **Counter** — the minimal case.
- **Profile** — the stress test: image, text field binding, nested stacks,
  a list, and two actions. This is where verbosity and nesting depth
  actually show.

Shared declarations for both (identical in all variants):

```
component Counter {
    state count: Int = 0

    logic {
        fn increment(count: Int) -> Int
        fn decrement(count: Int) -> Int
    }
    // ... view ...
}

component Profile {
    state name: String = ""
    state followers: Int = 0

    logic {
        fn follow(count: Int) -> Int
        fn shout(name: String) -> String
    }
    // ... view ...
}
```

## The four knobs being decided

1. **Properties** — positional in parens (`Button("+")`) vs named
   key–value (`label: "+"`).
2. **Children** — implicit nesting in the same braces vs an explicit
   `children: [...]` / `child:` property.
3. **Style** — chained modifiers vs an inline `style { }` group vs named
   reusable styles.
4. **Actions** — `action:` vs `on_click:` vs a trailing block.

---

## Variant A — SwiftUI-style (current)

Positional args, implicit children, chained dot-modifiers.

```
// Counter
VStack(spacing: 16) {
    Text("Count: {count}")
        .font(title)

    HStack(spacing: 12) {
        Button("-", action: count = decrement(count))
        Button("+", action: count = increment(count))
    }
}
.padding(24)
```

```
// Profile
VStack(spacing: 16) {
    Image("avatar")
        .foregroundColor(gray)

    Text("Hi, {name}!")
        .font(title)

    TextField(binding: name, placeholder: "Your name")

    HStack(spacing: 12) {
        Button("Follow", action: followers = follow(followers))
        Button("Shout", action: name = shout(name))
    }

    List {
        Text("Followers: {followers}")
    }
}
.padding(24)
```

- Tersest structure; nesting mirrors the visual hierarchy exactly.
- Positional args are opaque (`Image("avatar")` — source? label?), and
  modifiers dangling after `}` are easy to misattach when editing.
- Style is scattered: layout props in parens, visual props in a chain.

---

## Variant B — everything is a property (Flutter-style, your sketch)

Every view is `Name { key: value ... }`. Children are an explicit
`children: [ ... ]` (or `child:` for exactly one). Style is a nested block.

```
// Counter
VStack {
    spacing: 16
    style: {
        padding: 24
    }
    children: [
        Text {
            text: "Count: {count}"
            style: {
                font: title
            }
        }
        HStack {
            spacing: 12
            children: [
                Button {
                    label: "-"
                    on_click: count = decrement(count)
                }
                Button {
                    label: "+"
                    on_click: count = increment(count)
                }
            ]
        }
    ]
}
```

```
// Profile
VStack {
    spacing: 16
    style: { padding: 24 }
    children: [
        Image {
            source: "avatar"
            style: { color: gray }
        }
        Text {
            text: "Hi, {name}!"
            style: { font: title }
        }
        TextField {
            bind: name
            placeholder: "Your name"
        }
        HStack {
            spacing: 12
            children: [
                Button { label: "Follow", on_click: followers = follow(followers) }
                Button { label: "Shout",  on_click: name = shout(name) }
            ]
        }
        List {
            children: [
                Text { text: "Followers: {followers}" }
            ]
        }
    ]
}
```

- Maximally explicit and uniform — every value has a name; great for
  tooling, autocomplete, and reading unfamiliar views.
- `child:` vs `children:` makes arity part of the contract (Flutter-style).
- Verbose: the counter is ~2.3× the lines of Variant A, and real trees get
  deep fast — `children: [` adds a level of indentation at every layer.

---

## Variant C — properties + nested children (QML-style hybrid)

Properties are `key: value`; anything starting with a capitalized view name
is a child — no `children:` wrapper. Style is a grouped inline block.
Single-line form allowed with commas.

```
// Counter
VStack {
    spacing: 16
    style { padding: 24 }

    Text {
        text: "Count: {count}"
        style { font: title }
    }

    HStack {
        spacing: 12

        Button { label: "-", on_click: count = decrement(count) }
        Button { label: "+", on_click: count = increment(count) }
    }
}
```

```
// Profile
VStack {
    spacing: 16
    style { padding: 24 }

    Image {
        source: "avatar"
        style { color: gray }
    }

    Text {
        text: "Hi, {name}!"
        style { font: title }
    }

    TextField {
        bind: name
        placeholder: "Your name"
    }

    HStack {
        spacing: 12

        Button { label: "Follow", on_click: followers = follow(followers) }
        Button { label: "Shout",  on_click: name = shout(name) }
    }

    List {
        Text { text: "Followers: {followers}" }
    }
}
```

- Named properties like B, but one indentation level per visual level —
  the tree shape stays honest without `children:` ceremony.
- Style is grouped in one place per view instead of scattered chains.
- Rule to internalize: lowercase `key:` = property, Capitalized `Name {` =
  child. Unambiguous to parse, but mixing both in one block is a look you
  either like or don't.

---

## Variant D — named styles (CSS-inspired, no cascade)

Structure stays skinny (any of A/B/C works as the base — shown on A);
visuals move into named `style` bundles, referenced with a leading dot.
No selectors, no cascade — a style is just a named bag of properties
applied explicitly.

```
// Counter
style container {
    spacing: 16
    padding: 24
}

style big {
    font: title
}

VStack(.container) {
    Text(.big, "Count: {count}")

    HStack(spacing: 12) {
        Button("-", on_click: count = decrement(count))
        Button("+", on_click: count = increment(count))
    }
}
```

```
// Profile
style card {
    spacing: 16
    padding: 24
}

style headline {
    font: title
}

style muted {
    color: gray
}

VStack(.card) {
    Image(.muted, "avatar")
    Text(.headline, "Hi, {name}!")
    TextField(bind: name, placeholder: "Your name")

    HStack(spacing: 12) {
        Button("Follow", on_click: followers = follow(followers))
        Button("Shout",  on_click: name = shout(name))
    }

    List {
        Text("Followers: {followers}")
    }
}
```

- Styles become reusable and theme-able; structure reads like an outline.
- The CSS feel you wanted, without the parts that don't map to native
  (cascade, selectors, specificity).
- Indirection: you jump to see what a view looks like, and one-off styles
  force you to invent names. Works best *combined* with inline styles for
  one-offs (this knob is orthogonal — it can be added to any variant later).

---

## Variant E — compact closures (Compose-style)

Positional args stay; a Button's trailing block *is* its action; `.style()`
is one grouped modifier call.

```
// Counter
VStack(spacing: 16) {
    Text("Count: {count}").style(font: title)

    HStack(spacing: 12) {
        Button("-") { count = decrement(count) }
        Button("+") { count = increment(count) }
    }
}.style(padding: 24)
```

```
// Profile
VStack(spacing: 16) {
    Image("avatar").style(color: gray)
    Text("Hi, {name}!").style(font: title)
    TextField(bind: name, placeholder: "Your name")

    HStack(spacing: 12) {
        Button("Follow") { followers = follow(followers) }
        Button("Shout")  { name = shout(name) }
    }

    List {
        Text("Followers: {followers}")
    }
}.style(padding: 24)
```

- The tersest of all; actions read like code, which they conceptually are.
- Inconsistency baked in: a trailing block means *action* on a Button but
  *children* on a stack. Fine once learned, surprising at first.
- Same opaque-positional-args problem as A.

---

## Comparison

| | A (current) | B (properties) | C (QML hybrid) | D (named styles) | E (compact) |
|---|---|---|---|---|---|
| Counter view, lines | 11 | 26 | 15 | 15 | 9 |
| Every value named | no | **yes** | **yes** | partly | no |
| Nesting depth (Profile) | 3 | 6 | 3 | 3 | 3 |
| Style grouped per view | no (chain) | yes | yes | yes (named) | yes (one call) |
| Style reuse across views | no | no | no | **yes** | no |
| Tooling/autocomplete friendliness | ok | **best** | good | good | ok |
| Ambiguity risk when editing | dangling modifiers | none | prop-vs-child rule | style-name lookup | block meaning varies |

## Orthogonal knobs (pick separately, any variant)

- **Action key name**: `on_click:` (your sketch) vs `on_tap:` (mobile-true)
  vs `action:` (current). Pure taste.
- **Case convention**: `on_click` / `text_field` (snake) vs `onClick` /
  `TextField` (camel). Snake keys + Camel view names (QML does this) reads
  well in B/C.
- **Separators**: newline-only vs optional commas/semicolons for
  single-line forms (`Button { label: "-", on_click: ... }`).
- **The text property**: positional `Text("hi")` vs `text: "hi"` vs
  `value: "hi"`.
- **Named styles (D)** can be bolted onto any base variant later without
  breaking existing files — inline for one-offs, named for reuse.

## My read

C is the sweet spot for what you described: your property-block instinct
from B, without paying a `children: [...]` indentation tax at every level
of a real screen (compare the Profile examples — B is six levels deep, C is
three). Add D's named styles later once real apps make styles repeat.
But A/E optimize for terseness if you find B/C too ceremonial — this is
genuinely taste, which is why all five exist in this doc.
