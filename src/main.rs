use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
nui — compile .nui UI definitions

USAGE:
  nui build <file.nui> [--target <target>] [-o <out>]
        compile to the given target (stdout by default):
          swift         SwiftUI source (UI, store, logic protocol)
          uikit         UIKit source (experimental; same store/logic shape)
          rust          logic-crate interface: expected fn signatures + checks
          swift-bridge  adapter connecting the UI to the Rust logic
          ir            IR JSON (debugging)
        Without --target, the -o extension decides (.swift → swift,
        .rs → rust), otherwise ir.
  nui check <file.nui>
        parse and check without emitting
  nui help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut args = args.into_iter();
    let command = args
        .next()
        .ok_or_else(|| format!("missing command\n\n{USAGE}"))?;
    match command.as_str() {
        "build" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut target: Option<String> = None;
            while let Some(arg) = args.next() {
                if arg == "-o" || arg == "--out" {
                    let path = args.next().ok_or("`-o` requires a path")?;
                    output = Some(PathBuf::from(path));
                } else if arg == "-t" || arg == "--target" {
                    let value = args.next().ok_or("`--target` requires `swift` or `ir`")?;
                    target = Some(value);
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("unexpected argument `{arg}`\n\n{USAGE}"));
                }
            }
            let input = input.ok_or_else(|| format!("missing input file\n\n{USAGE}"))?;
            let target = target.unwrap_or_else(|| {
                let extension = output.as_deref().and_then(Path::extension);
                if extension.is_some_and(|ext| ext == "swift") {
                    "swift"
                } else if extension.is_some_and(|ext| ext == "rs") {
                    "rust"
                } else {
                    "ir"
                }
                .to_string()
            });

            let document = compile_file(&input)?;
            let mut rendered = match target.as_str() {
                "swift" => nui::swift::generate(&document),
                "uikit" => nui::uikit::generate(&document),
                "rust" => nui::rust_logic::generate(&document),
                "swift-bridge" => nui::swift_bridge::generate(&document),
                "ir" | "json" => serde_json::to_string_pretty(&document)
                    .map_err(|e| format!("failed to serialize IR: {e}"))?,
                other => {
                    return Err(format!(
                        "unknown target `{other}`; expected `swift`, `uikit`, `rust`, \
                         `swift-bridge`, or `ir`"
                    ));
                }
            };
            if !rendered.ends_with('\n') {
                rendered.push('\n');
            }
            match output {
                Some(path) => {
                    fs::write(&path, rendered)
                        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                    eprintln!("wrote {}", path.display());
                }
                None => print!("{rendered}"),
            }
            Ok(())
        }
        "check" => {
            let input = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| format!("missing input file\n\n{USAGE}"))?;
            compile_file(&input)?;
            eprintln!("{}: ok", input.display());
            Ok(())
        }
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
}

fn compile_file(path: &Path) -> Result<nui::ir::Document, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    nui::compile(&source)
        .map_err(|e| format!("{}:{}:{}: {}", path.display(), e.line, e.col, e.message))
}
