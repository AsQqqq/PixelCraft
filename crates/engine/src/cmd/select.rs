//! Select menu and the selection tools: all, deselect, reselect, inverse, and the shapes the
//! marquee, lasso and magic wand make. Selections live on the document, not in undo history.

use pixelcraft_geom::{Point, Rect};
use pixelcraft_raster::mask::magic_wand;
use pixelcraft_raster::{Mask, SelectMode};
use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_i32, opt_str, opt_u32, points};
use crate::{EngineError, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "select.all", "All", ["Select"], Some("Ctrl+A"), "{}", has_doc, all),
        cmd!(noundo "select.deselect", "Deselect", ["Select"], Some("Ctrl+D"), "{}", has_doc, deselect),
        cmd!(noundo "select.reselect", "Reselect", ["Select"], Some("Ctrl+Shift+D"), "{}", can_reselect, reselect),
        cmd!(noundo "select.inverse", "Inverse", ["Select"], Some("Ctrl+Shift+I"), "{}", has_doc, inverse),
        cmd!(
            noundo "select.shape",
            "Select Shape",
            [],
            None,
            "{shape: \"rect\"|\"ellipse\"|\"polygon\"|\"wand\", x0, y0, x1?, y1? (rect/ellipse corners, wand seed), points?: [[x,y]…] (polygon), mode?: \"replace\"|\"add\"|\"subtract\"|\"intersect\" = \"replace\", tolerance?: 0–255 = 0, contiguous?: bool = true}",
            has_doc,
            shape
        ),
    ]
}

fn can_reselect(s: &Session) -> std::result::Result<(), String> {
    if s.active().is_some_and(|d| d.last_selection.is_some()) { Ok(()) } else { Err("there is no previous selection".into()) }
}

fn summary(sel: &Option<Mask>) -> Json {
    match sel {
        Some(m) => {
            let b = m.bounds();
            json!({ "selected": true, "x": b.x, "y": b.y, "width": b.w, "height": b.h })
        }
        None => json!({ "selected": false }),
    }
}

fn all(s: &mut Session, _: &Json) -> Result<Json> {
    let d = doc_mut(s)?;
    d.selection = Some(Mask::all(d.sprite.size).map_err(|e| EngineError::Other(e.to_string()))?);
    Ok(summary(&d.selection))
}

fn deselect(s: &mut Session, _: &Json) -> Result<Json> {
    let d = doc_mut(s)?;
    if let Some(m) = d.selection.take() {
        d.last_selection = Some(m);
    }
    Ok(summary(&d.selection))
}

fn reselect(s: &mut Session, _: &Json) -> Result<Json> {
    let d = doc_mut(s)?;
    if let Some(m) = d.last_selection.clone() {
        d.selection = Some(m);
    }
    Ok(summary(&d.selection))
}

fn inverse(s: &mut Session, _: &Json) -> Result<Json> {
    let d = doc_mut(s)?;
    let mut m = match d.selection.take() {
        Some(m) => m,
        None => Mask::new(d.sprite.size).map_err(|e| EngineError::Other(e.to_string()))?,
    };
    m.invert();
    d.selection = (!m.is_empty()).then_some(m);
    Ok(summary(&d.selection))
}

fn shape(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "select.shape";
    let mode = match opt_str(ID, p, "mode")?.unwrap_or("replace") {
        "replace" => SelectMode::Replace,
        "add" => SelectMode::Add,
        "subtract" => SelectMode::Subtract,
        "intersect" => SelectMode::Intersect,
        other => return Err(bad(ID, format!("unknown mode `{other}`"))),
    };
    let kind = opt_str(ID, p, "shape")?.ok_or_else(|| bad(ID, "`shape` is required"))?.to_string();
    let corner = |k: &str| opt_i32(ID, p, k);
    let tolerance = opt_u32(ID, p, "tolerance")?.unwrap_or(0).min(255) as u8;
    let contiguous = p.get("contiguous").and_then(Json::as_bool).unwrap_or(true);
    let pts = points(ID, p, "points")?;
    let d = doc_mut(s)?;
    let size = d.sprite.size;
    let err = |e: pixelcraft_raster::RasterError| EngineError::Other(e.to_string());
    let made = match kind.as_str() {
        "rect" | "ellipse" => {
            let a = Point::new(corner("x0")?.ok_or_else(|| bad(ID, "`x0` is required"))?, corner("y0")?.ok_or_else(|| bad(ID, "`y0` is required"))?);
            let b = Point::new(corner("x1")?.unwrap_or(a.x), corner("y1")?.unwrap_or(a.y));
            let r: Rect = pixelcraft_paint::corners(a, b);
            if kind == "rect" { Mask::rect(size, r).map_err(err)? } else { Mask::ellipse(size, r).map_err(err)? }
        }
        "polygon" => {
            if pts.len() < 2 {
                return Err(bad(ID, "a polygon needs at least two points"));
            }
            Mask::polygon(size, &pts).map_err(err)?
        }
        "wand" => {
            let seed =
                Point::new(corner("x0")?.ok_or_else(|| bad(ID, "`x0` is required"))?, corner("y0")?.ok_or_else(|| bad(ID, "`y0` is required"))?);
            let flat = pixelcraft_compose::render_frame(&d.sprite, d.frame).map_err(err)?;
            magic_wand(&flat, &d.sprite.palette, seed, tolerance, contiguous).map_err(err)?
        }
        other => return Err(bad(ID, format!("unknown shape `{other}`"))),
    };
    let mut current = match (mode, d.selection.take()) {
        (SelectMode::Replace, _) | (_, None) => Mask::new(size).map_err(err)?,
        (_, Some(m)) => m,
    };
    current.combine(&made, if mode == SelectMode::Replace { SelectMode::Add } else { mode });
    d.selection = (!current.is_empty()).then_some(current);
    Ok(summary(&d.selection))
}
