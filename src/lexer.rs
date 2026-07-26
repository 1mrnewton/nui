//! Hand-written tokenizer for nui source.
//!
//! String literals are tokenized straight into interpolation segments
//! (`"Count: {count}"` becomes `[Literal("Count: "), Interp("count")]`) so
//! the parser never re-scans string contents.

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum StrSegment {
    Literal(String),
    /// A `{stateName}` or `{state.field}` interpolation.
    Interp(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Component,
    Type,
    State,
    Logic,
    Fn,
    If,
    Else,
    /// Retired keyword, kept so the parser can give a helpful error.
    Event,
    True,
    False,
    // Literals and names
    Ident(String),
    Int(i64),
    Float(f64),
    Str(Vec<StrSegment>),
    // Punctuation
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Comma,
    Dot,
    Eq,
    Arrow,
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Component => "`component`".into(),
            TokenKind::Type => "`type`".into(),
            TokenKind::State => "`state`".into(),
            TokenKind::Logic => "`logic`".into(),
            TokenKind::Fn => "`fn`".into(),
            TokenKind::If => "`if`".into(),
            TokenKind::Else => "`else`".into(),
            TokenKind::Event => "`event`".into(),
            TokenKind::True => "`true`".into(),
            TokenKind::False => "`false`".into(),
            TokenKind::Ident(name) => format!("`{name}`"),
            TokenKind::Int(_) | TokenKind::Float(_) => "a number".into(),
            TokenKind::Str(_) => "a string".into(),
            TokenKind::LBrace => "`{`".into(),
            TokenKind::RBrace => "`}`".into(),
            TokenKind::LParen => "`(`".into(),
            TokenKind::RParen => "`)`".into(),
            TokenKind::Colon => "`:`".into(),
            TokenKind::Comma => "`,`".into(),
            TokenKind::Dot => "`.`".into(),
            TokenKind::Eq => "`=`".into(),
            TokenKind::Arrow => "`->`".into(),
            TokenKind::Eof => "end of file".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub fn lex(source: &str) -> Result<Vec<Token>> {
    Lexer {
        chars: source.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
    }
    .run()
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn run(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_trivia();
            let (line, col) = (self.line, self.col);
            let Some(c) = self.peek() else {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    line,
                    col,
                });
                return Ok(tokens);
            };
            let kind = match c {
                '{' => self.single(TokenKind::LBrace),
                '}' => self.single(TokenKind::RBrace),
                '(' => self.single(TokenKind::LParen),
                ')' => self.single(TokenKind::RParen),
                ':' => self.single(TokenKind::Colon),
                ',' => self.single(TokenKind::Comma),
                '.' => self.single(TokenKind::Dot),
                '=' => self.single(TokenKind::Eq),
                '"' => self.string(line, col)?,
                '-' if self.peek2() == Some('>') => {
                    self.bump();
                    self.bump();
                    TokenKind::Arrow
                }
                '-' if self.peek2().is_some_and(|c| c.is_ascii_digit()) => {
                    self.number(line, col)?
                }
                c if c.is_ascii_digit() => self.number(line, col)?,
                c if is_ident_start(c) => self.ident(),
                other => {
                    return Err(Error::new(
                        format!("unexpected character `{other}`"),
                        line,
                        col,
                    ));
                }
            };
            tokens.push(Token { kind, line, col });
        }
    }

    fn single(&mut self, kind: TokenKind) -> TokenKind {
        self.bump();
        kind
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek2() == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    fn ident(&mut self) -> TokenKind {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if !is_ident_continue(c) {
                break;
            }
            name.push(c);
            self.bump();
        }
        match name.as_str() {
            "component" => TokenKind::Component,
            "type" => TokenKind::Type,
            "state" => TokenKind::State,
            "logic" => TokenKind::Logic,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "event" => TokenKind::Event,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Ident(name),
        }
    }

    fn number(&mut self, line: usize, col: usize) -> Result<TokenKind> {
        let mut text = String::new();
        if self.peek() == Some('-') {
            text.push('-');
            self.bump();
        }
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            text.push(c);
            self.bump();
        }
        // Only treat `.` as a decimal point when a digit follows, so a
        // modifier chain after a number never gets swallowed.
        if self.peek() == Some('.') && self.peek2().is_some_and(|c| c.is_ascii_digit()) {
            text.push('.');
            self.bump();
            while let Some(c) = self.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                text.push(c);
                self.bump();
            }
            let value: f64 = text
                .parse()
                .map_err(|_| Error::new(format!("invalid number `{text}`"), line, col))?;
            Ok(TokenKind::Float(value))
        } else {
            let value: i64 = text
                .parse()
                .map_err(|_| Error::new(format!("invalid number `{text}`"), line, col))?;
            Ok(TokenKind::Int(value))
        }
    }

    fn string(&mut self, line: usize, col: usize) -> Result<TokenKind> {
        self.bump(); // opening quote
        let mut segments: Vec<StrSegment> = Vec::new();
        let mut current = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(Error::new("unterminated string literal", line, col));
            };
            match c {
                '"' => {
                    self.bump();
                    break;
                }
                '\n' => return Err(Error::new("unterminated string literal", line, col)),
                '\\' => {
                    self.bump();
                    let (esc_line, esc_col) = (self.line, self.col);
                    let Some(escaped) = self.bump() else {
                        return Err(Error::new("unterminated string literal", line, col));
                    };
                    let resolved = match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        't' => '\t',
                        '{' => '{',
                        '}' => '}',
                        other => {
                            return Err(Error::new(
                                format!("unknown escape `\\{other}`"),
                                esc_line,
                                esc_col,
                            ));
                        }
                    };
                    current.push(resolved);
                }
                '{' => {
                    let (interp_line, interp_col) = (self.line, self.col);
                    self.bump();
                    if !current.is_empty() {
                        segments.push(StrSegment::Literal(std::mem::take(&mut current)));
                    }
                    let mut name = String::new();
                    while let Some(c) = self.peek() {
                        if !is_ident_continue(c) && c != '.' {
                            break;
                        }
                        name.push(c);
                        self.bump();
                    }
                    // A state name, or a dotted path into a record.
                    let valid = !name.is_empty()
                        && name
                            .split('.')
                            .all(|seg| seg.chars().next().is_some_and(is_ident_start));
                    if !valid {
                        return Err(Error::new(
                            "expected a state name (or `state.field` path) inside `{...}` \
                             interpolation",
                            interp_line,
                            interp_col,
                        ));
                    }
                    if self.peek() != Some('}') {
                        return Err(Error::new(
                            "expected `}` to close interpolation",
                            self.line,
                            self.col,
                        ));
                    }
                    self.bump();
                    segments.push(StrSegment::Interp(name));
                }
                _ => {
                    current.push(c);
                    self.bump();
                }
            }
        }
        if !current.is_empty() || segments.is_empty() {
            segments.push(StrSegment::Literal(current));
        }
        Ok(TokenKind::Str(segments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_interpolated_string() {
        let tokens = lex(r#""Count: {count}!""#).unwrap();
        assert_eq!(
            tokens[0].kind,
            TokenKind::Str(vec![
                StrSegment::Literal("Count: ".into()),
                StrSegment::Interp("count".into()),
                StrSegment::Literal("!".into()),
            ])
        );
    }

    #[test]
    fn number_followed_by_modifier_dot_stays_int() {
        let tokens = lex("padding(24).font").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("padding".into()),
                TokenKind::LParen,
                TokenKind::Int(24),
                TokenKind::RParen,
                TokenKind::Dot,
                TokenKind::Ident("font".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_comments() {
        let tokens = lex("// hello\nstate").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::State);
        assert_eq!(tokens[0].line, 2);
    }
}
