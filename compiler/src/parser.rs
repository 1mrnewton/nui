//! Recursive-descent parser for the .nui DSL.

use crate::ast::*;
use crate::lexer::Tok;

pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    pub fn new(toks: Vec<Tok>) -> Self {
        Self { toks, pos: 0 }
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos]
    }

    fn peek2(&self) -> &Tok {
        self.toks.get(self.pos + 1).unwrap_or(&Tok::Eof)
    }

    fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, t: &Tok) -> Result<(), String> {
        if self.peek() == t {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected {:?}, got {:?}", t, self.peek()))
        }
    }

    fn ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Tok::Ident(s) => Ok(s),
            t => Err(format!("expected identifier, got {t:?}")),
        }
    }

    pub fn parse(&mut self) -> Result<Component, String> {
        let kw = self.ident()?;
        if kw != "component" {
            return Err(format!("expected 'component', got '{kw}'"));
        }
        let name = self.ident()?;
        self.expect(&Tok::LBrace)?;

        let mut state = Vec::new();
        let mut events = Vec::new();
        let mut root: Option<Element> = None;

        while self.peek() != &Tok::RBrace {
            match self.peek() {
                Tok::Ident(s) if s == "state" => state = self.parse_state()?,
                Tok::Ident(s) if s == "event" => {
                    self.advance();
                    events.push(self.ident()?);
                }
                Tok::Ident(_) => {
                    if root.is_some() {
                        return Err("component may only have one root element".into());
                    }
                    root = Some(self.parse_element()?);
                }
                other => return Err(format!("unexpected {other:?} in component body")),
            }
        }
        self.expect(&Tok::RBrace)?;

        let root = root.ok_or("component has no root view element")?;
        Ok(Component { name, state, events, root })
    }

    fn parse_state(&mut self) -> Result<Vec<Field>, String> {
        self.advance(); // 'state'
        self.expect(&Tok::LBrace)?;
        let mut fields = Vec::new();
        while self.peek() != &Tok::RBrace {
            let name = self.ident()?;
            self.expect(&Tok::Colon)?;
            let ty = self.ident()?;
            let mut default = None;
            if self.peek() == &Tok::Equals {
                self.advance();
                default = Some(self.parse_value()?);
            }
            if self.peek() == &Tok::Comma {
                self.advance();
            }
            fields.push(Field { name, ty, default });
        }
        self.expect(&Tok::RBrace)?;
        Ok(fields)
    }

    fn parse_element(&mut self) -> Result<Element, String> {
        let name = self.ident()?;
        let args = if self.peek() == &Tok::LParen {
            self.parse_args()?
        } else {
            Vec::new()
        };

        let mut modifiers = Vec::new();
        let mut event = None;
        let mut children = Vec::new();
        let mut else_children = Vec::new();

        // Children, modifiers, and the `-> event` may appear in any order.
        loop {
            match self.peek() {
                Tok::LBrace => children = self.parse_children()?,
                Tok::Dot => modifiers.push(self.parse_modifier()?),
                Tok::Arrow => {
                    self.advance();
                    event = Some(self.ident()?);
                }
                _ => break,
            }
        }

        if name == "If" {
            if let Tok::Ident(s) = self.peek() {
                if s == "else" {
                    self.advance();
                    else_children = self.parse_children()?;
                }
            }
        }

        Ok(Element {
            name,
            args,
            modifiers,
            event,
            children,
            else_children,
        })
    }

    fn parse_children(&mut self) -> Result<Vec<Element>, String> {
        self.expect(&Tok::LBrace)?;
        let mut v = Vec::new();
        while self.peek() != &Tok::RBrace {
            v.push(self.parse_element()?);
        }
        self.expect(&Tok::RBrace)?;
        Ok(v)
    }

    fn parse_modifier(&mut self) -> Result<Modifier, String> {
        self.expect(&Tok::Dot)?;
        let name = self.ident()?;
        let args = if self.peek() == &Tok::LParen {
            self.parse_args()?
        } else {
            Vec::new()
        };
        Ok(Modifier { name, args })
    }

    fn parse_args(&mut self) -> Result<Vec<Arg>, String> {
        self.expect(&Tok::LParen)?;
        let mut args = Vec::new();
        while self.peek() != &Tok::RParen {
            let name = if matches!(self.peek(), Tok::Ident(_)) && self.peek2() == &Tok::Colon {
                let n = self.ident()?;
                self.advance(); // colon
                Some(n)
            } else {
                None
            };
            let value = self.parse_value()?;
            args.push(Arg { name, value });
            if self.peek() == &Tok::Comma {
                self.advance();
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(args)
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        match self.advance() {
            Tok::Str(s) => Ok(Value::Str(s)),
            Tok::Num(n) => Ok(Value::Num(n)),
            Tok::Ident(i) => Ok(Value::Ident(i)),
            t => Err(format!("expected a value, got {t:?}")),
        }
    }
}
