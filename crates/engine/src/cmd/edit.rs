//! Edit menu: undo and redo (cut, copy, paste and transforms follow).

use serde_json::{Value as Json, json};

use super::{CommandSpec, cmd, doc_mut};
use crate::{HistoryEntry, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "edit.undo", "Undo", ["Edit"], Some("Ctrl+Z"), "{}", can_undo, undo),
        cmd!(noundo "edit.redo", "Redo", ["Edit"], Some("Ctrl+Y"), "{}", can_redo, redo),
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
