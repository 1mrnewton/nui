//! Lowers the parsed AST into the serializable IR.
//!
//! All semantic checking happens here: view and style-property names,
//! property shapes, state references (including inside `{...}`
//! interpolations), state initial-value types, and full type checking of
//! actions against the declared logic functions.

use std::collections::{HashMap, HashSet};

use crate::ast::{self, Expr, ExprKind, Span};
use crate::error::{Error, Result};
use crate::ir;
use crate::lexer::StrSegment;

pub fn lower(doc: &ast::Document) -> Result<ir::Document> {
    let component = &doc.component;

    let mut state = Vec::new();
    let mut state_types: HashMap<String, ir::Type> = HashMap::new();
    for decl in &component.states {
        if state_types.contains_key(&decl.name) {
            return Err(Error::new(
                format!("state `{}` is declared twice", decl.name),
                decl.span.line,
                decl.span.col,
            ));
        }
        let ty = parse_type(&decl.ty, decl.span)?;
        let initial = lower_initial(&decl.name, ty, &decl.initial)?;
        state_types.insert(decl.name.clone(), ty);
        state.push(ir::StateDecl {
            name: decl.name.clone(),
            ty,
            initial,
        });
    }

    let mut functions = Vec::new();
    let mut function_decls: HashMap<String, ir::FunctionDecl> = HashMap::new();
    for decl in &component.functions {
        if function_decls.contains_key(&decl.name) {
            return Err(Error::new(
                format!("function `{}` is declared twice", decl.name),
                decl.span.line,
                decl.span.col,
            ));
        }
        let params = decl
            .params
            .iter()
            .map(|param| {
                Ok(ir::Param {
                    name: param.name.clone(),
                    ty: parse_type(&param.ty, param.span)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let lowered = ir::FunctionDecl {
            name: decl.name.clone(),
            params,
            returns: parse_type(&decl.returns, decl.span)?,
        };
        function_decls.insert(decl.name.clone(), lowered.clone());
        functions.push(lowered);
    }

    let ctx = Ctx {
        state_types,
        functions: function_decls,
    };
    let root = lower_node(&ctx, &component.root)?;

    Ok(ir::Document {
        format_version: ir::FORMAT_VERSION,
        component: ir::Component {
            name: component.name.clone(),
            state,
            functions,
            root,
        },
    })
}

struct Ctx {
    state_types: HashMap<String, ir::Type>,
    functions: HashMap<String, ir::FunctionDecl>,
}

fn parse_type(name: &str, span: Span) -> Result<ir::Type> {
    match name {
        "Int" => Ok(ir::Type::Int),
        "Float" => Ok(ir::Type::Float),
        "Bool" => Ok(ir::Type::Bool),
        "String" => Ok(ir::Type::String),
        other => Err(Error::new(
            format!("unknown type `{other}`; expected Int, Float, Bool, or String"),
            span.line,
            span.col,
        )),
    }
}

fn type_name(ty: ir::Type) -> &'static str {
    match ty {
        ir::Type::Int => "Int",
        ir::Type::Float => "Float",
        ir::Type::Bool => "Bool",
        ir::Type::String => "String",
    }
}

fn lower_initial(name: &str, ty: ir::Type, expr: &Expr) -> Result<ir::Value> {
    let value = match (ty, &expr.kind) {
        (ir::Type::Int, ExprKind::Int(v)) => Some(ir::Value::Int(*v)),
        (ir::Type::Float, ExprKind::Float(v)) => Some(ir::Value::Float(*v)),
        (ir::Type::Float, ExprKind::Int(v)) => Some(ir::Value::Float(*v as f64)),
        (ir::Type::Bool, ExprKind::Bool(v)) => Some(ir::Value::Bool(*v)),
        (ir::Type::String, ExprKind::Str(segments)) => segments
            .iter()
            .map(|segment| match segment {
                StrSegment::Literal(text) => Some(text.as_str()),
                StrSegment::Interp(_) => None,
            })
            .collect::<Option<String>>()
            .map(ir::Value::String),
        _ => None,
    };
    value.ok_or_else(|| {
        Error::new(
            format!(
                "initial value for `{name}` must be a plain {} literal",
                type_name(ty)
            ),
            expr.span.line,
            expr.span.col,
        )
    })
}

fn lower_node(ctx: &Ctx, node: &ast::NodeExpr) -> Result<ir::Node> {
    let lowered = match node.name.as_str() {
        "Text" => {
            check_props("Text", node, &["text"])?;
            no_children("Text", node)?;
            let text = require_prop("Text", node, "text", r#"text: "Count: {count}""#)?;
            ir::Node::Text {
                content: text_content(ctx, &text.value)?,
                modifiers: lower_styles(node)?,
            }
        }
        "Button" => {
            check_props("Button", node, &["label", "on_click"])?;
            no_children("Button", node)?;
            let label = require_prop("Button", node, "label", r#"label: "+""#)?;
            let action = require_prop(
                "Button",
                node,
                "on_click",
                "on_click: { count = increment(count) }",
            )?;
            ir::Node::Button {
                label: text_content(ctx, &label.value)?,
                action: lower_action(ctx, &action.value)?,
                modifiers: lower_styles(node)?,
            }
        }
        "TextField" => {
            check_props("TextField", node, &["bind", "placeholder"])?;
            no_children("TextField", node)?;
            let bind = require_prop("TextField", node, "bind", "bind: someStringState")?;
            let binding = state_ref(ctx, &bind.value)?;
            if ctx.state_types[&binding] != ir::Type::String {
                return Err(Error::new(
                    format!("`TextField` binding `{binding}` must be a String state"),
                    bind.span.line,
                    bind.span.col,
                ));
            }
            let placeholder = find_prop(node, "placeholder")
                .map(|prop| plain_string(&prop.value))
                .transpose()?;
            ir::Node::TextField {
                binding,
                placeholder,
                modifiers: lower_styles(node)?,
            }
        }
        "Image" => {
            check_props("Image", node, &["source"])?;
            no_children("Image", node)?;
            let source = require_prop("Image", node, "source", r#"source: "logo""#)?;
            ir::Node::Image {
                source: plain_string(&source.value)?,
                modifiers: lower_styles(node)?,
            }
        }
        "VStack" | "HStack" => {
            check_props(&node.name, node, &["spacing"])?;
            let spacing = find_prop(node, "spacing")
                .map(|prop| number(&prop.value))
                .transpose()?;
            let children = lower_children(ctx, node)?;
            let modifiers = lower_styles(node)?;
            if node.name == "VStack" {
                ir::Node::VStack {
                    spacing,
                    children,
                    modifiers,
                }
            } else {
                ir::Node::HStack {
                    spacing,
                    children,
                    modifiers,
                }
            }
        }
        "List" => {
            check_props("List", node, &[])?;
            ir::Node::List {
                children: lower_children(ctx, node)?,
                modifiers: lower_styles(node)?,
            }
        }
        "Spacer" => {
            check_props("Spacer", node, &[])?;
            no_children("Spacer", node)?;
            if let Some(style) = node.styles.first() {
                return Err(Error::new(
                    "`Spacer` does not take a `style:` block",
                    style.span.line,
                    style.span.col,
                ));
            }
            ir::Node::Spacer
        }
        other => {
            return Err(Error::new(
                format!(
                    "unknown view `{other}`; known views are Text, Button, TextField, \
                     Image, VStack, HStack, List, and Spacer"
                ),
                node.span.line,
                node.span.col,
            ));
        }
    };
    Ok(lowered)
}

fn lower_children(ctx: &Ctx, node: &ast::NodeExpr) -> Result<Vec<ir::Node>> {
    node.children
        .iter()
        .map(|child| lower_child(ctx, child))
        .collect()
}

fn lower_child(ctx: &Ctx, child: &ast::ChildExpr) -> Result<ir::Node> {
    match child {
        ast::ChildExpr::Node(node) => lower_node(ctx, node),
        ast::ChildExpr::If(if_expr) => {
            let condition = &if_expr.condition;
            let Some(ty) = ctx.state_types.get(condition).copied() else {
                return Err(Error::new(
                    format!(
                        "unknown state `{condition}`; an `if` condition names a \
                         declared Bool state"
                    ),
                    if_expr.span.line,
                    if_expr.span.col,
                ));
            };
            if ty != ir::Type::Bool {
                return Err(Error::new(
                    format!(
                        "`if {condition}` needs a Bool state, but `{condition}` is {}",
                        type_name(ty)
                    ),
                    if_expr.span.line,
                    if_expr.span.col,
                ));
            }
            Ok(ir::Node::If {
                condition: condition.clone(),
                then_children: if_expr
                    .then_children
                    .iter()
                    .map(|child| lower_child(ctx, child))
                    .collect::<Result<Vec<_>>>()?,
                else_children: if_expr
                    .else_children
                    .iter()
                    .map(|child| lower_child(ctx, child))
                    .collect::<Result<Vec<_>>>()?,
            })
        }
    }
}

fn lower_styles(node: &ast::NodeExpr) -> Result<Vec<ir::Modifier>> {
    node.styles.iter().map(lower_style).collect()
}

fn lower_style(style: &ast::Prop) -> Result<ir::Modifier> {
    match style.name.as_str() {
        "padding" => Ok(ir::Modifier::Padding {
            value: number(&style.value)?,
        }),
        "font" => Ok(ir::Modifier::Font {
            style: font_style(&style.value)?,
        }),
        "color" => Ok(ir::Modifier::ForegroundColor {
            color: color(&style.value)?,
        }),
        other => Err(Error::new(
            format!(
                "unknown style property `{other}`; known properties are padding, \
                 font, and color"
            ),
            style.span.line,
            style.span.col,
        )),
    }
}

// --- property plumbing ---

fn check_props(kind: &str, node: &ast::NodeExpr, allowed: &[&str]) -> Result<()> {
    let mut seen = HashSet::new();
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(Error::new(
                format!("unknown property `{}:` for `{kind}`", prop.name),
                prop.span.line,
                prop.span.col,
            ));
        }
        if !seen.insert(prop.name.as_str()) {
            return Err(Error::new(
                format!("property `{}:` given twice", prop.name),
                prop.span.line,
                prop.span.col,
            ));
        }
    }
    Ok(())
}

fn find_prop<'a>(node: &'a ast::NodeExpr, name: &str) -> Option<&'a ast::Prop> {
    node.props.iter().find(|prop| prop.name == name)
}

fn require_prop<'a>(
    kind: &str,
    node: &'a ast::NodeExpr,
    name: &str,
    example: &str,
) -> Result<&'a ast::Prop> {
    find_prop(node, name).ok_or_else(|| {
        Error::new(
            format!("`{kind}` requires a `{name}:` property, e.g. `{example}`"),
            node.span.line,
            node.span.col,
        )
    })
}

fn no_children(kind: &str, node: &ast::NodeExpr) -> Result<()> {
    match node.children.first() {
        Some(child) => Err(Error::new(
            format!("`{kind}` does not take children"),
            child.span().line,
            child.span().col,
        )),
        None => Ok(()),
    }
}

// --- expression helpers ---

fn check_state(ctx: &Ctx, name: &str, span: Span) -> Result<()> {
    if ctx.state_types.contains_key(name) {
        Ok(())
    } else {
        Err(Error::new(
            format!("unknown state `{name}`; declare it with `state {name}: <Type> = <value>`"),
            span.line,
            span.col,
        ))
    }
}

fn text_content(ctx: &Ctx, expr: &Expr) -> Result<ir::TextContent> {
    match &expr.kind {
        ExprKind::Str(segments) => {
            let mut out = Vec::new();
            for segment in segments {
                match segment {
                    StrSegment::Literal(value) => out.push(ir::TextSegment::Literal {
                        value: value.clone(),
                    }),
                    StrSegment::Interp(name) => {
                        check_state(ctx, name, expr.span)?;
                        out.push(ir::TextSegment::State { name: name.clone() });
                    }
                }
            }
            Ok(ir::TextContent(out))
        }
        ExprKind::Ident(name) => {
            check_state(ctx, name, expr.span)?;
            Ok(ir::TextContent(vec![ir::TextSegment::State {
                name: name.clone(),
            }]))
        }
        _ => Err(Error::new(
            "expected text: a string literal or a state name",
            expr.span.line,
            expr.span.col,
        )),
    }
}

/// Lowers `count = increment(count)` into a fully type-checked [`ir::Action`].
fn lower_action(ctx: &Ctx, expr: &Expr) -> Result<ir::Action> {
    let ExprKind::Assign { target, call } = &expr.kind else {
        return Err(Error::new(
            "expected an action: `state = function(args)`",
            expr.span.line,
            expr.span.col,
        ));
    };
    let Some(target_ty) = ctx.state_types.get(target).copied() else {
        return Err(Error::new(
            format!("unknown state `{target}`; declare it with `state {target}: <Type> = <value>`"),
            expr.span.line,
            expr.span.col,
        ));
    };
    let ExprKind::Call { function, args } = &call.kind else {
        unreachable!("parser only produces Assign with a Call");
    };
    let Some(decl) = ctx.functions.get(function) else {
        return Err(Error::new(
            format!(
                "unknown function `{function}`; declare it in a `logic {{ fn {function}(...) -> ... }}` block"
            ),
            call.span.line,
            call.span.col,
        ));
    };
    if decl.returns != target_ty {
        return Err(Error::new(
            format!(
                "`{function}` returns {} but `{target}` is {}",
                type_name(decl.returns),
                type_name(target_ty)
            ),
            call.span.line,
            call.span.col,
        ));
    }
    if args.len() != decl.params.len() {
        return Err(Error::new(
            format!(
                "`{function}` takes {} argument(s), found {}",
                decl.params.len(),
                args.len()
            ),
            call.span.line,
            call.span.col,
        ));
    }
    let mut lowered_args = Vec::new();
    for (arg, param) in args.iter().zip(&decl.params) {
        lowered_args.push(lower_call_arg(ctx, function, arg, param)?);
    }
    Ok(ir::Action {
        state: target.clone(),
        function: function.clone(),
        args: lowered_args,
    })
}

fn lower_call_arg(
    ctx: &Ctx,
    function: &str,
    arg: &Expr,
    param: &ir::Param,
) -> Result<ir::CallArg> {
    let mismatch = |found: &str| {
        Error::new(
            format!(
                "`{function}` expects {} for `{}`, found {found}",
                type_name(param.ty),
                param.name
            ),
            arg.span.line,
            arg.span.col,
        )
    };
    match &arg.kind {
        ExprKind::Ident(name) => {
            let Some(state_ty) = ctx.state_types.get(name).copied() else {
                return Err(Error::new(
                    format!("unknown state `{name}`"),
                    arg.span.line,
                    arg.span.col,
                ));
            };
            if state_ty != param.ty {
                return Err(mismatch(&format!(
                    "state `{name}` of type {}",
                    type_name(state_ty)
                )));
            }
            Ok(ir::CallArg::State { name: name.clone() })
        }
        ExprKind::Int(v) => match param.ty {
            ir::Type::Int => Ok(ir::CallArg::Value {
                value: ir::Value::Int(*v),
            }),
            ir::Type::Float => Ok(ir::CallArg::Value {
                value: ir::Value::Float(*v as f64),
            }),
            _ => Err(mismatch("an Int literal")),
        },
        ExprKind::Float(v) => match param.ty {
            ir::Type::Float => Ok(ir::CallArg::Value {
                value: ir::Value::Float(*v),
            }),
            _ => Err(mismatch("a Float literal")),
        },
        ExprKind::Bool(v) => match param.ty {
            ir::Type::Bool => Ok(ir::CallArg::Value {
                value: ir::Value::Bool(*v),
            }),
            _ => Err(mismatch("a Bool literal")),
        },
        ExprKind::Str(_) => match param.ty {
            ir::Type::String => Ok(ir::CallArg::Value {
                value: ir::Value::String(plain_string(arg)?),
            }),
            _ => Err(mismatch("a String literal")),
        },
        ExprKind::Call { .. } | ExprKind::Assign { .. } => {
            Err(mismatch("a nested call (not supported)"))
        }
    }
}

fn state_ref(ctx: &Ctx, expr: &Expr) -> Result<String> {
    match &expr.kind {
        ExprKind::Ident(name) => {
            check_state(ctx, name, expr.span)?;
            Ok(name.clone())
        }
        _ => Err(Error::new(
            "expected a state name",
            expr.span.line,
            expr.span.col,
        )),
    }
}

fn plain_string(expr: &Expr) -> Result<String> {
    if let ExprKind::Str(segments) = &expr.kind {
        let mut out = String::new();
        for segment in segments {
            match segment {
                StrSegment::Literal(value) => out.push_str(value),
                StrSegment::Interp(_) => {
                    return Err(Error::new(
                        "`{...}` interpolation is not allowed here",
                        expr.span.line,
                        expr.span.col,
                    ));
                }
            }
        }
        return Ok(out);
    }
    Err(Error::new(
        "expected a string literal",
        expr.span.line,
        expr.span.col,
    ))
}

fn number(expr: &Expr) -> Result<f64> {
    match expr.kind {
        ExprKind::Int(value) => Ok(value as f64),
        ExprKind::Float(value) => Ok(value),
        _ => Err(Error::new(
            "expected a number",
            expr.span.line,
            expr.span.col,
        )),
    }
}

fn font_style(expr: &Expr) -> Result<ir::FontStyle> {
    if let ExprKind::Ident(name) = &expr.kind {
        let style = match name.as_str() {
            "largeTitle" => Some(ir::FontStyle::LargeTitle),
            "title" => Some(ir::FontStyle::Title),
            "headline" => Some(ir::FontStyle::Headline),
            "body" => Some(ir::FontStyle::Body),
            "caption" => Some(ir::FontStyle::Caption),
            _ => None,
        };
        if let Some(style) = style {
            return Ok(style);
        }
    }
    Err(Error::new(
        "expected a font style: largeTitle, title, headline, body, or caption",
        expr.span.line,
        expr.span.col,
    ))
}

fn color(expr: &Expr) -> Result<ir::Color> {
    if let ExprKind::Ident(name) = &expr.kind {
        let color = match name.as_str() {
            "primary" => Some(ir::Color::Primary),
            "secondary" => Some(ir::Color::Secondary),
            "red" => Some(ir::Color::Red),
            "green" => Some(ir::Color::Green),
            "blue" => Some(ir::Color::Blue),
            "orange" => Some(ir::Color::Orange),
            "yellow" => Some(ir::Color::Yellow),
            "purple" => Some(ir::Color::Purple),
            "gray" => Some(ir::Color::Gray),
            _ => None,
        };
        if let Some(color) = color {
            return Ok(color);
        }
    }
    Err(Error::new(
        "expected a color: primary, secondary, red, green, blue, orange, yellow, purple, or gray",
        expr.span.line,
        expr.span.col,
    ))
}
