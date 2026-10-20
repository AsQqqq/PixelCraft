//! Workspace tooling: `cargo xtask <command>` (or `cargo run -p xtask -- <command>`).
//!
//! Pure Rust (std only). External tools (`cargo`) are invoked through `std::process::Command`.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[cfg(feature = "icons")]
mod icons;
mod layers;
mod version;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  ci              fmt --check, clippy -D warnings, test, layers, wasm (stops at the first failure)
  layers          enforce the crate dependency layering (AGENTS.md \"Layering\")
  wasm            cargo check --target wasm32-unknown-unknown for the L0-L6 crates (+ pixelcraft-web)
  version [set X] print the workspace version, or set it (Cargo.toml + Cargo.lock), e.g. 0.1.2-beta
  icons           render every app icon from assets/app-icon/pixelcraft.svg
                  (cargo run -p xtask --features icons -- icons, or packaging/icons.sh)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("layers") => layers::run(&root()),
        Some("wasm") => cmd_wasm(),
        Some("version") => version::run(&root(), &args.iter().skip(1).map(String::as_str).collect::<Vec<_>>()),
        #[cfg(feature = "icons")]
        Some("icons") => icons::run(&root()),
        #[cfg(not(feature = "icons"))]
        Some("icons") => Err("build xtask with the icon renderer: cargo run -p xtask --features icons -- icons".into()),
        Some("ci") => cmd_ci(),
        Some("-h" | "--help" | "help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root (the parent of this crate's manifest directory).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn cargo(args: &[&str]) -> Result<(), String> {
    println!("$ cargo {}", args.join(" "));
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(args)
        .current_dir(root())
        .status()
        .map_err(|e| format!("couldn't run cargo: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("`cargo {}` failed", args.join(" "))) }
}

fn cmd_wasm() -> Result<(), String> {
    let mut args = vec!["check", "--target", "wasm32-unknown-unknown"];
    for (name, layer) in layers::TABLE {
        if layer.is_some_and(|l| l <= 6) {
            args.extend(["-p", name]);
        }
    }
    args.extend(["-p", "pixelcraft-web"]);
    cargo(&args)
}

fn cmd_ci() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    cargo(&["test", "--workspace"])?;
    layers::run(&root())?;
    cmd_wasm()
}
