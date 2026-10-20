//! `cargo xtask icons` (with `--features icons`): every app icon from the master
//! `assets/app-icon/pixelcraft.svg`, as the crafting apps do.
//!
//! The artwork is a full-bleed 512-unit tile (rounded square, rx=112). macOS renders pad it onto
//! Apple's 824/1024 icon grid (transparent margin); Windows and Linux renders crop 22 units off
//! each side (into the rounded corners) so the figure reads at 16–48 px.
//!
//! Writes: `pixelcraft-1024.png` (macOS), `hicolor/<n>x<n>/apps/ai.storyteller.pixelcraft.png`
//! (Linux, n = 16…512), `hicolor/scalable/apps/ai.storyteller.pixelcraft.svg`, `pixelcraft.ico`
//! (Windows, PNG entries 16…256) and, on macOS, `pixelcraft.icns` via `iconutil`.

use std::path::Path;
use std::process::Command;

const APP_ID: &str = "ai.storyteller.pixelcraft";
const VIEWBOX: &str = r#"viewBox="0 0 512 512""#;
const MAC_VIEWBOX: &str = r#"viewBox="-62 -62 636 636""#;
const TIGHT_VIEWBOX: &str = r#"viewBox="22 22 468 468""#;

/// Renders `svg` (text) to a square PNG of `size` pixels.
fn render(svg: &str, size: u32) -> Result<Vec<u8>, String> {
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(svg, &opt).map_err(|e| format!("bad SVG: {e}"))?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).ok_or("bad icon size")?;
    let s = tree.size();
    let scale = size as f32 / s.width().max(s.height());
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| format!("PNG: {e}"))
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// A Windows .ico holding PNG-compressed images (Vista and later read these at every size).
pub fn ico(images: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    out.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len() as u32;
    for (size, png) in images {
        let side = if *size >= 256 { 0 } else { *size as u8 }; // 0 means 256
        out.extend_from_slice(&[side, side, 0, 0]); // width, height, palette, reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // colour planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in images {
        out.extend_from_slice(png);
    }
    out
}

pub fn run(root: &Path) -> Result<(), String> {
    let dir = root.join("assets/app-icon");
    let master = dir.join("pixelcraft.svg");
    let svg = std::fs::read_to_string(&master).map_err(|e| format!("{}: {e}", master.display()))?;
    if !svg.contains(VIEWBOX) {
        return Err(format!("expected {VIEWBOX} in {}", master.display()));
    }
    let mac = svg.replacen(VIEWBOX, MAC_VIEWBOX, 1);
    let tight = svg.replacen(VIEWBOX, TIGHT_VIEWBOX, 1);

    write(&dir.join("pixelcraft-1024.png"), &render(&mac, 1024)?)?;
    for s in [16, 24, 32, 48, 64, 128, 256, 512] {
        write(&dir.join(format!("hicolor/{s}x{s}/apps/{APP_ID}.png")), &render(&tight, s)?)?;
    }
    write(&dir.join(format!("hicolor/scalable/apps/{APP_ID}.svg")), svg.as_bytes())?;

    let mut entries = Vec::new();
    for s in [16, 20, 24, 32, 40, 48, 64, 128, 256] {
        entries.push((s, render(&tight, s)?));
    }
    write(&dir.join("pixelcraft.ico"), &ico(&entries))?;

    // macOS .icns through the system's iconutil (absent elsewhere: keep the committed file).
    let set = std::env::temp_dir().join("pixelcraft.iconset");
    let _ = std::fs::remove_dir_all(&set);
    for s in [16u32, 32, 128, 256, 512] {
        write(&set.join(format!("icon_{s}x{s}.png")), &render(&mac, s)?)?;
        write(&set.join(format!("icon_{s}x{s}@2x.png")), &render(&mac, s * 2)?)?;
    }
    let icns = dir.join("pixelcraft.icns");
    match Command::new("iconutil").arg("-c").arg("icns").arg("-o").arg(&icns).arg(&set).status() {
        Ok(st) if st.success() => {}
        Ok(st) => return Err(format!("iconutil failed: {st}")),
        Err(_) => eprintln!("warning: iconutil not found (macOS only); pixelcraft.icns not regenerated"),
    }
    let _ = std::fs::remove_dir_all(&set);
    println!("icons written to {}", dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ico_directory_points_at_each_image() {
        let ico = ico(&[(16, vec![1, 2, 3]), (256, vec![4, 5])]);
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 2, 0]);
        assert_eq!(ico[6], 16);
        assert_eq!(ico[22], 0, "256 px is stored as 0");
        let off = u32::from_le_bytes([ico[18], ico[19], ico[20], ico[21]]) as usize;
        assert_eq!(&ico[off..off + 3], &[1, 2, 3]);
    }
}
