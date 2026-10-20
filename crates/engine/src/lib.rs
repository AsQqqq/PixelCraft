//! The PixelCraft engine façade.
//!
//! Every user-visible action is a command with a stable id (`file.new`, `layer.new`,
//! `frame.new`, `pixel.stroke`…) and JSON parameters. The egui UI, the CLI, the control channel
//! and MCP all go through [`Session::execute`], so every action is scriptable and undoable.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

pub mod cmd;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value as Json;

pub use cmd::{CommandInfo, CommandSpec, command_specs, find_command};
pub use pixelcraft_color as color;
pub use pixelcraft_compose as compose;
pub use pixelcraft_doc as doc;
pub use pixelcraft_geom as geom;
pub use pixelcraft_io as io;
pub use pixelcraft_paint as paint;
pub use pixelcraft_raster as raster;

use pixelcraft_doc::Sprite;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),
    #[error("command `{0}` is not available right now: {1}")]
    Disabled(String, String),
    #[error("invalid parameters for `{cmd}`: {msg}")]
    BadParams { cmd: String, msg: String },
    #[error("no sprite open")]
    NoDocument,
    #[error("{0}")]
    Other(String),
    #[error("internal error in `{0}` (the sprite was kept as it was): {1}")]
    Internal(String, String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub label: String,
    pub sprite: Arc<Sprite>,
}

/// One open sprite with its history and the timeline cursor.
#[derive(Clone, Debug)]
pub struct Document {
    pub uid: u64,
    pub name: String,
    pub path: Option<String>,
    pub sprite: Arc<Sprite>,
    /// The active layer and frame (the timeline's current cel).
    pub layer: usize,
    pub frame: usize,
    pub undo: Vec<HistoryEntry>,
    pub redo: Vec<HistoryEntry>,
    /// The undo depth at which the document matches its file (or, never saved, its new state);
    /// `None` once that state can't be reached by undo/redo any more.
    pub saved_at: Option<usize>,
    /// The selected pixels (canvas-sized), if anything is selected.
    pub selection: Option<pixelcraft_raster::Mask>,
    /// The last selection that was deselected, for Select ▸ Reselect.
    pub last_selection: Option<pixelcraft_raster::Mask>,
}

impl Document {
    /// Unsaved changes: the history is not at the saved point. Undoing back to it makes the
    /// document clean again, like the target app.
    pub fn is_dirty(&self) -> bool {
        self.saved_at != Some(self.undo.len())
    }

    /// Marks the current state as saved.
    pub fn mark_saved(&mut self) {
        self.saved_at = Some(self.undo.len());
    }
}

/// What the UI and agents see of the session, read through `engine.state`.
#[derive(Clone, Debug, Serialize)]
pub struct StateSummary {
    pub documents: Vec<String>,
    pub active: Option<usize>,
}

#[derive(Debug, Default)]
pub struct Session {
    pub docs: Vec<Document>,
    pub active: Option<usize>,
    next_uid: u64,
    /// Numbers new sprites `Sprite-0001`, `-0002`… for the whole session; never reused.
    next_sprite_number: u32,
    /// What Edit ▸ Copy / Cut took, for Edit ▸ Paste (shared by every open sprite).
    pub clipboard: Option<Clip>,
}

/// Copied pixels: RGBA whatever the source sprite's colour mode (paste converts to the target's),
/// transparent outside the selection, and where they were on the canvas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub image: pixelcraft_raster::Image,
    pub origin: pixelcraft_geom::Point,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self) -> Option<&Document> {
        self.docs.get(self.active?)
    }

    pub fn active_mut(&mut self) -> Option<&mut Document> {
        let i = self.active?;
        self.docs.get_mut(i)
    }

    /// Opens `sprite` as a new document and makes it active.
    pub fn add_document(&mut self, name: impl Into<String>, sprite: Sprite) -> usize {
        self.next_uid = self.next_uid.saturating_add(1);
        self.docs.push(Document {
            uid: self.next_uid,
            name: name.into(),
            path: None,
            sprite: Arc::new(sprite),
            layer: 0,
            frame: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            saved_at: Some(0),
            selection: None,
            last_selection: None,
        });
        let i = self.docs.len() - 1;
        self.active = Some(i);
        i
    }

    /// The next untitled sprite name, `Sprite-0001` first.
    /// Opens an image file's `bytes` as a new, active document named `name` (the file name),
    /// remembering `path` when there is one (a browser drop has none). Not a command because the
    /// bytes don't travel as JSON; `file.open` reads a path and calls this.
    pub fn open_image(&mut self, name: &str, path: Option<String>, bytes: &[u8]) -> std::result::Result<usize, pixelcraft_io::IoError> {
        let sprite = pixelcraft_io::read_image(bytes)?;
        let i = self.add_document(name, sprite);
        if let Some(d) = self.docs.get_mut(i) {
            d.path = path;
        }
        Ok(i)
    }

    /// Where Edit ▸ Paste puts the clipboard on the active sprite by default (top-left).
    pub fn paste_position(&self) -> Option<pixelcraft_geom::Point> {
        let clip = self.clipboard.as_ref()?;
        let size = self.active()?.sprite.size;
        Some(cmd::edit::paste_position(clip, size, clip.image.size().w, clip.image.size().h))
    }

    pub fn next_sprite_name(&mut self) -> String {
        self.next_sprite_number = self.next_sprite_number.saturating_add(1);
        format!("Sprite-{:04}", self.next_sprite_number)
    }

    pub fn summary(&self) -> StateSummary {
        StateSummary { documents: self.docs.iter().map(|d| d.name.clone()).collect(), active: self.active }
    }

    /// Runs command `id` with `params`. An undoable command snapshots the sprite first; if the
    /// command fails or panics, the sprite is restored and the error is returned.
    pub fn execute(&mut self, id: &str, params: &Json) -> Result<Json> {
        let spec = find_command(id).ok_or_else(|| EngineError::UnknownCommand(id.to_string()))?;
        (spec.enabled)(self).map_err(|why| EngineError::Disabled(id.to_string(), why))?;
        let before = self.active().map(|d| (d.uid, Arc::clone(&d.sprite)));
        let result = catch_unwind(AssertUnwindSafe(|| (spec.run)(self, params)))
            .unwrap_or_else(|_| Err(EngineError::Internal(id.to_string(), "the command panicked".into())));
        let Some((uid, old)) = before else { return result };
        let Some(doc) = self.docs.iter_mut().find(|d| d.uid == uid) else { return result };
        match &result {
            Err(_) => doc.sprite = old,
            Ok(_) if spec.undoable && !Arc::ptr_eq(&old, &doc.sprite) => {
                // A saved state that was undone past can't come back once history branches.
                if doc.saved_at.is_some_and(|at| at > doc.undo.len()) {
                    doc.saved_at = None;
                }
                doc.undo.push(HistoryEntry { label: spec.label.to_string(), sprite: old });
                doc.redo.clear();
            }
            Ok(_) => {}
        }
        result
    }
}

#[cfg(test)]
mod tests;
