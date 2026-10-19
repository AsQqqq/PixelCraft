//! The timeline cursor: which layer and frame are active. Navigation only, nothing is undoable.

use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_u32};
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "frame.goToFirst", "First Frame", ["Frame", "Jump to"], Some("Home"), "{}", has_doc, |s, _| go(s, |_, _| 0)),
        cmd!(noundo "frame.goToPrevious", "Previous Frame", ["Frame", "Jump to"], Some("Left"), "{}", has_doc, |s, _| go(s, |f, n| (f + n - 1) % n)),
        cmd!(noundo "frame.goToNext", "Next Frame", ["Frame", "Jump to"], Some("Right"), "{}", has_doc, |s, _| go(s, |f, n| (f + 1) % n)),
        cmd!(noundo "frame.goToLast", "Last Frame", ["Frame", "Jump to"], Some("End"), "{}", has_doc, |s, _| go(s, |_, n| n - 1)),
        cmd!(noundo "layer.goToNext", "Next Layer", [], Some("Up"), "{}", has_doc, |s, _| go_layer(s, true)),
        cmd!(noundo "layer.goToPrevious", "Previous Layer", [], Some("Down"), "{}", has_doc, |s, _| go_layer(s, false)),
        cmd!(noundo "timeline.setCel", "Select Cel", [], None, "{layer?: index = active, frame?: index = active}", has_doc, set_cel),
    ]
}

/// Moves to the frame `next(current, count)` returns; frames wrap around like the target app.
fn go(s: &mut Session, next: fn(usize, usize) -> usize) -> Result<Json> {
    let d = doc_mut(s)?;
    let n = d.sprite.frames.len().max(1);
    d.frame = next(d.frame.min(n - 1), n);
    Ok(json!({ "frame": d.frame }))
}

/// "Next" is the layer above (layers[0] is the bottom); it stops at the ends.
fn go_layer(s: &mut Session, up: bool) -> Result<Json> {
    let d = doc_mut(s)?;
    let top = d.sprite.layers.len().saturating_sub(1);
    d.layer = if up { (d.layer + 1).min(top) } else { d.layer.saturating_sub(1) };
    Ok(json!({ "layer": d.layer }))
}

fn set_cel(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "timeline.setCel";
    let layer = opt_u32(ID, p, "layer")?;
    let frame = opt_u32(ID, p, "frame")?;
    let d = doc_mut(s)?;
    let layer = layer.map_or(d.layer, |l| l as usize);
    let frame = frame.map_or(d.frame, |f| f as usize);
    if layer >= d.sprite.layers.len() {
        return Err(bad(ID, format!("there is no layer {layer}")));
    }
    if frame >= d.sprite.frames.len() {
        return Err(bad(ID, format!("there is no frame {frame}")));
    }
    d.layer = layer;
    d.frame = frame;
    Ok(json!({ "layer": layer, "frame": frame }))
}
