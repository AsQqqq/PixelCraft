//! The command registry. Ids follow Aseprite's menus (`file.new`, `edit.undo`,
//! `sprite.canvasSize`, `layer.new`, `frame.new`) plus primitive editing commands
//! (`draw.stroke`, `draw.shape`, `select.shape`, `timeline.setCel`). Command *names* come from the target app; everything
//! else (code, wording of help text) is our own.

pub mod draw;
pub mod edit;
pub mod file;
pub mod frame;
pub mod layer;
pub mod pixel;
pub mod select;
pub mod sprite;
pub mod timeline;

use std::sync::OnceLock;

use serde::Serialize;
use serde_json::Value as Json;

use crate::{Document, EngineError, Result, Session};

pub type Run = fn(&mut Session, &Json) -> Result<Json>;
pub type Enabled = fn(&Session) -> std::result::Result<(), String>;

pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    /// Menu placement, e.g. `["Layer", "New"]`. Empty = not in menus.
    pub menu: &'static [&'static str],
    pub shortcut: Option<&'static str>,
    pub params: &'static str,
    pub enabled: Enabled,
    pub run: Run,
    pub undoable: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub menu: Vec<&'static str>,
    pub shortcut: Option<&'static str>,
    pub params: &'static str,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
}

impl CommandSpec {
    pub fn info(&self, s: &Session) -> CommandInfo {
        let e = (self.enabled)(s);
        CommandInfo {
            id: self.id,
            label: self.label,
            menu: self.menu.to_vec(),
            shortcut: self.shortcut,
            params: self.params,
            enabled: e.is_ok(),
            disabled_reason: e.err(),
        }
    }
}

pub fn always(_: &Session) -> std::result::Result<(), String> {
    Ok(())
}
pub fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no sprite open".into())
}

macro_rules! cmd {
    ($id:literal, $label:literal, [$($m:literal),*], $sc:expr, $params:literal, $en:expr, $run:expr) => {
        $crate::cmd::CommandSpec { id: $id, label: $label, menu: &[$($m),*], shortcut: $sc, params: $params, enabled: $en, run: $run, undoable: true }
    };
    (noundo $id:literal, $label:literal, [$($m:literal),*], $sc:expr, $params:literal, $en:expr, $run:expr) => {
        $crate::cmd::CommandSpec { id: $id, label: $label, menu: &[$($m),*], shortcut: $sc, params: $params, enabled: $en, run: $run, undoable: false }
    };
}
pub(crate) use cmd;

/// Every command, in menu order. Each area module contributes its own `specs()`.
pub fn command_specs() -> &'static [CommandSpec] {
    static ALL: OnceLock<Vec<CommandSpec>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut v = Vec::new();
        v.extend(file::specs());
        v.extend(edit::specs());
        v.extend(sprite::specs());
        v.extend(layer::specs());
        v.extend(frame::specs());
        v.extend(timeline::specs());
        v.extend(select::specs());
        v.extend(draw::specs());
        v.extend(pixel::specs());
        v
    })
}

pub fn find_command(id: &str) -> Option<&'static CommandSpec> {
    command_specs().iter().find(|c| c.id == id)
}

// --- param helpers -------------------------------------------------------------------------------

pub(crate) fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: cmd.to_string(), msg: msg.into() }
}

/// An optional unsigned integer param; wrong types are an error, not a silent default.
pub(crate) fn opt_u32(cmd: &str, p: &Json, key: &str) -> Result<Option<u32>> {
    match p.get(key) {
        None | Some(Json::Null) => Ok(None),
        Some(v) => v.as_u64().and_then(|n| u32::try_from(n).ok()).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be a whole number"))),
    }
}

pub(crate) fn opt_i32(cmd: &str, p: &Json, key: &str) -> Result<Option<i32>> {
    match p.get(key) {
        None | Some(Json::Null) => Ok(None),
        Some(v) => v.as_i64().and_then(|n| i32::try_from(n).ok()).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be a whole number"))),
    }
}

pub(crate) fn opt_str<'a>(cmd: &str, p: &'a Json, key: &str) -> Result<Option<&'a str>> {
    match p.get(key) {
        None | Some(Json::Null) => Ok(None),
        Some(v) => v.as_str().map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be a string"))),
    }
}

/// A list of `[x, y]` canvas points (missing = empty); anything else is an error. Capped at
/// 2²⁰ points.
pub(crate) fn points(cmd: &str, p: &Json, key: &str) -> Result<Vec<pixelcraft_geom::Point>> {
    let Some(v) = p.get(key) else { return Ok(Vec::new()) };
    let arr = v.as_array().ok_or_else(|| bad(cmd, format!("`{key}` must be a list of [x, y] points")))?;
    if arr.len() > 1 << 20 {
        return Err(bad(cmd, format!("`{key}` has too many points")));
    }
    arr.iter()
        .map(|pt| {
            let xy = pt.as_array().filter(|a| a.len() == 2);
            let n = |i: usize| xy.and_then(|a| a.get(i)).and_then(Json::as_i64).and_then(|n| i32::try_from(n).ok());
            match (n(0), n(1)) {
                (Some(x), Some(y)) => Ok(pixelcraft_geom::Point::new(x, y)),
                _ => Err(bad(cmd, format!("`{key}` must be a list of [x, y] whole-number points"))),
            }
        })
        .collect()
}

pub(crate) fn doc_mut(s: &mut Session) -> Result<&mut Document> {
    s.active_mut().ok_or(EngineError::NoDocument)
}
