# The nui language — minimal grammar (v0)

This is the smallest language that can express the counter app. It is
deliberately tiny: nui describes *what the UI is*, never *what it computes*.
Anything smarter than a binding belongs in the logic layer.

## Example

```
component Counter {
    state count: Int = 0

    logic {
        fn increment(count: Int) -> Int
        fn decrement(count: Int) -> Int
    }

    VStack {
        spacing: 16
        style: { padding: 24 }

        Text {
            text: "Count: {count}"
            style: { font: title }
        }

        HStack {
            spacing: 12

            Button {
                label: "-"
                on_click: { count = decrement(count) }
            }
            Button {
                label: "+"
                on_click: { count = increment(count) }
            }
        }
    }
}
```

## Lexical rules

- **Whitespace** is insignificant. Newlines are just whitespace.
- **Comments**: `//` to end of line.
- **Identifiers**: `[A-Za-z_][A-Za-z0-9_]*`.
- **Keywords**: `component`, `state`, `logic`, `fn`, `true`, `false`.
  (`event` is reserved and produces a migration error. `style` and
  `on_click` are ordinary identifiers with special meaning as view keys.)
- **Numbers**: `123`, `-4`, `2.5`. A `.` only starts a fraction when a digit
  follows; `->` is the arrow.
- **Strings**: double-quoted. Escapes: `\"`, `\\`, `\n`, `\t`, `\{`, `\}`.
  `{name}` inside a string is **state interpolation** and is resolved at
  compile time into text segments — runtimes never parse strings.

## Grammar (EBNF)

```ebnf
document    = component ;
component   = "component" IDENT "{" { declaration } node "}" ;
declaration = stateDecl | logicBlock ;
stateDecl   = "state" IDENT ":" type "=" literal ;
logicBlock  = "logic" "{" { fnDecl } "}" ;
fnDecl      = "fn" IDENT "(" [ paramList ] ")" "->" type ;
paramList   = param { "," param } ;
param       = IDENT ":" type ;
type        = "Int" | "Float" | "Bool" | "String" ;

node        = IDENT [ "{" { entry } "}" ] ;
entry       = styleBlock | actionBlock | property | node ;
property    = IDENT ":" expr ;
styleBlock  = "style" ":" "{" { styleProp } "}" ;
styleProp   = IDENT ":" expr ;
actionBlock = "on_click" ":" "{" action "}" ;
action      = IDENT "=" call ;
call        = IDENT "(" [ expr { "," expr } ] ")" ;

expr        = literal | IDENT ;
literal     = INT | FLOAT | STRING | "true" | "false" ;
```

Notes:

- A component has exactly one root view, after all declarations.
- Everything in a view body is keyed by how it starts: `name: value` is a
  property, `style:` opens a style block, `on_click:` opens an action
  block, and a bare view name starts a child. Capitalized view names vs.
  lowercase keys keep the two visually distinct. Commas between entries
  are optional — use them for one-liners
  (`Button { label: "+", on_click: { ... } }`).
- A bare `IDENT` in value position is resolved during lowering: a state
  reference (`text: name`) or an enum-like value (`font: title`).
- `on_click:` blocks hold exactly one action (for now). Call arguments are
  state references or literals — never nested calls. Actions are fully
  type-checked: the function must be declared, argument types must match
  the parameters, and the return type must match the assigned state.

## Views (v0)

| View | Properties | Children |
| --- | --- | --- |
| `Text` | `text:` string (may interpolate) or state name — required | no |
| `Button` | `label:` string — required; `on_click: { state = fn(args) }` — required | no |
| `TextField` | `bind:` a `String` state — required; `placeholder:` string | no |
| `Image` | `source:` plain string — required | no |
| `VStack` / `HStack` | `spacing:` number | yes |
| `List` | none | yes |
| `Spacer` | none | no |

Every view except `Spacer` also accepts one `style:` block.

## Style properties (v0)

| Property | Value |
| --- | --- |
| `padding:` | number |
| `font:` | `largeTitle`, `title`, `headline`, `body`, `caption` |
| `color:` | `primary`, `secondary`, `red`, `green`, `blue`, `orange`, `yellow`, `purple`, `gray` |

## Semantics: nui owns the state, the backend owns the logic

- `state` declares the schema and initial values. The generated UI store
  owns the canonical state and renders from it.
- `logic` declares the typed interface the backend implements — pure
  functions, no globals. From it the compiler generates the Swift protocol,
  the Rust signature checks, and the FFI bridge; the only handwritten code
  is the function bodies in the logic crate.
- Actions are declarative routing, not code:
  `on_click: { count = increment(count) }` means "on tap, call `increment`
  with the current `count`, assign the result back to `count`". The UI
  never computes values — computation lives exclusively in the logic
  layer, so both platforms behave identically.
- `TextField` writes its text into the bound `String` state directly: local,
  UI-owned mutation with no computation involved.
- Everything is checked at compile time: state references, function
  references, property names, argument types, and return types. A `.nui`
  file that compiles cannot make an ill-typed call at runtime.

## Reserved for later (not implemented, by design)

- Record types (`type Todo { title: String, done: Bool }`) usable as state
  and in logic signatures — needed for `data = fetchData()`
- `if` / `else` — conditional subtrees driven by `Bool` state
- `for x in items` — dynamic `List` content over collection state
- Component composition (`component Row { ... }` used inside another view)
- More action keys (`on_submit:` for `TextField`, `on_appear:`) and
  multiple actions per block
- Named styles (`style card { ... }` declared once, applied as
  `style: card`) — reuse and theming without CSS's cascade or selectors:
  those don't map to SwiftUI/Compose.
