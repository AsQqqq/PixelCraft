//! Drawing primitives the tools are built on (not in menus): one command per stroke or shape,
//! so each is a single undo step. Painting stays inside the selection when there is one. Colour
//! params everywhere: `color` ("#rrggbb[aa]", RGBA/grayscale), `index` (indexed), `erase`.

use std::sync::Arc;

use pixelcraft_color::{ColorMode, Rgba8};
use pixelcraft_geom::{Point, Rect};
use pixelcraft_raster::mask::magic_wand;
use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_i32, opt_str, opt_u32, points};
use crate::{Document, EngineError, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "draw.stroke",
            "Brush Stroke",
            [],
            None,
            "{points: [[x,y]…] canvas pixels; size?: 1–64 = 1; pixelPerfect?: bool; connect?: bool = true (false: dots, for the spray); color/index/erase}",
            has_doc,
            stroke
        ),
        cmd!(
            "draw.shape",
            "Draw Shape",
            [],
            None,
            "{shape: \"line\"|\"rect\"|\"ellipse\"|\"polygon\", x0, y0, x1, y1 (corners/ends) or points (polygon); filled?: bool; size?: 1–64 (line) = 1; color/index/erase}",
            has_doc,
            shape
        ),
        cmd!("draw.fill", "Paint Bucket", [], None, "{x, y; tolerance?: 0–255 = 0; contiguous?: bool = true; color/index}", has_doc, fill),
        cmd!(
            "cel.moveContent",
            "Move",
            [],
            None,
            "{dx, dy: pixels} — moves the selected pixels (and the selection), or the whole cel when nothing is selected",
            has_doc,
            move_content
        ),
    ]
}

/// The raw pixel to paint, in the sprite's colour mode, from `color` / `index` / `erase`.
pub(crate) fn pixel_param(id: &str, p: &Json, mode: ColorMode, palette: &pixelcraft_color::Palette) -> Result<Vec<u8>> {
    let erase = p.get("erase").and_then(Json::as_bool).unwrap_or(false);
    let color = match opt_str(id, p, "color")? {
        Some(h) => Rgba8::from_hex(h).ok_or_else(|| bad(id, format!("`{h}` is not a colour")))?,
        None => Rgba8::BLACK,
    };
    let index = match opt_u32(id, p, "index")? {
        Some(i) => u8::try_from(i).map_err(|_| bad(id, "`index` must be 0–255"))?,
        // Indexed sprites paint the palette entry matching `color` when given.
        None => palette.colors.iter().position(|&c| c == color).and_then(|i| u8::try_from(i).ok()).unwrap_or(1),
    };
    Ok(match (mode, erase) {
        (ColorMode::Rgba, true) => vec![0, 0, 0, 0],
        (ColorMode::Grayscale, true) => vec![0, 0],
        (ColorMode::Indexed, true) => vec![0],
        (ColorMode::Rgba, false) => vec![color.r, color.g, color.b, color.a],
        (ColorMode::Grayscale, false) => {
            let v = (u32::from(color.r) * 30 + u32::from(color.g) * 59 + u32::from(color.b) * 11) / 100;
            vec![v as u8, color.a]
        }
        (ColorMode::Indexed, false) => vec![index],
    })
}

/// Paints `pixels` on the active cel (inside the selection, if any).
fn paint(id: &str, d: &mut Document, pixels: &[Point], px: &[u8]) -> Result<usize> {
    let (layer, frame) = (d.layer, d.frame);
    if d.sprite.layers.get(layer).is_some_and(|l| l.locked) {
        return Err(bad(id, "the layer is locked"));
    }
    let selection = d.selection.clone();
    let sp = Arc::make_mut(&mut d.sprite);
    let cel = sp.cel_or_new(layer, frame).map_err(|e| bad(id, e.to_string()))?;
    let origin = cel.position;
    Ok(pixelcraft_paint::apply(Arc::make_mut(&mut cel.image), origin, pixels, px, selection.as_ref()))
}

fn stroke(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "draw.stroke";
    let path = points(ID, p, "points")?;
    if path.is_empty() {
        return Err(bad(ID, "`points` needs at least one point"));
    }
    let size = opt_u32(ID, p, "size")?.unwrap_or(1).clamp(1, pixelcraft_paint::MAX_BRUSH);
    let perfect = p.get("pixelPerfect").and_then(Json::as_bool).unwrap_or(false);
    let d = doc_mut(s)?;
    let px = pixel_param(ID, p, d.sprite.mode, &d.sprite.palette)?;
    let connect = p.get("connect").and_then(Json::as_bool).unwrap_or(true);
    let pixels = if connect { pixelcraft_paint::stroke(&path, size, perfect) } else { pixelcraft_paint::stamp(&path, size) };
    let n = paint(ID, d, &pixels, &px)?;
    Ok(json!({ "pixels": n }))
}

fn shape(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "draw.shape";
    let kind = opt_str(ID, p, "shape")?.ok_or_else(|| bad(ID, "`shape` is required"))?.to_string();
    let filled = p.get("filled").and_then(Json::as_bool).unwrap_or(false);
    let size = opt_u32(ID, p, "size")?.unwrap_or(1).clamp(1, pixelcraft_paint::MAX_BRUSH);
    let pts = points(ID, p, "points")?;
    let corner = |k: &str| opt_i32(ID, p, k);
    let d = doc_mut(s)?;
    let px = pixel_param(ID, p, d.sprite.mode, &d.sprite.palette)?;
    let clip = Rect::from_size(d.sprite.size);
    let pixels = if kind == "polygon" {
        if pts.len() < 2 {
            return Err(bad(ID, "a polygon needs at least two points"));
        }
        pixelcraft_paint::polygon(&pts, filled, clip)
    } else {
        let a = Point::new(corner("x0")?.ok_or_else(|| bad(ID, "`x0` is required"))?, corner("y0")?.ok_or_else(|| bad(ID, "`y0` is required"))?);
        let b = Point::new(corner("x1")?.unwrap_or(a.x), corner("y1")?.unwrap_or(a.y));
        match kind.as_str() {
            "line" => pixelcraft_paint::stroke(&[a, b], size, false),
            "rect" => pixelcraft_paint::rectangle(a, b, filled),
            "ellipse" => pixelcraft_paint::ellipse(a, b, filled),
            other => return Err(bad(ID, format!("unknown shape `{other}`"))),
        }
    };
    let n = paint(ID, d, &pixels, &px)?;
    Ok(json!({ "pixels": n }))
}

fn fill(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "draw.fill";
    let x = opt_i32(ID, p, "x")?.ok_or_else(|| bad(ID, "`x` is required"))?;
    let y = opt_i32(ID, p, "y")?.ok_or_else(|| bad(ID, "`y` is required"))?;
    let tolerance = opt_u32(ID, p, "tolerance")?.unwrap_or(0).min(255) as u8;
    let contiguous = p.get("contiguous").and_then(Json::as_bool).unwrap_or(true);
    let d = doc_mut(s)?;
    let px = pixel_param(ID, p, d.sprite.mode, &d.sprite.palette)?;
    // The bucket follows the active cel's own pixels, like the target app's default.
    // An empty layer gets its cel here, so the bucket fills it like any other.
    let (layer, frame) = (d.layer, d.frame);
    Arc::make_mut(&mut d.sprite).cel_or_new(layer, frame).map_err(|e| bad(ID, e.to_string()))?;
    let cel = d.sprite.cel(layer, frame).ok_or_else(|| bad(ID, "the current layer has no cel in this frame"))?;
    let origin = cel.position;
    let seed = Point::new(x.saturating_sub(origin.x), y.saturating_sub(origin.y));
    let region = magic_wand(&cel.image, &d.sprite.palette, seed, tolerance, contiguous).map_err(|e| EngineError::Other(e.to_string()))?;
    let size = region.size();
    let mut pixels = Vec::new();
    for yy in 0..size.h as i32 {
        for xx in 0..size.w as i32 {
            if region.get(Point::new(xx, yy)) {
                pixels.push(Point::new(xx.saturating_add(origin.x), yy.saturating_add(origin.y)));
            }
        }
    }
    let n = paint(ID, d, &pixels, &px)?;
    Ok(json!({ "pixels": n }))
}

fn move_content(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "cel.moveContent";
    let dx = opt_i32(ID, p, "dx")?.unwrap_or(0);
    let dy = opt_i32(ID, p, "dy")?.unwrap_or(0);
    let d = doc_mut(s)?;
    let (layer, frame) = (d.layer, d.frame);
    if d.sprite.layers.get(layer).is_some_and(|l| l.locked || l.background) {
        return Err(bad(ID, "a locked or background layer can't move"));
    }
    let selection = d.selection.clone();
    let sp = Arc::make_mut(&mut d.sprite);
    let cel = sp.cel_or_new(layer, frame).map_err(|e| bad(ID, e.to_string()))?;
    let Some(sel) = selection else {
        cel.position = cel.position.offset(dx, dy);
        return Ok(json!({ "moved": "cel", "x": cel.position.x, "y": cel.position.y }));
    };
    // Lift the selected pixels, clear them, and put them back shifted (clipped to the cel).
    let origin = cel.position;
    let img = Arc::make_mut(&mut cel.image);
    let mode = img.mode();
    let clear = vec![0u8; mode.bytes_per_pixel()];
    let size = sel.size();
    let mut lifted = Vec::new();
    for y in 0..size.h as i32 {
        for x in 0..size.w as i32 {
            let canvas = Point::new(x, y);
            if !sel.get(canvas) {
                continue;
            }
            let local = Point::new(x.saturating_sub(origin.x), y.saturating_sub(origin.y));
            if let Some(px) = img.pixel(local).map(<[u8]>::to_vec) {
                lifted.push((local, px));
                img.put_pixel(local, &clear);
            }
        }
    }
    for (local, px) in &lifted {
        img.put_pixel(local.offset(dx, dy), px);
    }
    // The selection moves with its pixels.
    let mut moved = pixelcraft_raster::Mask::new(size).map_err(|e| EngineError::Other(e.to_string()))?;
    for y in 0..size.h as i32 {
        for x in 0..size.w as i32 {
            if sel.get(Point::new(x, y)) {
                moved.set(Point::new(x, y).offset(dx, dy), true);
            }
        }
    }
    d.selection = (!moved.is_empty()).then_some(moved);
    Ok(json!({ "moved": "selection", "pixels": lifted.len() }))
}
