//! Frame menu: new frame (duplicates the current frame's cels), delete frame; durations and tags
//! to come.

use std::sync::Arc;

use pixelcraft_doc::MAX_FRAMES;
use serde_json::{Value as Json, json};

use super::{CommandSpec, bad, cmd, doc_mut, has_doc, opt_u32};
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!("frame.new", "New Frame", ["Frame"], Some("Alt+N"), "{}", has_doc, new_frame),
        cmd!(
            "frame.remove",
            "Delete Frame",
            ["Frame"],
            Some("Alt+C"),
            "{frame?: index = current} — removes the frame and its cels; the last frame can't go",
            can_remove,
            remove_frame
        ),
    ]
}

fn can_remove(s: &Session) -> std::result::Result<(), String> {
    match s.active() {
        None => Err("no sprite open".into()),
        Some(d) if d.sprite.frames.len() < 2 => Err("a sprite needs at least one frame".into()),
        Some(_) => Ok(()),
    }
}

fn remove_frame(s: &mut Session, p: &Json) -> Result<Json> {
    const ID: &str = "frame.remove";
    let want = opt_u32(ID, p, "frame")?;
    let d = doc_mut(s)?;
    let count = d.sprite.frames.len();
    if count < 2 {
        return Err(bad(ID, "a sprite needs at least one frame"));
    }
    let at = want.map_or(d.frame, |n| n as usize);
    if at >= count {
        return Err(bad(ID, format!("there is no frame {at}")));
    }
    let sp = Arc::make_mut(&mut d.sprite);
    sp.frames.remove(at);
    sp.cels.retain(|c| c.frame != at);
    for c in &mut sp.cels {
        if c.frame > at {
            c.frame -= 1;
        }
    }
    // Tags shrink around the removed frame; a tag that only covered it goes.
    sp.tags.retain(|t| !(t.from == at && t.to == at));
    for t in &mut sp.tags {
        if t.from > at {
            t.from -= 1;
        }
        if t.to >= at && t.to > 0 {
            t.to -= 1;
        }
    }
    // The current frame stays where it was, or moves back when it was the last one.
    if d.frame > at || d.frame >= sp.frames.len() {
        d.frame = d.frame.saturating_sub(1);
    }
    Ok(json!({ "removed": at, "frames": sp.frames.len(), "frame": d.frame }))
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
