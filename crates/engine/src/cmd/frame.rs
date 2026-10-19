//! Frame menu: new frame (duplicates the current frame's cels), durations, tags (to come).

use std::sync::Arc;

use pixelcraft_doc::MAX_FRAMES;
use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc};
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!("frame.new", "New Frame", ["Frame"], Some("Alt+N"), "{}", has_doc, new_frame)]
}

fn new_frame(s: &mut Session, _: &Json) -> Result<Json> {
    const ID: &str = "frame.new";
    let d = doc_mut(s)?;
    if d.sprite.frames.len() >= MAX_FRAMES {
        return Err(bad(ID, format!("a sprite can have at most {MAX_FRAMES} frames")));
    }
    let cur = d.frame.min(d.sprite.frames.len().saturating_sub(1));
    let sp = Arc::make_mut(&mut d.sprite);
    let at = cur + 1;
    let dur = sp.frames.get(cur).cloned().unwrap_or_default();
    sp.frames.insert(at, dur);
    for c in &mut sp.cels {
        if c.frame >= at {
            c.frame += 1;
        }
    }
    // The new frame starts as a copy of the current one; images are shared until edited.
    let copies: Vec<_> = sp
        .cels
        .iter()
        .filter(|c| c.frame == cur)
        .cloned()
        .map(|mut c| {
            c.frame = at;
            c
        })
        .collect();
    sp.cels.extend(copies);
    d.frame = at;
    Ok(json!({ "frame": at }))
}
