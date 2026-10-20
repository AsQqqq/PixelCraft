//! Edit menu: undo, redo, cut, copy, paste and delete.

use std::sync::Arc;

use pixelcraft_color::{ColorMode, Palette, Rgba8};
use pixelcraft_compose::blend_normal;
use pixelcraft_geom::{Point, Rect, Size};
use pixelcraft_raster::{Image, Mask};
use serde_json::{Value as Json, json};

use super::draw::pixel_param;
use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_i32, opt_u32};
use crate::{Clip, Document, EngineError, HistoryEntry, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "edit.undo", "Undo", ["Edit"], Some("Ctrl+Z"), "{}", can_undo, undo),
        cmd!(noundo "edit.redo", "Redo", ["Edit"], Some("Ctrl+Y"), "{}", can_redo, redo),
        cmd!(
            "edit.cut",
            "Cut",
            ["Edit"],
            Some("Ctrl+X"),
            "{color?: \"#rrggbb\" = black: what a background layer is left as} — copies the selected pixels of the current cel (the whole canvas without a selection), then deletes them",
            has_doc,
            cut
        ),
        cmd!(
            noundo "edit.copy",
            "Copy",
            ["Edit"],
            Some("Ctrl+C"),
            "{} — copies the selected pixels of the current cel (the whole canvas without a selection)",
            has_doc,
            copy
        ),
        cmd!(
            "edit.paste",
            "Paste",
            ["Edit"],
            Some("Ctrl+V"),
            "{x?, y?: top-left of the box on the canvas = where it was copied (centred if that is off the canvas); width?, height?: 1–65535 = the copied size; angle?: degrees clockwise = 0} — draws the copied pixels, stretched and turned, onto the current cel and selects them",
            can_paste,
            paste
        ),
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

fn can_paste(s: &Session) -> std::result::Result<(), String> {
    has_doc(s)?;
    if s.clipboard.is_some() { Ok(()) } else { Err("nothing has been copied".into()) }
}

/// The selected pixels of the current cel (the whole canvas without a selection), as RGBA.
fn take_clip(id: &str, d: &Document) -> Result<Clip> {
    let canvas = Rect::from_size(d.sprite.size);
    let area = d.selection.as_ref().map_or(canvas, Mask::bounds);
    if area.w == 0 || area.h == 0 {
        return Err(bad(id, "the selection is empty"));
    }
    let cel = d.sprite.cel(d.layer, d.frame);
    let mut image = Image::new(ColorMode::Rgba, Size::new(area.w, area.h)).map_err(|e| EngineError::Other(e.to_string()))?;
    for y in 0..area.h as i32 {
        for x in 0..area.w as i32 {
            let p = Point::new(area.x.saturating_add(x), area.y.saturating_add(y));
            if d.selection.as_ref().is_some_and(|m| !m.get(p)) {
                continue;
            }
            let Some(cel) = cel else { continue };
            let local = Point::new(p.x.saturating_sub(cel.position.x), p.y.saturating_sub(cel.position.y));
            let c = cel.image.rgba(local, &d.sprite.palette);
            image.put_pixel(Point::new(x, y), &[c.r, c.g, c.b, c.a]);
        }
    }
    Ok(Clip { image, origin: Point::new(area.x, area.y) })
}

fn copy(s: &mut Session, _: &Json) -> Result<Json> {
    const ID: &str = "edit.copy";
    let d = s.active().ok_or_else(|| bad(ID, "no sprite open"))?;
    let clip = take_clip(ID, d)?;
    let out = json!({ "x": clip.origin.x, "y": clip.origin.y, "width": clip.image.size().w, "height": clip.image.size().h });
    s.clipboard = Some(clip);
    Ok(out)
}

fn cut(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "edit.cut";
    let d = s.active().ok_or_else(|| bad(ID, "no sprite open"))?;
    if d.sprite.layers.get(d.layer).is_some_and(|l| l.locked) {
        return Err(bad(ID, "the layer is locked"));
    }
    let clip = take_clip(ID, d)?;
    let out = json!({ "x": clip.origin.x, "y": clip.origin.y, "width": clip.image.size().w, "height": clip.image.size().h });
    clear(s, p)?;
    s.clipboard = Some(clip);
    Ok(out)
}

/// `c` as a raw pixel of `mode` (indexed: the nearest palette entry).
fn raw(c: Rgba8, mode: ColorMode, palette: &Palette) -> Vec<u8> {
    match mode {
        ColorMode::Rgba => vec![c.r, c.g, c.b, c.a],
        ColorMode::Grayscale => {
            let v = (u32::from(c.r) * 30 + u32::from(c.g) * 59 + u32::from(c.b) * 11) / 100;
            vec![v as u8, c.a]
        }
        ColorMode::Indexed => {
            let dist = |p: &Rgba8| {
                let d = |a: u8, b: u8| (i32::from(a) - i32::from(b)).pow(2);
                d(p.r, c.r) + d(p.g, c.g) + d(p.b, c.b)
            };
            let best = palette.colors.iter().enumerate().min_by_key(|(_, p)| dist(p)).map_or(0, |(i, _)| i);
            vec![u8::try_from(best).unwrap_or(0)]
        }
    }
}

/// Where a `w`×`h` paste of `clip` goes by default: where it was copied, unless that is off
/// a canvas of `size`; then centred.
pub(crate) fn paste_position(clip: &Clip, size: Size, w: u32, h: u32) -> Point {
    let (w, h) = (i32::try_from(w).unwrap_or(i32::MAX), i32::try_from(h).unwrap_or(i32::MAX));
    let (cw, ch) = (i32::try_from(size.w).unwrap_or(i32::MAX), i32::try_from(size.h).unwrap_or(i32::MAX));
    let o = clip.origin;
    let on_canvas = o.x < cw && o.y < ch && o.x.saturating_add(w) > 0 && o.y.saturating_add(h) > 0;
    if on_canvas { o } else { Point::new(cw.saturating_sub(w) / 2, ch.saturating_sub(h) / 2) }
}

fn paste(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "edit.paste";
    let clip = s.clipboard.clone().ok_or_else(|| bad(ID, "nothing has been copied"))?;
    let (cw, ch) = (clip.image.size().w, clip.image.size().h);
    let w = opt_u32(ID, p, "width")?.unwrap_or(cw);
    let h = opt_u32(ID, p, "height")?.unwrap_or(ch);
    let angle = p.get("angle").map_or(Ok(0.0), |v| v.as_f64().ok_or_else(|| bad(ID, "`angle` must be a number")))?;
    let d = doc_mut(s)?;
    let size = d.sprite.size;
    let default = paste_position(&clip, size, w, h);
    let x = opt_i32(ID, p, "x")?.unwrap_or(default.x);
    let y = opt_i32(ID, p, "y")?.unwrap_or(default.y);
    let (layer, frame) = (d.layer, d.frame);
    if d.sprite.layers.get(layer).is_some_and(|l| l.locked) {
        return Err(bad(ID, "the layer is locked"));
    }
    let (img, covered, offset) = pixelcraft_raster::transform::scale_rotate(&clip.image, w, h, angle as f32).map_err(|e| bad(ID, e.to_string()))?;
    let at = Point::new(x.saturating_add(offset.x), y.saturating_add(offset.y));
    let palette = Palette::clone(&d.sprite.palette);
    let mode = d.sprite.mode;
    let mut selection = Mask::new(size).map_err(|e| EngineError::Other(e.to_string()))?;
    let sp = Arc::make_mut(&mut d.sprite);
    let cel = sp.cel_or_new(layer, frame).map_err(|e| bad(ID, e.to_string()))?;
    let origin = cel.position;
    let dst = Arc::make_mut(&mut cel.image);
    let isize = img.size();
    for dy in 0..isize.h as i32 {
        for dx in 0..isize.w as i32 {
            let q = Point::new(dx, dy);
            if !covered.get(q) {
                continue;
            }
            let canvas = at.offset(dx, dy);
            selection.set(canvas, true);
            let src = img.rgba(q, &palette);
            if src.a == 0 {
                continue;
            }
            let local = Point::new(canvas.x.saturating_sub(origin.x), canvas.y.saturating_sub(origin.y));
            if dst.pixel(local).is_none() {
                continue;
            }
            let c = match mode {
                ColorMode::Indexed => src,
                _ => blend_normal(dst.rgba(local, &palette), src, 255),
            };
            dst.put_pixel(local, &raw(c, mode, &palette));
        }
    }
    d.selection = (!selection.is_empty()).then_some(selection);
    Ok(json!({ "x": at.x, "y": at.y, "width": isize.w, "height": isize.h }))
}
