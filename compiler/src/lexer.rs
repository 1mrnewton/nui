//! Hand-rolled lexer for the .nui DSL.

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Num(String),
    Str(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Comma,
    Dot,
    Equals,
    Arrow, // ->
    Eof,
}

pub fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut toks = Vec::new();

    while i < n {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // line comment
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        match c {
            '{' => { toks.push(Tok::LBrace); i += 1; }
            '}' => { toks.push(Tok::RBrace); i += 1; }
            '(' => { toks.push(Tok::LParen); i += 1; }
            ')' => { toks.push(Tok::RParen); i += 1; }
            ':' => { toks.push(Tok::Colon); i += 1; }
            ',' => { toks.push(Tok::Comma); i += 1; }
            '.' => { toks.push(Tok::Dot); i += 1; }
            '=' => { toks.push(Tok::Equals); i += 1; }
            '-' => {
                if i + 1 < n && chars[i + 1] == '>' {
                    toks.push(Tok::Arrow);
                    i += 2;
                } else {
                    return Err(format!("unexpected '-' at offset {i}"));
                }
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                // Read raw until the closing quote. Interpolation like \(x) is
                // kept verbatim and handled by the emitter.
                while i < n && chars[i] != '"' {
                    s.push(chars[i]);
                    i += 1;
                }
                if i >= n {
                    return Err("unterminated string literal".into());
                }
                i += 1; // closing quote
                toks.push(Tok::Str(s));
            }
            _ => {
                if c.is_ascii_digit() {
                    let mut s = String::new();
                    while i < n && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        s.push(chars[i]);
                        i += 1;
                    }
                    toks.push(Tok::Num(s));
                } else if c.is_alphabetic() || c == '_' {
                    let mut s = String::new();
                    while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        s.push(chars[i]);
                        i += 1;
                    }
                    toks.push(Tok::Ident(s));
                } else {
                    return Err(format!("unexpected character '{c}' at offset {i}"));
                }
            }
        }
    }

    toks.push(Tok::Eof);
    Ok(toks)
}
