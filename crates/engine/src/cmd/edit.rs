//! Edit menu: undo, redo and delete (cut, copy, paste and transforms follow).

use std::sync::Arc;

use pixelcraft_geom::Point;
use serde_json::{Value as Json, json};

use super::draw::pixel_param;
use super::{CommandSpec, bad, cmd, doc_mut, has_doc};
use crate::{HistoryEntry, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "edit.undo", "Undo", ["Edit"], Some("Ctrl+Z"), "{}", can_undo, undo),
        cmd!(noundo "edit.redo", "Redo", ["Edit"], Some("Ctrl+Y"), "{}", can_redo, redo),
        cmd!(
            "edit.clear",
            "Delete",
            ["Edit"],
            Some("Delete"),
            "{color?: \"#rrggbb\" = black: what a background layer is cleared to} — clears the selected pixels of the current cel to transparent, or the whole cel when nothing is selected (the UI also binds Backspace)",
            has_doc,
            clear
        ),
    ]
}

fn can_undo(s: &Session) -> std::result::Result<(), String> {
    if s.active().is_some_and(|d| !d.undo.is_empty()) { Ok(()) } else { Err("nothing to undo".into()) }
}
fn can_redo(s: &Session) -> std::result::Result<(), String> {
    if s.active().is_some_and(|d| !d.redo.is_empty()) { Ok(()) } else { Err("nothing to redo".into()) }
}

fn undo(s: &mut Session, _: &Json) -> Result<Json> {
    let d = doc_mut(s)?;
    let Some(e) = d.undo.pop() else { return Ok(json!({ "undone": null })) };
    let label = e.label.clone();
    d.redo.push(HistoryEntry { label: e.label, sprite: std::mem::replace(&mut d.sprite, e.sprite) });
    Ok(json!({ "undone": label }))
}

fn redo(s: &mut Session, _: &Json) -> Result<Json> {
    let d = doc_mut(s)?;
    let Some(e) = d.redo.pop() else { return Ok(json!({ "redone": null })) };
    let label = e.label.clone();
    d.undo.push(HistoryEntry { label: e.label, sprite: std::mem::replace(&mut d.sprite, e.sprite) });
    Ok(json!({ "redone": label }))
}

/// Edit ▸ Delete: the selected pixels of the current cel become transparent. Without a selection
/// the cel goes. A background layer has no transparency, so it is painted with `color` (the UI
/// passes the background colour). The selection stays.
fn clear(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "edit.clear";
    let d = doc_mut(s)?;
    let (layer, frame) = (d.layer, d.frame);
    let l = d.sprite.layers.get(layer).ok_or_else(|| bad(ID, "there is no current layer"))?;
    if l.locked {
        return Err(bad(ID, "the layer is locked"));
    }
    let background = l.background;
    let px = if background { pixel_param(ID, p, d.sprite.mode, &d.sprite.palette)? } else { vec![0; d.sprite.mode.bytes_per_pixel()] };
    // Nothing to clear: leave the sprite untouched, so no undo step is recorded.
    if d.sprite.cel(layer, frame).is_none() {
        return Ok(json!({ "cleared": 0 }));
    }
    let selection = d.selection.clone();
    if selection.is_none() && !background {
        Arc::make_mut(&mut d.sprite).cels.retain(|c| !(c.layer == layer && c.frame == frame));
        return Ok(json!({ "cleared": "cel" }));
    }
    let sp = Arc::make_mut(&mut d.sprite);
    let cel = sp.cel_mut(layer, frame).ok_or_else(|| bad(ID, "the current layer has no cel in this frame"))?;
    let origin = cel.position;
    let img = Arc::make_mut(&mut cel.image);
    let size = img.size();
    let mut n = 0usize;
    for y in 0..size.h as i32 {
        for x in 0..size.w as i32 {
            let canvas = Point::new(x.saturating_add(origin.x), y.saturating_add(origin.y));
            if selection.as_ref().is_none_or(|m| m.get(canvas)) {
                img.put_pixel(Point::new(x, y), &px);
                n += 1;
            }
        }
    }
    Ok(json!({ "cleared": n }))
}
