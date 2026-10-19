//! `cargo xtask layers`: a crate may depend only on crates in lower layers (or earlier in the same
//! layer's chain), and nothing below L6 may use a UI toolkit.

use std::path::Path;

/// The layering table (AGENTS.md "Layering", docs/architecture.md). `None` = exempt.
pub const TABLE: &[(&str, Option<u8>)] = &[
    ("pixelcraft-geom", Some(0)),
    ("pixelcraft-color", Some(0)),
    ("pixelcraft-raster", Some(0)),
    ("pixelcraft-ase", Some(0)),
    ("pixelcraft-doc", Some(1)),
    ("pixelcraft-paint", Some(2)),
    ("pixelcraft-compose", Some(3)),
    ("pixelcraft-io", Some(4)),
    ("pixelcraft-engine", Some(5)),
    ("pixelcraft-ui-egui", Some(6)),
    ("pixelcraft-mcp", Some(6)),
    ("pixelcraft", Some(7)),
    ("pixelcraft-cli", Some(7)),
    ("pixelcraft-web", Some(7)),
    ("xtask", None),
];

/// Orderings inside L0: an earlier crate may be used by a later one, never the reverse.
/// `pixelcraft-ase` is standalone and uses nothing from the workspace.
const L0_CHAIN: &[&str] = &["pixelcraft-geom", "pixelcraft-color", "pixelcraft-raster"];

const UI_TOOLKITS: &[&str] = &["egui", "eframe", "winit", "rfd"];

fn layer_of(name: &str) -> Option<Option<u8>> {
    TABLE.iter().find(|(n, _)| *n == name).map(|(_, l)| *l)
}

/// Package name and dependency names (`[dependencies]`-style sections only) of a manifest.
fn read_manifest(text: &str) -> (Option<String>, Vec<String>) {
    let (mut name, mut deps, mut section) = (None, Vec::new(), String::new());
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            section = line.trim_matches(['[', ']']).to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim();
        if section == "package" && key == "name" {
            name = Some(value.trim().trim_matches('"').to_string());
        } else if section.ends_with("dependencies") && !section.ends_with("dev-dependencies") && !key.starts_with('#') {
            deps.push(key.to_string());
        }
    }
    (name, deps)
}

pub fn run(root: &Path) -> Result<(), String> {
    let mut problems = Vec::new();
    for dir in ["crates", "apps", "xtask"] {
        let base = root.join(dir);
        let manifests: Vec<_> = if dir == "xtask" {
            vec![base.join("Cargo.toml")]
        } else {
            std::fs::read_dir(&base)
                .map_err(|e| format!("{}: {e}", base.display()))?
                .filter_map(|e| e.ok())
                .map(|e| e.path().join("Cargo.toml"))
                .collect()
        };
        for m in manifests.into_iter().filter(|m| m.exists()) {
            let text = std::fs::read_to_string(&m).map_err(|e| format!("{}: {e}", m.display()))?;
            let (Some(name), deps) = read_manifest(&text) else { continue };
            let Some(layer) = layer_of(&name) else {
                problems.push(format!("{name}: not in the layering table (xtask/src/layers.rs)"));
                continue;
            };
            let Some(layer) = layer else { continue };
            for dep in &deps {
                if layer < 6 && UI_TOOLKITS.contains(&dep.as_str()) {
                    problems.push(format!("{name} (L{layer}) uses the UI toolkit `{dep}`; only L6 and above may"));
                }
                let Some(Some(dl)) = layer_of(dep) else { continue };
                let same_layer_ok = dl == layer && layer == 0 && {
                    let pos = |n: &str| L0_CHAIN.iter().position(|c| *c == n);
                    matches!((pos(dep), pos(&name)), (Some(a), Some(b)) if a < b)
                };
                if dl > layer || (dl == layer && !same_layer_ok) {
                    problems.push(format!("{name} (L{layer}) must not depend on {dep} (L{dl})"));
                }
            }
        }
    }
    if problems.is_empty() {
        println!("layers: ok");
        Ok(())
    } else {
        Err(format!("layering violations:\n  {}", problems.join("\n  ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_names_and_runtime_deps_only() {
        let (name, deps) = read_manifest("[package]\nname = \"pixelcraft-doc\"\n[dependencies]\nserde = 1\n[dev-dependencies]\nfoo = 1\n");
        assert_eq!(name.as_deref(), Some("pixelcraft-doc"));
        assert_eq!(deps, vec!["serde"]);
    }

    #[test]
    fn the_workspace_is_layered() {
        run(&crate::root()).unwrap();
    }
}
