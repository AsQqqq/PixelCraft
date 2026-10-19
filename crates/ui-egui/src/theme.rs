//! Design tokens. Every colour and size the UI paints comes from here; panels never hard-code one.
//!
//! The palette is original to PixelCraft: a dark, low-contrast studio look that keeps attention on
//! the pixels. Sizes follow the proportions measured in the target app at its default 200 % UI
//! scale (`plan/aseprite/observed-ui.md`), in egui points.

use egui::Color32;

/// Panel sizes in points.
pub mod size {
    pub const TAB_BAR_H: f32 = 24.0;
    pub const CONTEXT_BAR_H: f32 = 34.0;
    pub const COLOR_BAR_W: f32 = 144.0;
    pub const SWATCH: f32 = 26.0;
    pub const PALETTE_COLUMNS: usize = 5;
    pub const TOOL_BAR_W: f32 = 36.0;
    pub const TOOL_BUTTON: f32 = 30.0;
    pub const TIMELINE_H: f32 = 140.0;
    pub const TIMELINE_ROW_H: f32 = 20.0;
    pub const TIMELINE_CEL_W: f32 = 20.0;
    pub const TIMELINE_LAYER_W: f32 = 200.0;
    pub const STATUS_BAR_H: f32 = 24.0;
    /// Side of a transparency checkerboard square, in sprite pixels (scales with the zoom)…
    pub const CHECKER_SPRITE_PX: f32 = 16.0;
    /// …but never smaller than this many screen points.
    pub const CHECKER_MIN: f32 = 8.0;
}

#[derive(Clone, Debug)]
pub struct Tokens {
    pub dark: bool,
    pub window: Color32,
    pub panel: Color32,
    pub panel_alt: Color32,
    pub button: Color32,
    pub button_hover: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub canvas_bg: Color32,
    pub canvas_border: Color32,
    pub checker_a: Color32,
    pub checker_b: Color32,
    pub row_active: Color32,
    pub cel_active: Color32,
}

impl Tokens {
    pub fn dark() -> Self {
        Self {
            dark: true,
            window: Color32::from_rgb(0x1e, 0x1d, 0x24),
            panel: Color32::from_rgb(0x2a, 0x29, 0x33),
            panel_alt: Color32::from_rgb(0x24, 0x23, 0x2c),
            button: Color32::from_rgb(0x38, 0x37, 0x44),
            button_hover: Color32::from_rgb(0x46, 0x45, 0x55),
            border: Color32::from_rgb(0x12, 0x11, 0x16),
            text: Color32::from_rgb(0xe6, 0xe4, 0xee),
            text_dim: Color32::from_rgb(0x9a, 0x97, 0xa8),
            accent: Color32::from_rgb(0xf2, 0x9a, 0x4a),
            on_accent: Color32::from_rgb(0x1e, 0x1d, 0x24),
            canvas_bg: Color32::from_rgb(0x34, 0x33, 0x3f),
            canvas_border: Color32::from_rgb(0x0c, 0x0b, 0x10),
            checker_a: Color32::from_rgb(0xc4, 0xc4, 0xc8),
            checker_b: Color32::from_rgb(0x9c, 0x9c, 0xa2),
            row_active: Color32::from_rgb(0x3a, 0x33, 0x2c),
            cel_active: Color32::from_rgb(0x6b, 0x4a, 0x2a),
        }
    }

    pub fn get(_ctx: &egui::Context) -> Self {
        Self::dark()
    }
}

pub fn apply(ctx: &egui::Context, t: &Tokens) {
    ctx.all_styles_mut(|s| {
        s.visuals.dark_mode = t.dark;
        s.visuals.panel_fill = t.panel;
        s.visuals.window_fill = t.window;
        s.visuals.override_text_color = Some(t.text);
        s.visuals.selection.bg_fill = t.accent;
        s.visuals.selection.stroke.color = t.on_accent;
        s.visuals.widgets.inactive.weak_bg_fill = t.button;
        s.visuals.widgets.hovered.weak_bg_fill = t.button_hover;
    });
    // Cmd+= / Cmd+- zoom the canvas, not the whole interface.
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}
