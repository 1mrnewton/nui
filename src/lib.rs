//! nui — a language-agnostic UI framework for mobile.
//!
//! This crate is the compiler: it parses `.nui` source, lowers it into a
//! portable IR (see [`ir`]), and transpiles that IR to native UI source —
//! SwiftUI on iOS ([`swift`]), Jetpack Compose on Android (planned).
//!
//! Pipeline: source → [`lexer`] → [`parser`] (AST in [`ast`]) → [`lower`]
//! (checks + lowering) → [`ir::Document`] → Swift source or IR JSON.

pub mod ast;
pub mod error;
pub mod ir;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod rust_logic;
pub mod swift;
pub mod swift_bridge;

pub use error::{Error, Result};

/// Compile nui source text into the portable IR.
pub fn compile(source: &str) -> Result<ir::Document> {
    let document = parser::parse(source)?;
    lower::lower(&document)
}
