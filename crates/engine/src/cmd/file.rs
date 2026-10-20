//! File menu: new, open, save, export.

use std::sync::Arc;

use pixelcraft_color::{ColorMode, Rgba8};
use pixelcraft_doc::Sprite;
use pixelcraft_geom::Size;
use serde_json::{Value as Json, json};

use super::{CommandSpec, always, bad, cmd, has_doc, opt_str, opt_u32};
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "file.new", "New...", ["File"], Some("Ctrl+N"), "{width?: 1–65535 = 32, height?: 1–65535 = 32, colorMode?: \"rgba\"|\"grayscale\"|\"indexed\" = \"rgba\", background?: \"transparent\"|\"white\"|\"black\" = \"transparent\"}", always, new_sprite),
        cmd!(
            noundo "file.open",
            "Open...",
            ["File"],
            Some("Ctrl+O"),
            "{path: string} — opens an image (PNG, JPEG, GIF, BMP, WebP, TGA, ICO, TIFF, PNM, QOI) as a new sprite; a file that is already open becomes the active tab",
            always,
            open
        ),
        cmd!(noundo "file.close", "Close", ["File"], Some("Ctrl+W"), "{document?: index = active}", has_doc, close),
    ]
}

fn new_sprite(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "file.new";
    let w = opt_u32(ID, p, "width")?.unwrap_or(32);
    let h = opt_u32(ID, p, "height")?.unwrap_or(32);
    let mode = match opt_str(ID, p, "colorMode")?.unwrap_or("rgba") {
        "rgba" => ColorMode::Rgba,
        "grayscale" => ColorMode::Grayscale,
        "indexed" => ColorMode::Indexed,
        other => return Err(bad(ID, format!("unknown colour mode `{other}`"))),
    };
    let fill = match opt_str(ID, p, "background")?.unwrap_or("transparent") {
        "transparent" => None,
        "white" => Some(Rgba8::WHITE),
        "black" => Some(Rgba8::BLACK),
        other => return Err(bad(ID, format!("unknown background `{other}`"))),
    };
    let mut sprite = Sprite::new(Size::new(w, h), mode).map_err(|e| bad(ID, e.to_string()))?;
    // A white or black background makes the only layer an opaque Background layer.
    if let Some(c) = fill {
        let px: Vec<u8> = match mode {
            ColorMode::Rgba => vec![c.r, c.g, c.b, 255],
            ColorMode::Grayscale => vec![c.r, 255],
            ColorMode::Indexed => vec![sprite.palette.colors.iter().position(|&p| p == c).and_then(|i| u8::try_from(i).ok()).unwrap_or(0)],
        };
        if let Some(cel) = sprite.cel_mut(0, 0) {
            Arc::make_mut(&mut cel.image).fill(&px);
        }
        if let Some(layer) = sprite.layers.first_mut() {
            layer.name = "Background".into();
            layer.background = true;
        }
    }
    let name = s.next_sprite_name();
    let index = s.add_document(name.clone(), sprite);
    Ok(json!({ "document": index, "name": name }))
}

fn open(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "file.open";
    let path = opt_str(ID, p, "path")?.ok_or_else(|| bad(ID, "`path` is required"))?.to_string();
    if let Some(i) = s.docs.iter().position(|d| d.path.as_deref() == Some(path.as_str())) {
        s.active = Some(i);
        return Ok(json!({ "document": i, "name": s.docs.get(i).map(|d| d.name.clone()), "alreadyOpen": true }));
    }
    let bytes = std::fs::read(&path).map_err(|e| bad(ID, format!("can't read {path}: {e}")))?;
    let name = std::path::Path::new(&path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
    let i = s.open_image(&name, Some(path), &bytes).map_err(|e| bad(ID, e.to_string()))?;
    let d = s.docs.get(i).ok_or_else(|| bad(ID, "the document went away"))?;
    Ok(json!({ "document": i, "name": d.name, "width": d.sprite.size.w, "height": d.sprite.size.h, "frames": d.sprite.frames.len() }))
}

/// Closes a document without asking (the UI asks about unsaved changes before calling this).
fn close(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "file.close";
    let i = match opt_u32(ID, p, "document")? {
        Some(i) => i as usize,
        None => s.active.unwrap_or(0),
    };
    if i >= s.docs.len() {
        return Err(bad(ID, format!("there is no document {i}")));
    }
    s.docs.remove(i);
    s.active = if s.docs.is_empty() { None } else { Some(s.active.unwrap_or(0).min(s.docs.len() - 1)) };
    Ok(json!({ "closed": i, "active": s.active }))
}
