//! Primitive drawing commands the tools are built on (not in menus).

use std::sync::Arc;

use pixelcraft_color::{ColorMode, Rgba8};
use pixelcraft_geom::Point;
use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_i32, opt_str, opt_u32};
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "pixel.stroke",
        "Pencil Stroke",
        [],
        None,
        "{x0, y0, x1?, y1?: canvas pixels; color?: \"#rrggbb[aa]\" (rgba/grayscale) = black; index?: 0–255 (indexed) = 1}",
        has_doc,
        stroke
    )]
}

fn stroke(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "pixel.stroke";
    let x0 = opt_i32(ID, p, "x0")?.ok_or_else(|| bad(ID, "`x0` is required"))?;
    let y0 = opt_i32(ID, p, "y0")?.ok_or_else(|| bad(ID, "`y0` is required"))?;
    let x1 = opt_i32(ID, p, "x1")?.unwrap_or(x0);
    let y1 = opt_i32(ID, p, "y1")?.unwrap_or(y0);
    let color = match opt_str(ID, p, "color")? {
        Some(h) => Rgba8::from_hex(h).ok_or_else(|| bad(ID, format!("`{h}` is not a colour")))?,
        None => Rgba8::BLACK,
    };
    let index = opt_u32(ID, p, "index")?.unwrap_or(1);
    let index = u8::try_from(index).map_err(|_| bad(ID, "`index` must be 0–255"))?;
    let d = doc_mut(s)?;
    let (layer, frame) = (d.layer, d.frame);
    let sp = Arc::make_mut(&mut d.sprite);
    let px: Vec<u8> = match sp.mode {
        ColorMode::Rgba => vec![color.r, color.g, color.b, color.a],
        ColorMode::Grayscale => {
            let v = (u32::from(color.r) * 30 + u32::from(color.g) * 59 + u32::from(color.b) * 11) / 100;
            vec![v as u8, color.a]
        }
        ColorMode::Indexed => vec![index],
    };
    let cel = sp.cel_mut(layer, frame).ok_or_else(|| bad(ID, "the current layer has no cel in this frame"))?;
    let origin = cel.position;
    let local = |x: i32, y: i32| Point::new(x.saturating_sub(origin.x), y.saturating_sub(origin.y));
    pixelcraft_paint::stroke_line(Arc::make_mut(&mut cel.image), local(x0, y0), local(x1, y1), &px);
    Ok(json!({}))
}
