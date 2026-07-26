//! Syntax tree produced by the parser, before checking and lowering.
//!
//! Deliberately loose: node names, modifier names, and argument shapes are
//! plain strings here. The lowering pass (`lower.rs`) is what knows which
//! views exist and validates references, so parse errors and semantic errors
//! stay separate.

use crate::lexer::StrSegment;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Document {
    /// Top-level `type Name { field: Type ... }` declarations.
    pub types: Vec<TypeDecl>,
    pub component: Component,
}

/// A record type declaration. Field shapes reuse [`Param`]; lowering
/// checks that field types are primitives.
#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub fields: Vec<Param>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub states: Vec<StateDecl>,
    pub functions: Vec<FnDecl>,
    pub root: NodeExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    /// Type name as written (`Int`, `String`, ...); validated during lowering.
    pub ty: String,
    pub initial: Expr,
    pub span: Span,
}

/// A declared logic function: `fn increment(count: Int) -> Int`.
/// Implemented by the backend (Rust); called from view actions.
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    /// Return type name as written; validated during lowering.
    pub returns: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    /// Type name as written; validated during lowering.
    pub ty: String,
    pub span: Span,
}

/// A view: `Name { prop: value  style: { ... }  Child { ... } }`.
///
/// Everything in a view body is keyed: `prop: value` entries land in
/// `props` (an `on_click: { ... }` action block is a prop whose value is
/// an [`ExprKind::Assign`]), `style: { ... }` entries land in `styles`,
/// and capitalized `Name { ... }` or `if ... { ... }` entries land in
/// `children`.
#[derive(Debug, Clone)]
pub struct NodeExpr {
    pub name: String,
    pub props: Vec<Prop>,
    pub styles: Vec<Prop>,
    pub children: Vec<ChildExpr>,
    pub span: Span,
}

/// One entry in child position: a view, or an `if` over a Bool state.
#[derive(Debug, Clone)]
pub enum ChildExpr {
    Node(NodeExpr),
    If(IfExpr),
}

impl ChildExpr {
    pub fn span(&self) -> Span {
        match self {
            ChildExpr::Node(node) => node.span,
            ChildExpr::If(if_expr) => if_expr.span,
        }
    }
}

/// `if showHint { ... } else { ... }` — the condition is a Bool state
/// name (validated during lowering); the else branch may be empty.
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: String,
    pub then_children: Vec<ChildExpr>,
    pub else_children: Vec<ChildExpr>,
    pub span: Span,
}

/// A single `name: value` entry in a view body or a `style:` block.
#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Vec<StrSegment>),
    /// A bare name: a state reference (possibly a dotted path into a
    /// record, `person.name`) or an enum-like value such as a font style.
    /// Resolved during lowering.
    Ident(String),
    /// A logic call: `increment(count)`. Only valid inside an action.
    Call { function: String, args: Vec<Expr> },
    /// A record literal: `Person(name: "Ada", bio: "...")`. Distinguished
    /// from a call by its named arguments; valid as a state initializer.
    RecordLit { name: String, fields: Vec<Prop> },
    /// An action: `count = increment(count)` inside an `on_click: { ... }`
    /// block — call a logic function and assign the result to a state.
    Assign { target: String, call: Box<Expr> },
}
