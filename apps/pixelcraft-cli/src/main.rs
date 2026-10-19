//! PixelCraft without a window.
//!
//! ```text
//! pixelcraft-cli commands [--search <text>]     every command: id, label, menu, shortcut, params
//! pixelcraft-cli run --cmd 'id=<json>' ...      run commands on a new sprite, print each result
//! ```
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::process::ExitCode;

use pixelcraft_engine::{Session, command_specs};
use serde_json::{Value as Json, json};

const USAGE: &str = "\
usage: pixelcraft-cli <command>

commands:
  commands [--search TEXT]      list every command (id, label, menu, shortcut, params)
  run --cmd 'ID[=JSON]' ...     run commands on a new 32×32 sprite and print each result
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("commands") => list(args.iter().position(|a| a == "--search").and_then(|i| args.get(i + 1)).map(String::as_str)),
        Some("run") => run(&args[1..]),
        Some("-h" | "--help" | "help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn list(search: Option<&str>) -> Result<(), String> {
    let s = Session::new();
    let needle = search.map(str::to_lowercase);
    for c in command_specs() {
        if needle.as_ref().is_some_and(|n| !c.id.to_lowercase().contains(n) && !c.label.to_lowercase().contains(n)) {
            continue;
        }
        let i = c.info(&s);
        println!("{:<22} {:<18} {:<14} {}", i.id, i.label, i.shortcut.unwrap_or(""), i.params);
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).map_err(|e| e.to_string())?;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a != "--cmd" {
            return Err(format!("unexpected argument `{a}`"));
        }
        let spec = it.next().ok_or("--cmd needs a value")?;
        let (id, params) = match spec.split_once('=') {
            Some((id, p)) => (id, serde_json::from_str::<Json>(p).map_err(|e| format!("{id}: params are not JSON: {e}"))?),
            None => (spec.as_str(), json!({})),
        };
        let out = s.execute(id, &params).map_err(|e| e.to_string())?;
        println!("{id}: {out}");
    }
    Ok(())
}
