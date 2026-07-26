//! The nui intermediate representation (IR).
//!
//! This is the portable contract between the nui compiler and the platform
//! runtimes. The compiler lowers `.nui` source into this tree and serializes
//! it as JSON; a Swift package renders it with SwiftUI and a Kotlin library
//! renders it with Jetpack Compose.
//!
//! The runtime model: **nui owns the state, the backend owns the logic.**
//!
//! - `Component::state` declares the state schema and initial values. The
//!   generated store keeps the canonical state and renders from it.
//! - `Component::functions` declares the typed interface the logic layer
//!   (Rust behind FFI) implements — pure functions, no globals.
//! - The UI never computes values; it routes. An [`Action`] is declarative:
//!   "call this function with these inputs, assign the result to this
//!   state". Computation happens in the logic layer only.
//! - `TextField` writes its text into the bound state directly (local,
//!   UI-owned mutation — no computation involved).
//!
//! Everything here serializes with stable camelCase names so Swift `Codable`
//! and `kotlinx.serialization` decoders can be written against it directly.

use serde::{Deserialize, Serialize};

/// Bumped whenever the IR shape changes incompatibly. Runtimes check it
/// before decoding the rest of the document.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub format_version: u32,
    pub component: Component,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub name: String,
    pub state: Vec<StateDecl>,
    pub functions: Vec<FunctionDecl>,
    pub root: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
    pub initial: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
}

/// A literal value. Untagged in JSON: `0`, `1.5`, `true`, `"hi"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// A declared logic function, implemented by the backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub returns: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

/// A view action: call `function` with `args`, assign the result to `state`.
/// Fully checked at compile time against the declared functions and state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub state: String,
    pub function: String,
    pub args: Vec<CallArg>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CallArg {
    /// Pass the current value of a state.
    State { name: String },
    /// Pass a literal.
    Value { value: Value },
}

/// A view node. Tagged in JSON as `{"type": "vStack", ...}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Node {
    Text {
        content: TextContent,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    Button {
        label: TextContent,
        action: Action,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    TextField {
        /// Name of the String state this field reads from and writes to.
        binding: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    Image {
        source: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    VStack {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        children: Vec<Node>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    HStack {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        children: Vec<Node>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    List {
        children: Vec<Node>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<Modifier>,
    },
    Spacer,
    /// Structural branch driven by a Bool state: exactly one branch is in
    /// the layout at a time. Checked at compile time: `condition` names a
    /// declared Bool state.
    If {
        condition: String,
        then_children: Vec<Node>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        else_children: Vec<Node>,
    },
}

/// Text with `{state}` interpolation resolved into segments, so runtimes
/// never have to parse strings at render time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextContent(pub Vec<TextSegment>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TextSegment {
    Literal { value: String },
    State { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "camelCase")]
pub enum Modifier {
    Padding { value: f64 },
    Font { style: FontStyle },
    ForegroundColor { color: Color },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontStyle {
    LargeTitle,
    Title,
    Headline,
    Body,
    Caption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Color {
    Primary,
    Secondary,
    Red,
    Green,
    Blue,
    Orange,
    Yellow,
    Purple,
    Gray,
}
