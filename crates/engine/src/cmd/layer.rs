//! Layer menu: new layer, visibility (properties, groups, blending follow).

use std::sync::Arc;

use pixelcraft_doc::{Layer, MAX_LAYERS};
use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_str, opt_u32};
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!("layer.new", "New Layer", ["Layer", "New"], Some("Shift+N"), "{name?: string}", has_doc, new_layer),
        cmd!("layer.toggleVisible", "Show/Hide Layer", [], None, "{layer?: index = active}", has_doc, toggle_visible),
    ]
}

fn new_layer(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "layer.new";
    let name = opt_str(ID, p, "name")?.map(str::to_string);
    let d = doc_mut(s)?;
    if d.sprite.layers.len() >= MAX_LAYERS {
        return Err(bad(ID, format!("a sprite can have at most {MAX_LAYERS} layers")));
    }
    let sp = Arc::make_mut(&mut d.sprite);
    let name = name.unwrap_or_else(|| format!("Layer {}", sp.layers.len() + 1));
    // New layers go above the active one.
    let at = (d.layer + 1).min(sp.layers.len());
    sp.layers.insert(at, Layer::new(name));
    for c in &mut sp.cels {
        if c.layer >= at {
            c.layer += 1;
        }
    }
    d.layer = at;
    Ok(json!({ "layer": at }))
}

fn toggle_visible(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "layer.toggleVisible";
    let want = opt_u32(ID, p, "layer")?;
    let d = doc_mut(s)?;
    let i = want.map_or(d.layer, |n| n as usize);
    let layer = Arc::make_mut(&mut d.sprite).layers.get_mut(i).ok_or_else(|| bad(ID, format!("there is no layer {i}")))?;
    layer.visible = !layer.visible;
    Ok(json!({ "layer": i, "visible": layer.visible }))
}
