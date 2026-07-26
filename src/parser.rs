//! Recursive-descent parser. See docs/GRAMMAR.md for the grammar.

use crate::ast::{
    ChildExpr, Component, Document, Expr, ExprKind, FnDecl, IfExpr, NodeExpr, Param, Prop, Span,
    StateDecl, TypeDecl,
};
use crate::error::{Error, Result};
use crate::lexer::{lex, Token, TokenKind};

pub fn parse(source: &str) -> Result<Document> {
    let tokens = lex(source)?;
    Parser { tokens, pos: 0 }.document()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    /// Kind of the token after the current one (Eof when out of range).
    fn peek2_kind(&self) -> &TokenKind {
        self.tokens
            .get(self.pos + 1)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn span(&self) -> Span {
        let token = self.peek();
        Span {
            line: token.line,
            col: token.col,
        }
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        // Clamp on the trailing Eof token so peek/bump never go out of range.
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn error_here(&self, message: impl Into<String>) -> Error {
        let token = self.peek();
        Error::new(message, token.line, token.col)
    }

    fn expect(&mut self, kind: TokenKind, what: &str) -> Result<Token> {
        if self.peek_kind() == &kind {
            Ok(self.bump())
        } else {
            Err(self.error_here(format!(
                "expected {what}, found {}",
                self.peek_kind().describe()
            )))
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<(String, Span)> {
        let span = self.span();
        match self.peek_kind() {
            TokenKind::Ident(_) => {
                let TokenKind::Ident(name) = self.bump().kind else {
                    unreachable!()
                };
                Ok((name, span))
            }
            other => Err(self.error_here(format!(
                "expected {what}, found {}",
                other.describe()
            ))),
        }
    }

    fn document(&mut self) -> Result<Document> {
        let mut types = Vec::new();
        while self.peek_kind() == &TokenKind::Type {
            types.push(self.type_decl()?);
        }
        let component = self.component()?;
        if self.peek_kind() != &TokenKind::Eof {
            return Err(self.error_here(format!(
                "expected end of file after the component, found {}",
                self.peek_kind().describe()
            )));
        }
        Ok(Document { types, component })
    }

    /// Parses `type Name { field: Type ... }` (commas optional).
    fn type_decl(&mut self) -> Result<TypeDecl> {
        let span = self.span();
        self.expect(TokenKind::Type, "`type`")?;
        let (name, _) = self.expect_ident("a type name")?;
        self.expect(TokenKind::LBrace, "`{` to open the type body")?;
        let mut fields = Vec::new();
        loop {
            match self.peek_kind() {
                TokenKind::RBrace => {
                    self.bump();
                    break;
                }
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::Ident(_) => {
                    let field_span = self.span();
                    let (field_name, _) = self.expect_ident("a field name")?;
                    self.expect(TokenKind::Colon, "`:` followed by the field type")?;
                    let (ty, _) =
                        self.expect_ident("a field type (Int, Float, Bool, or String)")?;
                    fields.push(Param {
                        name: field_name,
                        ty,
                        span: field_span,
                    });
                }
                other => {
                    return Err(self.error_here(format!(
                        "expected a field (`name: Type`) or `}}` in `type {name}`, found {}",
                        other.describe()
                    )));
                }
            }
        }
        Ok(TypeDecl { name, fields, span })
    }

    fn component(&mut self) -> Result<Component> {
        let span = self.span();
        self.expect(TokenKind::Component, "`component`")?;
        let (name, _) = self.expect_ident("a component name")?;
        self.expect(TokenKind::LBrace, "`{` to open the component body")?;

        let mut states = Vec::new();
        let mut functions = Vec::new();
        loop {
            match self.peek_kind() {
                TokenKind::State => states.push(self.state_decl()?),
                TokenKind::Logic => self.logic_block(&mut functions)?,
                TokenKind::Event => {
                    return Err(self.error_here(
                        "`event` was replaced by logic functions: declare \
                         `logic { fn name(arg: Type) -> Type }` and use \
                         `on_click: { state = name(args) }`",
                    ));
                }
                _ => break,
            }
        }

        if self.peek_kind() == &TokenKind::RBrace {
            return Err(self.error_here(
                "a component needs exactly one root view (e.g. `VStack { ... }`)",
            ));
        }
        let root = self.node()?;
        self.expect(
            TokenKind::RBrace,
            "`}` to close the component (a component has exactly one root view)",
        )?;
        Ok(Component {
            name,
            states,
            functions,
            root,
            span,
        })
    }

    fn state_decl(&mut self) -> Result<StateDecl> {
        let span = self.span();
        self.expect(TokenKind::State, "`state`")?;
        let (name, _) = self.expect_ident("a state name")?;
        self.expect(TokenKind::Colon, "`:` followed by a type")?;
        let (ty, _) =
            self.expect_ident("a type (Int, Float, Bool, String, or a declared type)")?;
        self.expect(TokenKind::Eq, "`=` followed by an initial value")?;
        let initial = self.literal_expr("a literal initial value")?;
        Ok(StateDecl {
            name,
            ty,
            initial,
            span,
        })
    }

    fn logic_block(&mut self, functions: &mut Vec<FnDecl>) -> Result<()> {
        self.expect(TokenKind::Logic, "`logic`")?;
        self.expect(TokenKind::LBrace, "`{` to open the logic block")?;
        while self.peek_kind() != &TokenKind::RBrace {
            functions.push(self.fn_decl()?);
        }
        self.bump(); // closing brace
        Ok(())
    }

    fn fn_decl(&mut self) -> Result<FnDecl> {
        let span = self.span();
        self.expect(TokenKind::Fn, "`fn` (or `}` to close the logic block)")?;
        let (name, _) = self.expect_ident("a function name")?;
        self.expect(TokenKind::LParen, "`(` after the function name")?;
        let mut params = Vec::new();
        if self.peek_kind() != &TokenKind::RParen {
            loop {
                let param_span = self.span();
                let (param_name, _) = self.expect_ident("a parameter name")?;
                self.expect(TokenKind::Colon, "`:` followed by the parameter type")?;
                let (ty, _) =
                    self.expect_ident("a type (Int, Float, Bool, String, or a declared type)")?;
                params.push(Param {
                    name: param_name,
                    ty,
                    span: param_span,
                });
                if self.peek_kind() == &TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "`)` to close the parameter list")?;
        self.expect(TokenKind::Arrow, "`->` followed by the return type")?;
        let (returns, _) = self.expect_ident("a return type")?;
        Ok(FnDecl {
            name,
            params,
            returns,
            span,
        })
    }

    /// Parses a view: `Name { prop: value  style: { ... }  Child { ... } }`.
    ///
    /// Inside the body, an identifier followed by `:` is a keyed entry
    /// (`style:` and `on_click:` take `{ ... }` blocks, everything else a
    /// plain value); any other identifier starts a child view. Commas
    /// between entries are optional.
    fn node(&mut self) -> Result<NodeExpr> {
        let (name, span) = self.expect_ident("a view name (e.g. `VStack`, `Text`)")?;

        if self.peek_kind() == &TokenKind::LParen {
            return Err(self.error_here(format!(
                "views take `name: value` properties in a `{{ ... }}` body, not \
                 `(...)` arguments — e.g. `{name} {{ spacing: 16 }}`"
            )));
        }

        let mut props = Vec::new();
        let mut styles = Vec::new();
        let mut children = Vec::new();
        if self.peek_kind() == &TokenKind::LBrace {
            self.bump();
            loop {
                match self.peek_kind() {
                    TokenKind::RBrace => {
                        self.bump();
                        break;
                    }
                    TokenKind::Eof => {
                        return Err(self.error_here(format!(
                            "expected `}}` to close the body of `{name}`"
                        )));
                    }
                    TokenKind::Comma => {
                        self.bump();
                    }
                    TokenKind::If => children.push(ChildExpr::If(self.if_expr()?)),
                    TokenKind::Ident(_) if self.peek2_kind() == &TokenKind::Colon => {
                        let (key, key_span) = self.expect_ident("a property name")?;
                        self.bump(); // colon
                        match key.as_str() {
                            "style" => self.style_block(&mut styles)?,
                            "on_click" => props.push(Prop {
                                name: key,
                                value: self.action_block()?,
                                span: key_span,
                            }),
                            _ => props.push(Prop {
                                name: key,
                                value: self.expr()?,
                                span: key_span,
                            }),
                        }
                    }
                    TokenKind::Ident(_) => children.push(ChildExpr::Node(self.node()?)),
                    other => {
                        return Err(self.error_here(format!(
                            "expected a property (`name: value`) or a child view \
                             in the body of `{name}`, found {}",
                            other.describe()
                        )));
                    }
                }
            }
        }

        if self.peek_kind() == &TokenKind::Dot {
            return Err(self.error_here(format!(
                "`.modifier(...)` chains were replaced by `style:` — write \
                 `style: {{ padding: 24 }}` inside `{name}`"
            )));
        }

        Ok(NodeExpr {
            name,
            props,
            styles,
            children,
            span,
        })
    }

    /// Parses `if state { children } [else { children }]` in child position.
    fn if_expr(&mut self) -> Result<IfExpr> {
        let span = self.span();
        self.expect(TokenKind::If, "`if`")?;
        let (condition, _) = self.expect_ident("a Bool state name after `if`")?;
        let condition = self.dotted_path(condition)?;
        self.expect(TokenKind::LBrace, "`{` to open the `if` branch")?;
        let then_children = self.child_list("if")?;
        let mut else_children = Vec::new();
        if self.peek_kind() == &TokenKind::Else {
            self.bump();
            if self.peek_kind() == &TokenKind::If {
                return Err(self.error_here(
                    "`else if` is not supported yet — nest an `if` inside `else { ... }`",
                ));
            }
            self.expect(TokenKind::LBrace, "`{` to open the `else` branch")?;
            else_children = self.child_list("else")?;
        }
        Ok(IfExpr {
            condition,
            then_children,
            else_children,
            span,
        })
    }

    /// Parses children until `}` — views and nested `if`s only; branches
    /// carry no properties of their own.
    fn child_list(&mut self, context: &str) -> Result<Vec<ChildExpr>> {
        let mut children = Vec::new();
        loop {
            match self.peek_kind() {
                TokenKind::RBrace => {
                    self.bump();
                    return Ok(children);
                }
                TokenKind::Eof => {
                    return Err(self.error_here(format!(
                        "expected `}}` to close the `{context}` branch"
                    )));
                }
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::If => children.push(ChildExpr::If(self.if_expr()?)),
                TokenKind::Ident(_) if self.peek2_kind() == &TokenKind::Colon => {
                    return Err(self.error_here(format!(
                        "an `{context}` branch holds child views only — \
                         properties belong on the views inside"
                    )));
                }
                TokenKind::Ident(_) => children.push(ChildExpr::Node(self.node()?)),
                other => {
                    return Err(self.error_here(format!(
                        "expected a child view or `}}` in the `{context}` branch, found {}",
                        other.describe()
                    )));
                }
            }
        }
    }

    /// Parses the `{ name: value ... }` block after `style:`.
    fn style_block(&mut self, styles: &mut Vec<Prop>) -> Result<()> {
        self.expect(
            TokenKind::LBrace,
            "`{` after `style:` — styles are written `style: { padding: 24 }`",
        )?;
        loop {
            match self.peek_kind() {
                TokenKind::RBrace => {
                    self.bump();
                    return Ok(());
                }
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::Ident(_) if self.peek2_kind() == &TokenKind::Colon => {
                    let (key, key_span) = self.expect_ident("a style property name")?;
                    self.bump(); // colon
                    styles.push(Prop {
                        name: key,
                        value: self.expr()?,
                        span: key_span,
                    });
                }
                other => {
                    return Err(self.error_here(format!(
                        "expected a style property (`padding: 24`) or `}}`, found {}",
                        other.describe()
                    )));
                }
            }
        }
    }

    /// Parses the `{ state = fn(args) }` block after `on_click:`.
    fn action_block(&mut self) -> Result<Expr> {
        self.expect(
            TokenKind::LBrace,
            "`{` — actions are written `on_click: { state = fn(args) }`",
        )?;
        let action = self.expr()?;
        if !matches!(action.kind, ExprKind::Assign { .. }) {
            return Err(Error::new(
                "expected an action: `state = function(args)`",
                action.span.line,
                action.span.col,
            ));
        }
        self.expect(
            TokenKind::RBrace,
            "`}` to close `on_click:` (one action per block, for now)",
        )?;
        Ok(action)
    }

    fn expr(&mut self) -> Result<Expr> {
        let span = self.span();
        let kind = match self.peek_kind() {
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Str(_) => match self.bump().kind {
                TokenKind::Int(value) => ExprKind::Int(value),
                TokenKind::Float(value) => ExprKind::Float(value),
                TokenKind::True => ExprKind::Bool(true),
                TokenKind::False => ExprKind::Bool(false),
                TokenKind::Str(segments) => ExprKind::Str(segments),
                _ => unreachable!(),
            },
            TokenKind::Ident(_) => {
                let (name, _) = self.expect_ident("a value")?;
                let name = self.dotted_path(name)?;
                match self.peek_kind() {
                    // `increment(count)` — a logic call (or a record literal)
                    TokenKind::LParen => self.call_expr(name)?,
                    // `count = increment(count)` — an action
                    TokenKind::Eq => {
                        self.bump();
                        let call_span = self.span();
                        let (function, _) =
                            self.expect_ident("a logic function name after `=`")?;
                        if self.peek_kind() != &TokenKind::LParen {
                            return Err(self.error_here(format!(
                                "expected `(` — an action assigns a logic call, \
                                 e.g. `{name} = {function}(...)`"
                            )));
                        }
                        let call_kind = self.call_expr(function)?;
                        ExprKind::Assign {
                            target: name,
                            call: Box::new(Expr {
                                kind: call_kind,
                                span: call_span,
                            }),
                        }
                    }
                    _ => ExprKind::Ident(name),
                }
            }
            other => {
                return Err(self.error_here(format!(
                    "expected a value, found {}",
                    other.describe()
                )));
            }
        };
        Ok(Expr { kind, span })
    }

    /// Extends `name` with `.field` segments into a dotted path
    /// (`person.name`). Returns `name` unchanged when no dot follows.
    fn dotted_path(&mut self, mut name: String) -> Result<String> {
        while self.peek_kind() == &TokenKind::Dot
            && matches!(self.peek2_kind(), TokenKind::Ident(_))
        {
            self.bump(); // dot
            let (segment, _) = self.expect_ident("a field name after `.`")?;
            name.push('.');
            name.push_str(&segment);
        }
        Ok(name)
    }

    /// Parses `(...)` after a name; the `(` is peeked, not consumed.
    /// Positional arguments make a logic call (`increment(count)`); named
    /// arguments make a record literal (`Person(name: "Ada")`).
    fn call_expr(&mut self, function: String) -> Result<ExprKind> {
        self.expect(TokenKind::LParen, "`(`")?;
        if matches!(self.peek_kind(), TokenKind::Ident(_))
            && self.peek2_kind() == &TokenKind::Colon
        {
            let mut fields = Vec::new();
            loop {
                let field_span = self.span();
                let (field, _) = self.expect_ident("a field name")?;
                self.expect(TokenKind::Colon, "`:` after the field name")?;
                fields.push(Prop {
                    name: field,
                    value: self.expr()?,
                    span: field_span,
                });
                if self.peek_kind() == &TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RParen, "`)` to close the record literal")?;
            return Ok(ExprKind::RecordLit {
                name: function,
                fields,
            });
        }
        let mut args = Vec::new();
        if self.peek_kind() != &TokenKind::RParen {
            loop {
                args.push(self.expr()?);
                if self.peek_kind() == &TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "`)` to close the call")?;
        Ok(ExprKind::Call { function, args })
    }

    fn literal_expr(&mut self, what: &str) -> Result<Expr> {
        let expr = self.expr()?;
        if !matches!(
            expr.kind,
            ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Bool(_)
                | ExprKind::Str(_)
                | ExprKind::RecordLit { .. }
        ) {
            return Err(Error::new(
                format!("expected {what}"),
                expr.span.line,
                expr.span.col,
            ));
        }
        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_with_props_styles_and_children() {
        let doc = parse(
            r#"
            component X {
                state title: String = "hi"
                logic {
                    fn shout(text: String) -> String
                }
                VStack {
                    spacing: 8
                    style: { padding: 24 }

                    Text { text: title }
                    Button {
                        label: "go"
                        on_click: { title = shout(title) }
                    }
                }
            }
            "#,
        )
        .unwrap();
        let component = doc.component;
        assert_eq!(component.name, "X");
        assert_eq!(component.states.len(), 1);
        assert_eq!(component.functions.len(), 1);
        assert_eq!(component.functions[0].name, "shout");
        assert_eq!(component.functions[0].params.len(), 1);
        assert_eq!(component.functions[0].returns, "String");
        assert_eq!(component.root.name, "VStack");
        assert_eq!(component.root.children.len(), 2);
        assert_eq!(component.root.props.len(), 1);
        assert_eq!(component.root.props[0].name, "spacing");
        assert_eq!(component.root.styles.len(), 1);
        assert_eq!(component.root.styles[0].name, "padding");
        let ChildExpr::Node(button) = &component.root.children[1] else {
            panic!("expected a view child");
        };
        assert_eq!(button.props[0].name, "label");
        let action = &button.props[1];
        assert_eq!(action.name, "on_click");
        assert!(matches!(&action.value.kind, ExprKind::Assign { target, .. } if target == "title"));
    }

    #[test]
    fn parses_if_else_in_child_position() {
        let doc = parse(
            r#"
            component X {
                state on: Bool = false
                VStack {
                    if on {
                        Text { text: "yes" }
                        Text { text: "still yes" }
                    } else {
                        Text { text: "no" }
                    }
                }
            }
            "#,
        )
        .unwrap();
        let ChildExpr::If(if_expr) = &doc.component.root.children[0] else {
            panic!("expected an if child");
        };
        assert_eq!(if_expr.condition, "on");
        assert_eq!(if_expr.then_children.len(), 2);
        assert_eq!(if_expr.else_children.len(), 1);
    }

    #[test]
    fn else_if_gets_a_helpful_error() {
        let err = parse(
            r#"
            component X {
                state on: Bool = false
                VStack {
                    if on { Spacer } else if on { Spacer }
                }
            }
            "#,
        )
        .unwrap_err();
        assert!(err.message.contains("nest an `if`"), "got: {}", err.message);
    }

    #[test]
    fn properties_inside_branches_are_rejected() {
        let err = parse(
            r#"
            component X {
                state on: Bool = false
                VStack {
                    if on { spacing: 4 }
                }
            }
            "#,
        )
        .unwrap_err();
        assert!(
            err.message.contains("child views only"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn commas_between_entries_are_optional() {
        let doc = parse(
            r#"
            component X {
                state n: Int = 0
                logic { fn bump(n: Int) -> Int }
                Button { label: "+", on_click: { n = bump(n) } }
            }
            "#,
        )
        .unwrap();
        assert_eq!(doc.component.root.props.len(), 2);
    }

    #[test]
    fn paren_arguments_get_migration_error() {
        let err = parse(r#"component X { VStack(spacing: 8) { Spacer } }"#).unwrap_err();
        assert!(err.message.contains("name: value"), "got: {}", err.message);
    }

    #[test]
    fn modifier_chains_get_migration_error() {
        let err = parse(r#"component X { Text { text: "hi" }.font(title) }"#).unwrap_err();
        assert!(err.message.contains("style:"), "got: {}", err.message);
    }

    #[test]
    fn on_click_requires_a_block() {
        let err = parse(
            r#"
            component X {
                state n: Int = 0
                logic { fn bump(n: Int) -> Int }
                Button { label: "+", on_click: n = bump(n) }
            }
            "#,
        )
        .unwrap_err();
        assert!(
            err.message.contains("on_click: { state = fn(args) }"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn event_keyword_gets_migration_error() {
        let err = parse("component X { event tap Spacer }").unwrap_err();
        assert!(err.message.contains("logic"), "got: {}", err.message);
    }

    #[test]
    fn reports_missing_root_view() {
        let err = parse("component X { state n: Int = 0 }").unwrap_err();
        assert!(err.message.contains("root view"), "got: {}", err.message);
    }

    #[test]
    fn parses_type_declarations_and_record_literals() {
        let doc = parse(
            r#"
            type Person {
                name: String
                bio: String
            }
            component X {
                state p: Person = Person(name: "Ada", bio: "First programmer.")
                Text { text: "{p.name}" }
            }
            "#,
        )
        .unwrap();
        assert_eq!(doc.types.len(), 1);
        assert_eq!(doc.types[0].name, "Person");
        assert_eq!(doc.types[0].fields.len(), 2);
        assert_eq!(doc.types[0].fields[0].name, "name");
        assert_eq!(doc.types[0].fields[0].ty, "String");
        let ExprKind::RecordLit { name, fields } = &doc.component.states[0].initial.kind else {
            panic!("expected a record literal initializer");
        };
        assert_eq!(name, "Person");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "name");
    }

    #[test]
    fn parses_dotted_paths_in_values_and_conditions() {
        let doc = parse(
            r#"
            component X {
                state ready: Bool = false
                logic { fn greet(name: String) -> String }
                VStack {
                    if flags.ready { Spacer }
                    Button { label: "go", on_click: { title = greet(p.name) } }
                }
            }
            "#,
        )
        .unwrap();
        let ChildExpr::If(if_expr) = &doc.component.root.children[0] else {
            panic!("expected an if child");
        };
        assert_eq!(if_expr.condition, "flags.ready");
        let ChildExpr::Node(button) = &doc.component.root.children[1] else {
            panic!("expected a view child");
        };
        let ExprKind::Assign { call, .. } = &button.props[1].value.kind else {
            panic!("expected an action");
        };
        let ExprKind::Call { args, .. } = &call.kind else {
            panic!("expected a call");
        };
        assert!(matches!(&args[0].kind, ExprKind::Ident(name) if name == "p.name"));
    }
}
