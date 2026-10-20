//! UI state that isn't part of the document: the current tool, colours, viewports, panels.
//! Serializable where it makes sense, so the control channel can read and drive it.

use std::collections::HashMap;

use pixelcraft_engine::color::Rgba8;
use serde::{Deserialize, Serialize};

use crate::zoom::Viewport;

/// Every tool, in the target app's tool bar order (`plan/aseprite/tools.md`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tool {
    RectangularMarquee,
    EllipticalMarquee,
    Lasso,
    PolygonalLasso,
    MagicWand,
    #[default]
    Pencil,
    Spray,
    Eraser,
    Eyedropper,
    Zoom,
    Hand,
    Move,
    Slice,
    Bucket,
    Gradient,
    Line,
    Curve,
    Rectangle,
    FilledRectangle,
    Ellipse,
    FilledEllipse,
    Contour,
    Polygon,
    Blur,
    Jumble,
    Text,
}

/// The tool bar's twelve buttons, top to bottom; each holds a group of tools.
pub const TOOL_GROUPS: [&[Tool]; 12] = [
    &[Tool::RectangularMarquee, Tool::EllipticalMarquee, Tool::Lasso, Tool::PolygonalLasso, Tool::MagicWand],
    &[Tool::Pencil, Tool::Spray],
    &[Tool::Eraser],
    &[Tool::Eyedropper],
    &[Tool::Zoom, Tool::Hand],
    &[Tool::Move, Tool::Slice],
    &[Tool::Bucket, Tool::Gradient],
    &[Tool::Line, Tool::Curve],
    &[Tool::Rectangle, Tool::FilledRectangle, Tool::Ellipse, Tool::FilledEllipse],
    &[Tool::Contour, Tool::Polygon],
    &[Tool::Blur, Tool::Jumble],
    &[Tool::Text],
];

impl Tool {
    pub const ALL: [Tool; 26] = [
        Tool::RectangularMarquee,
        Tool::EllipticalMarquee,
        Tool::Lasso,
        Tool::PolygonalLasso,
        Tool::MagicWand,
        Tool::Pencil,
        Tool::Spray,
        Tool::Eraser,
        Tool::Eyedropper,
        Tool::Zoom,
        Tool::Hand,
        Tool::Move,
        Tool::Slice,
        Tool::Bucket,
        Tool::Gradient,
        Tool::Line,
        Tool::Curve,
        Tool::Rectangle,
        Tool::FilledRectangle,
        Tool::Ellipse,
        Tool::FilledEllipse,
        Tool::Contour,
        Tool::Polygon,
        Tool::Blur,
        Tool::Jumble,
        Tool::Text,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tool::RectangularMarquee => "Rectangular Marquee",
            Tool::EllipticalMarquee => "Elliptical Marquee",
            Tool::Lasso => "Lasso",
            Tool::PolygonalLasso => "Polygonal Lasso",
            Tool::MagicWand => "Magic Wand",
            Tool::Pencil => "Pencil",
            Tool::Spray => "Spray",
            Tool::Eraser => "Eraser",
            Tool::Eyedropper => "Eyedropper",
            Tool::Zoom => "Zoom",
            Tool::Hand => "Hand",
            Tool::Move => "Move",
            Tool::Slice => "Slice",
            Tool::Bucket => "Paint Bucket",
            Tool::Gradient => "Gradient",
            Tool::Line => "Line",
            Tool::Curve => "Curve",
            Tool::Rectangle => "Rectangle",
            Tool::FilledRectangle => "Filled Rectangle",
            Tool::Ellipse => "Ellipse",
            Tool::FilledEllipse => "Filled Ellipse",
            Tool::Contour => "Contour",
            Tool::Polygon => "Polygon",
            Tool::Blur => "Blur",
            Tool::Jumble => "Jumble",
            Tool::Text => "Text",
        }
    }

    /// The key that selects the tool (the target app's defaults). Pressing it again cycles
    /// through the tools that share it.
    pub fn shortcut(self) -> &'static str {
        match self {
            Tool::RectangularMarquee => "M",
            Tool::EllipticalMarquee => "Shift+M",
            Tool::Lasso => "Q",
            Tool::PolygonalLasso => "Shift+Q",
            Tool::MagicWand => "W",
            Tool::Pencil => "B",
            Tool::Spray => "Shift+B",
            Tool::Eraser => "E",
            Tool::Eyedropper => "I",
            Tool::Zoom => "Z",
            Tool::Hand => "H",
            Tool::Move => "V",
            Tool::Slice => "Shift+C",
            Tool::Bucket => "G",
            Tool::Gradient => "Shift+G",
            Tool::Line => "L",
            Tool::Curve => "Shift+L",
            Tool::Rectangle | Tool::FilledRectangle => "U",
            Tool::Ellipse | Tool::FilledEllipse => "Shift+U",
            Tool::Contour => "D",
            Tool::Polygon => "Shift+D",
            Tool::Blur | Tool::Jumble => "R",
            Tool::Text => "T",
        }
    }

    /// What the mouse buttons do, for the tooltip (empty for most tools).
    pub fn buttons(self) -> &'static [&'static str] {
        if self.selects() {
            return &["Left Button: replace/add to current selection", "Right Button: remove from current selection"];
        }
        match self {
            Tool::Eraser => &[
                "Left Button: Erase with the background color in \"Background\" layer or transparent color in any other layer",
                "Right Button: Replace foreground with background color",
            ],
            _ => &[],
        }
    }

    /// The tool bar group (0–11) the tool belongs to.
    pub fn group(self) -> usize {
        TOOL_GROUPS.iter().position(|g| g.contains(&self)).unwrap_or(0)
    }

    /// Selection tools: they make the selection instead of painting.
    pub fn selects(self) -> bool {
        matches!(self, Tool::RectangularMarquee | Tool::EllipticalMarquee | Tool::Lasso | Tool::PolygonalLasso | Tool::MagicWand)
    }

    /// Tools that paint with the brush and show its preview under the pointer.
    pub fn paints_with_brush(self) -> bool {
        matches!(self, Tool::Pencil | Tool::Spray | Tool::Eraser | Tool::Line)
    }

    /// Whether the tool does something yet; the others say so in the status bar.
    pub fn works(self) -> bool {
        !matches!(self, Tool::Slice | Tool::Gradient | Tool::Curve | Tool::Blur | Tool::Jumble | Tool::Text)
    }
}

/// The Preview window: it plays the animation on its own (the editor stays on its frame) and
/// has its own zoom and pan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviewState {
    pub playing: bool,
    /// The frame shown while playing.
    pub frame: usize,
    /// Milliseconds spent on `frame`.
    pub elapsed_ms: f32,
    /// `None`: fit the sprite in the window; otherwise this zoom and `pan`.
    pub zoom: Option<f32>,
    pub pan: egui::Vec2,
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

/// Edit ▸ Preferences (sections as in the target app's Preferences dialog).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    /// Cursors ▸ "Use native mouse cursors": the OS's own cursors instead of the pixel ones.
    pub native_cursor: bool,
    /// Cursors ▸ "Mouse Cursor Scale": 1 = 100 %, 2 = 200 %, 3 = 300 %.
    pub cursor_scale: u8,
    /// Editor ▸ "Invert the mouse wheel zoom direction".
    pub invert_wheel_zoom: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self { native_cursor: false, cursor_scale: 1, invert_wheel_zoom: false }
    }
}

/// A section of the Preferences dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefSection {
    Editor,
    Cursors,
}

/// The modal dialog on screen, if any.
#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    /// File ▸ New: the form being edited (width/height as typed text).
    NewSprite { form: NewSpriteForm, width: String, height: String, focus: bool },
    /// "Saving changes to the sprite … before closing/quitting?" for document `uid`.
    Unsaved { uid: u64, quitting: bool },
    /// Edit ▸ Preferences: the section shown and the values being edited (applied on OK/Apply).
    Preferences { section: PrefSection, draft: Preferences },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiState {
    pub tool: Tool,
    /// The tool each tool bar button shows (the group's last-used tool).
    pub group_tool: [Tool; 12],
    /// Brush diameter in pixels (1–64); `-` and `+` change it.
    pub brush_size: u32,
    /// How marquee, lasso and wand combine with the current selection (context bar).
    pub select_mode: pixelcraft_engine::raster::SelectMode,
    /// The floating Preview window (F7, tool bar).
    pub show_preview: bool,
    /// The Preview window's own playback and view.
    #[serde(skip)]
    pub preview: PreviewState,
    /// The tool bar button under the pointer, for the status bar.
    #[serde(skip)]
    pub hover_tool: Option<Tool>,
    pub fg: Rgba8,
    pub bg: Rgba8,
    /// The colour picker's hue, kept while the colour is grey.
    pub picker_hue: f32,
    pub pixel_perfect: bool,
    /// Edit ▸ Preferences.
    pub prefs: Preferences,
    pub timeline_visible: bool,
    /// A frame selected in the timeline by clicking its number: Delete/Backspace then delete the
    /// frame (Frame ▸ Delete Frame) instead of clearing pixels, like the target app. Clicking a
    /// cel, a layer or the canvas drops it.
    pub timeline_frame: Option<usize>,
    pub playing: bool,
    /// The sprite pixel under the pointer, for the status bar.
    #[serde(skip)]
    pub hover_pixel: Option<(i32, i32)>,
    /// Zoom and pan per open document (keyed by document uid).
    #[serde(skip)]
    pub viewports: HashMap<u64, Viewport>,
    /// Documents just opened from a file: their first view fits them when they are bigger
    /// than the editor (a photo), instead of 100 %.
    pub fit_on_open: std::collections::HashSet<u64>,
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
            group_tool: TOOL_GROUPS.map(|g| g.first().copied().unwrap_or_default()),
            brush_size: 1,
            select_mode: pixelcraft_engine::raster::SelectMode::Replace,
            show_preview: true,
            preview: PreviewState::default(),
            hover_tool: None,
            fg: Rgba8::BLACK,
            bg: Rgba8::WHITE,
            picker_hue: 0.0,
            pixel_perfect: false,
            prefs: Preferences::default(),
            timeline_visible: true,
            timeline_frame: None,
            playing: false,
            hover_pixel: None,
            viewports: HashMap::new(),
            fit_on_open: Default::default(),
            home_open: true,
            home_active: true,
            new_sprite: NewSpriteForm::default(),
            dialog: None,
        }
    }
}
