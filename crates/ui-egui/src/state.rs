//! UI state that isn't part of the document: the current tool, colours, viewports, panels.
//! Serializable where it makes sense, so the control channel can read and drive it.

use std::collections::HashMap;

use pixelcraft_engine::color::Rgba8;
use serde::{Deserialize, Serialize};

use crate::zoom::Viewport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tool {
    #[default]
    Pencil,
    Eraser,
    Eyedropper,
    Zoom,
    Hand,
    Move,
    Bucket,
    Line,
    Rectangle,
    Ellipse,
}

impl Tool {
    /// Tool bar order.
    pub const ALL: [Tool; 10] =
        [Tool::Pencil, Tool::Eraser, Tool::Eyedropper, Tool::Zoom, Tool::Hand, Tool::Move, Tool::Bucket, Tool::Line, Tool::Rectangle, Tool::Ellipse];

    pub fn label(self) -> &'static str {
        match self {
            Tool::Pencil => "Pencil",
            Tool::Eraser => "Eraser",
            Tool::Eyedropper => "Eyedropper",
            Tool::Zoom => "Zoom",
            Tool::Hand => "Hand",
            Tool::Move => "Move",
            Tool::Bucket => "Paint Bucket",
            Tool::Line => "Line",
            Tool::Rectangle => "Rectangle",
            Tool::Ellipse => "Ellipse",
        }
    }

    /// The key that selects the tool (the target app's defaults).
    pub fn shortcut(self) -> &'static str {
        match self {
            Tool::Pencil => "B",
            Tool::Eraser => "E",
            Tool::Eyedropper => "I",
            Tool::Zoom => "Z",
            Tool::Hand => "H",
            Tool::Move => "V",
            Tool::Bucket => "G",
            Tool::Line => "L",
            Tool::Rectangle => "U",
            Tool::Ellipse => "Shift+U",
        }
    }

    /// Whether the tool does something yet; the others say so in the status bar.
    pub fn works(self) -> bool {
        matches!(self, Tool::Pencil | Tool::Eraser | Tool::Eyedropper | Tool::Zoom | Tool::Hand)
    }
}

/// The New Sprite dialog's values; the dialog opens with the last ones used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewSpriteForm {
    pub width: u32,
    pub height: u32,
    /// `rgba`, `grayscale` or `indexed` (the `file.new` param values).
    pub color_mode: String,
    /// `transparent`, `white` or `black`.
    pub background: String,
    pub advanced: bool,
}

impl Default for NewSpriteForm {
    fn default() -> Self {
        Self { width: 32, height: 32, color_mode: "rgba".into(), background: "transparent".into(), advanced: false }
    }
}

/// The modal dialog on screen, if any.
#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    /// File ▸ New: the form being edited (width/height as typed text).
    NewSprite { form: NewSpriteForm, width: String, height: String, focus: bool },
    /// "Saving changes to the sprite … before closing/quitting?" for document `uid`.
    Unsaved { uid: u64, quitting: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiState {
    pub tool: Tool,
    pub fg: Rgba8,
    pub bg: Rgba8,
    /// The colour picker's hue, kept while the colour is grey.
    pub picker_hue: f32,
    pub pixel_perfect: bool,
    /// Flips the wheel's zoom direction (View ▸ Invert Wheel Zoom). Off by default.
    pub invert_wheel_zoom: bool,
    pub timeline_visible: bool,
    pub playing: bool,
    /// The sprite pixel under the pointer, for the status bar.
    #[serde(skip)]
    pub hover_pixel: Option<(i32, i32)>,
    /// Zoom and pan per open document (keyed by document uid).
    #[serde(skip)]
    pub viewports: HashMap<u64, Viewport>,
    /// The Home tab: open at launch, closable, reopened by View ▸ Home.
    pub home_open: bool,
    /// Home is the active tab (otherwise the session's active document is).
    pub home_active: bool,
    pub new_sprite: NewSpriteForm,
    #[serde(skip)]
    pub dialog: Option<Dialog>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            tool: Tool::Pencil,
            fg: Rgba8::BLACK,
            bg: Rgba8::WHITE,
            picker_hue: 0.0,
            pixel_perfect: false,
            invert_wheel_zoom: false,
            timeline_visible: true,
            playing: false,
            hover_pixel: None,
            viewports: HashMap::new(),
            home_open: true,
            home_active: true,
            new_sprite: NewSpriteForm::default(),
            dialog: None,
        }
    }
}
