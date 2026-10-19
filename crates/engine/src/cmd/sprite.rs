//! Sprite menu: properties, colour mode, canvas size (to come). For now the read-only summary.

use serde_json::{Value as Json, json};

use super::{CommandSpec, cmd, has_doc};
use crate::{EngineError, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(noundo "sprite.properties", "Properties...", ["Sprite"], Some("Ctrl+P"), "{}", has_doc, properties)]
}

fn properties(s: &mut Session, _: &Json) -> Result<Json> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let sp = &d.sprite;
    Ok(json!({
        "width": sp.size.w,
        "height": sp.size.h,
        "colorMode": sp.mode,
        "layers": sp.layers.len(),
        "frames": sp.frames.len(),
        "paletteSize": sp.palette.colors.len(),
    }))
}
