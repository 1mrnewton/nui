//! nuic — the nui compiler.
//!
//! Usage: nuic <input.nui> [-o output.swift]
//! With no -o, the generated SwiftUI is printed to stdout.

mod ast;
mod emit_kotlin;
mod emit_swift;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: nuic <input.nui> [--target swift|compose] [-o output]");
        exit(2);
    }

    let input = &args[1];
    let mut output: Option<String> = None;
    let mut target = "swift".to_string();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" if i + 1 < args.len() => {
                output = Some(args[i + 1].clone());
                i += 2;
            }
            "--target" if i + 1 < args.len() => {
                target = args[i + 1].clone();
                i += 2;
            }
            other => {
                eprintln!("unknown argument: {other}");
                exit(2);
            }
        }
    }

    let src = fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("nuic: cannot read {input}: {e}");
        exit(1);
    });

    let toks = lexer::lex(&src).unwrap_or_else(|e| {
        eprintln!("nuic: lex error: {e}");
        exit(1);
    });

    let component = parser::Parser::new(toks).parse().unwrap_or_else(|e| {
        eprintln!("nuic: parse error: {e}");
        exit(1);
    });

    let generated = match target.as_str() {
        "swift" => emit_swift::emit(&component),
        "compose" | "kotlin" => emit_kotlin::emit(&component),
        other => {
            eprintln!("nuic: unknown target '{other}' (expected swift|compose)");
            exit(2);
        }
    };

    match output {
        Some(path) => {
            fs::write(&path, generated).unwrap_or_else(|e| {
                eprintln!("nuic: cannot write {path}: {e}");
                exit(1);
            });
            eprintln!("nuic: wrote {path}");
        }
        None => print!("{generated}"),
    }
}
