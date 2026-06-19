//! The nui AST. Grows with the vocabulary — see docs/VOCABULARY.md.

#[derive(Debug, Clone)]
pub enum Value {
    /// String literal contents (quotes stripped). May contain `\(...)` interpolation.
    Str(String),
    /// Numeric literal kept as raw text (e.g. "28", "96").
    Num(String),
    /// Bare identifier used as a value (e.g. `bold`, `center`, `secondary`, `red`).
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct Arg {
    /// `None` for positional args like `Text("hi")` or `.color(secondary)`.
    pub name: Option<String>,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct Modifier {
    pub name: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub args: Vec<Arg>,
    pub modifiers: Vec<Modifier>,
    /// Set by `-> eventName` (e.g. a button's action).
    pub event: Option<String>,
    pub children: Vec<Element>,
    /// Populated for `If` when followed by `else { ... }`.
    pub else_children: Vec<Element>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub default: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub state: Vec<Field>,
    pub events: Vec<String>,
    pub root: Element,
}
