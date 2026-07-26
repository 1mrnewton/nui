# The nui language — minimal grammar (v0)

This is the smallest language that can express the counter app. It is
deliberately tiny: nui describes *what the UI is*, never *what it computes*.
Anything smarter than a binding belongs in the logic layer.

Record types (`type Person { ... }`) let state and logic functions carry
structured values; see `examples/profile.nui`. List types (`[Todo]`) and
`for ... in` render dynamic content; see `examples/todos.nui`.

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
- **Keywords**: `component`, `type`, `state`, `logic`, `fn`, `if`, `else`,
  `for`, `in`, `true`, `false`. (`event` is reserved and produces a
  migration error. `style` and `on_click` are ordinary identifiers with
  special meaning as view keys.)
- **Numbers**: `123`, `-4`, `2.5`. A `.` only starts a fraction when a digit
  follows; `->` is the arrow.
- **Strings**: double-quoted. Escapes: `\"`, `\\`, `\n`, `\t`, `\{`, `\}`.
  `{name}` inside a string is **state interpolation** and is resolved at
  compile time into text segments — runtimes never parse strings. A dotted
  path reaches into a record: `{person.name}`.

## Grammar (EBNF)

```ebnf
document    = { typeDecl } component ;
typeDecl    = "type" IDENT "{" { field } "}" ;
field       = IDENT ":" primType ;
component   = "component" IDENT "{" { declaration } node "}" ;
declaration = stateDecl | logicBlock ;
stateDecl   = "state" IDENT ":" type "=" ( literal | recordLit | listLit ) ;
logicBlock  = "logic" "{" { fnDecl } "}" ;
fnDecl      = "fn" IDENT "(" [ paramList ] ")" "->" type ;
paramList   = param { "," param } ;
param       = IDENT ":" type ;
primType    = "Int" | "Float" | "Bool" | "String" ;
type        = primType
            | IDENT               (* a declared record type *)
            | "[" ( primType | IDENT ) "]" ;  (* a list — no nesting *)

node        = IDENT [ "{" { entry } "}" ] ;
entry       = styleBlock | actionBlock | property | child ;
child       = node | ifBranch | forLoop ;
ifBranch    = "if" path "{" { child } "}" [ "else" "{" { child } "}" ] ;
forLoop     = "for" IDENT "in" path "{" { child } "}" ;
property    = IDENT ":" expr ;
styleBlock  = "style" ":" "{" { styleProp } "}" ;
styleProp   = IDENT ":" expr ;
actionBlock = "on_click" ":" "{" action "}" ;
action      = IDENT "=" call ;
call        = IDENT "(" [ expr { "," expr } ] ")" ;
recordLit   = IDENT "(" fieldInit { "," fieldInit } ")" ;
fieldInit   = IDENT ":" literal ;
listLit     = "[" [ ( literal | recordLit ) { "," ( literal | recordLit ) } ] "]" ;

path        = IDENT { "." IDENT } ;
expr        = literal | path ;
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
- A bare `IDENT` (or dotted path) in value position is resolved during
  lowering: a state reference (`text: name`, `text: person.name`) or an
  enum-like value (`font: title`).
- `type` declares a record: named, typed fields, primitives only for now.
  A record state initializes with a literal naming every field —
  `state person: Person = Person(name: "Ada", bio: "...")` (named
  arguments distinguish a record literal from a logic call). Records pass
  whole through logic functions; text reaches into them with dotted paths.
- `on_click:` blocks hold exactly one action (for now). Call arguments are
  state references, record-field paths, or literals — never nested calls.
  Actions are fully type-checked: the function must be declared, argument
  types must match the parameters, and the return type must match the
  assigned state. Actions assign to a whole state, never to a field —
  return a new record from the logic function instead.
- `if` appears in child position and branches hold child views only. The
  condition is always a declared `Bool` state (or a `Bool` record field) —
  no comparisons or boolean expressions in the UI; anything smarter than a
  flag is computed in the logic layer (see `examples/toggle.nui`).
  `else if` is not supported yet; nest an `if` inside `else { ... }`.
- `for item in items` also appears in child position and renders one
  subtree per element of a list state. The loop variable is a scoped local:
  interpolate it (`"{todo.title}"`) anywhere in the body. V1 guardrails,
  all compile errors: no `if`, `for`, or `TextField` inside a `for` body,
  and loop variables can't be passed to logic functions (whole-list
  operations only — per-row actions need identity and come later).

## Views (v0)

| View | Properties | Children |
| --- | --- | --- |
| `Text` | `text:` string (may interpolate) or state name — required | no |
| `Button` | `label:` string — required; `on_click: { state = fn(args) }` — required | no |
| `TextField` | `bind:` a `String` state — required; `placeholder:` string | no |
| `Image` | `source:` plain string — required | no |
| `VStack` / `HStack` | `spacing:` number | yes |
| `List` | none | yes (typically a `for`) |
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
- `if` is structural: exactly one branch is in the layout at a time,
  driven by a `Bool` state. SwiftUI renders it as a native `if` in the
  view builder; UIKit renders each branch as a container whose visibility
  tracks the state.
- Record types cross the FFI boundary by value. The Rust side gets a
  UniFFI `Record` struct prefixed with the component name
  (`ProfilePerson`) so the UniFFI-generated Swift never collides with the
  UI-side struct (`Person`); the generated bridge converts field-by-field.
- Lists cross the FFI boundary whole (`[Todo]` ↔ `Vec<TodoListTodo>`),
  converted element-wise in the bridge. `for` renders them: SwiftUI gets a
  native `ForEach`; UIKit gets a container stack whose rows are rebuilt
  from a generated `makeRow(_:)` on every state change — deliberate
  wholesale rebuild, still no diffing engine.
- Everything is checked at compile time: state references, record fields,
  function references, property names, argument types, and return types.
  A `.nui` file that compiles cannot make an ill-typed call at runtime.

## Reserved for later (not implemented, by design)

- Record fields of record or list type (nesting), and nested lists
- Per-row actions (toggle/delete one item) — needs element identity across
  the FFI, not just an index
- `else if` chains (nest an `if` inside `else { ... }` for now)
- Component composition (`component Row { ... }` used inside another view)
- More action keys (`on_submit:` for `TextField`, `on_appear:`) and
  multiple actions per block
- Named styles (`style card { ... }` declared once, applied as
  `style: card`) — reuse and theming without CSS's cascade or selectors:
  those don't map to SwiftUI/Compose.
