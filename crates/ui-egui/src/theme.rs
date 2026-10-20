//! Design tokens and the pixel skin. Every colour and size the UI paints comes from here; panels
//! never hard-code one.
//!
//! **Look:** a pixel-art interface. One UI pixel is [`size::UI_PX`] points (the target app's
//! default 200 % scale); text uses PixelCraft's own pixel font ([`crate::pixel_font`]); shapes are
//! drawn without anti-aliasing, without rounded corners or shadows; raised controls get a
//! one-pixel bevel ([`bevel`]). **Colours:** an original "dusk" palette — violet-grey panels, a
//! warm orange accent — deliberately not the target app's grey-blue and beige. Sizes follow the
//! proportions measured in the target app (`plan/aseprite/observed-ui.md`), in egui points.

use egui::{Color32, Rect, Stroke, StrokeKind, vec2};

/// Panel sizes in points.
pub mod size {
    /// One pixel of the pixel-art interface, in points.
    pub const UI_PX: f32 = 2.0;
    pub const TAB_BAR_H: f32 = 24.0;
    pub const CONTEXT_BAR_H: f32 = 34.0;
    pub const COLOR_BAR_W: f32 = 144.0;
    pub const SWATCH: f32 = 26.0;
    pub const PALETTE_COLUMNS: usize = 5;
    pub const TOOL_BAR_W: f32 = 36.0;
    pub const TOOL_BUTTON: f32 = 30.0;
    pub const TIMELINE_H: f32 = 140.0;
    pub const TIMELINE_ROW_H: f32 = 22.0;
    pub const TIMELINE_CEL_W: f32 = 22.0;
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
    /// Menu bar, tab strip, status bar.
    pub window: Color32,
    pub panel: Color32,
    pub panel_alt: Color32,
    /// Raised controls: face, hover face, and the bevel's light and dark edges.
    pub button: Color32,
    pub button_hover: Color32,
    pub bevel_light: Color32,
    pub bevel_dark: Color32,
    /// Outlines around controls and panels.
    pub border: Color32,
    /// Sunken fields (text boxes, lists).
    pub field: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub accent_light: Color32,
    pub accent_dark: Color32,
    pub on_accent: Color32,
    pub canvas_bg: Color32,
    pub canvas_border: Color32,
    pub checker_a: Color32,
    pub checker_b: Color32,
    pub row_active: Color32,
    pub cel_active: Color32,
    pub tooltip: Color32,
    pub on_tooltip: Color32,
}

impl Tokens {
    /// "Dusk": the default theme.
    pub fn dusk() -> Self {
        Self {
            dark: true,
            window: Color32::from_rgb(0x1f, 0x1b, 0x2e),
            panel: Color32::from_rgb(0x2c, 0x27, 0x42),
            panel_alt: Color32::from_rgb(0x24, 0x1f, 0x36),
            button: Color32::from_rgb(0x3b, 0x34, 0x56),
            button_hover: Color32::from_rgb(0x4a, 0x42, 0x69),
            bevel_light: Color32::from_rgb(0x5f, 0x55, 0x85),
            bevel_dark: Color32::from_rgb(0x17, 0x13, 0x22),
            border: Color32::from_rgb(0x0e, 0x0b, 0x16),
            field: Color32::from_rgb(0x17, 0x14, 0x24),
            text: Color32::from_rgb(0xef, 0xe9, 0xff),
            text_dim: Color32::from_rgb(0xa3, 0x9c, 0xbf),
            accent: Color32::from_rgb(0xff, 0x9f, 0x43),
            accent_light: Color32::from_rgb(0xff, 0xcf, 0x8a),
            accent_dark: Color32::from_rgb(0xb3, 0x5c, 0x1e),
            on_accent: Color32::from_rgb(0x1f, 0x1b, 0x2e),
            canvas_bg: Color32::from_rgb(0x3a, 0x33, 0x50),
            canvas_border: Color32::from_rgb(0x0e, 0x0b, 0x16),
            checker_a: Color32::from_rgb(0xc9, 0xc4, 0xd6),
            checker_b: Color32::from_rgb(0xa2, 0x9d, 0xb3),
            row_active: Color32::from_rgb(0x3d, 0x33, 0x58),
            cel_active: Color32::from_rgb(0x8a, 0x5a, 0x2e),
            tooltip: Color32::from_rgb(0xff, 0xe0, 0x8a),
            on_tooltip: Color32::from_rgb(0x1f, 0x1b, 0x2e),
        }
    }

    pub fn get(_ctx: &egui::Context) -> Self {
        Self::dusk()
    }
}

/// Once per context: the pixel font, text sizes, the pixel style.
pub fn setup(ctx: &egui::Context, t: &Tokens) {
    crate::pixel_font::install(ctx);
    let px = size::UI_PX;
    ctx.all_styles_mut(|s| {
        use egui::{FontId, TextStyle};
        let font = FontId::proportional(crate::pixel_font::SIZE);
        s.text_styles = [
            (TextStyle::Small, font.clone()),
            (TextStyle::Body, font.clone()),
            (TextStyle::Button, font.clone()),
            (TextStyle::Heading, font),
            (TextStyle::Monospace, FontId::monospace(crate::pixel_font::SIZE)),
        ]
        .into();
        s.spacing.item_spacing = vec2(4.0 * px, 2.0 * px);
        s.spacing.button_padding = vec2(3.0 * px, 0.0);
        s.spacing.interact_size.y = 10.0 * px;
        s.spacing.menu_margin = egui::Margin::same(3 * px as i8);
        s.spacing.window_margin = egui::Margin::same(4 * px as i8);

        let v = &mut s.visuals;
        v.dark_mode = t.dark;
        v.panel_fill = t.panel;
        v.window_fill = t.panel;
        v.extreme_bg_color = t.field;
        v.faint_bg_color = t.panel_alt;
        v.override_text_color = Some(t.text);
        v.selection.bg_fill = t.accent;
        v.selection.stroke = Stroke::new(px, t.on_accent);
        v.hyperlink_color = t.accent;
        v.window_corner_radius = egui::CornerRadius::ZERO;
        v.menu_corner_radius = egui::CornerRadius::ZERO;
        v.window_shadow = egui::Shadow::NONE;
        v.popup_shadow = egui::Shadow::NONE;
        v.window_stroke = Stroke::new(px, t.border);
        v.text_cursor.stroke = Stroke::new(px, t.accent);
        for (w, fill, stroke) in [
            (&mut v.widgets.noninteractive, t.panel, t.border),
            (&mut v.widgets.inactive, t.button, t.border),
            (&mut v.widgets.hovered, t.button_hover, t.border),
            (&mut v.widgets.active, t.accent_dark, t.border),
            (&mut v.widgets.open, t.button_hover, t.border),
        ] {
            w.corner_radius = egui::CornerRadius::ZERO;
            w.bg_fill = fill;
            w.weak_bg_fill = fill;
            w.bg_stroke = Stroke::new(px, stroke);
            w.fg_stroke = Stroke::new(px, t.text);
            w.expansion = 0.0;
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(px, t.bevel_dark);
        v.widgets.noninteractive.fg_stroke = Stroke::new(px, t.text_dim);
        s.interaction.tooltip_delay = 0.6;
    });
    ctx.options_mut(|o| {
        // Cmd+= / Cmd+- zoom the canvas, not the whole interface.
        o.zoom_with_keyboard = false;
        // Hard pixel edges: no anti-aliasing on shapes.
        o.tessellation_options.feathering = false;
    });
}

/// A raised pixel-art control: a one-UI-pixel outline, light top/left and dark bottom/right
/// edges, and the face. `pressed` sinks it (edges swap).
pub fn bevel(p: &egui::Painter, r: Rect, face: Color32, pressed: bool, t: &Tokens) {
    let px = size::UI_PX;
    p.rect_filled(r, 0.0, t.border);
    let inner = r.shrink(px);
    let (tl, br) = if pressed { (t.bevel_dark, t.bevel_light) } else { (t.bevel_light, t.bevel_dark) };
    p.rect_filled(inner, 0.0, br);
    p.rect_filled(Rect::from_min_max(inner.min, inner.max - vec2(px, px)), 0.0, tl);
    p.rect_filled(Rect::from_min_max(inner.min + vec2(px, px), inner.max - vec2(px, px)), 0.0, face);
}

/// The accent-coloured bevel of a selected control.
pub fn bevel_selected(p: &egui::Painter, r: Rect, t: &Tokens) {
    let px = size::UI_PX;
    p.rect_filled(r, 0.0, t.border);
    let inner = r.shrink(px);
    p.rect_filled(inner, 0.0, t.accent_dark);
    p.rect_filled(Rect::from_min_max(inner.min, inner.max - vec2(px, px)), 0.0, t.accent_light);
    p.rect_filled(Rect::from_min_max(inner.min + vec2(px, px), inner.max - vec2(px, px)), 0.0, t.accent);
}

/// A sunken field: dark outline and face, lighter bottom/right edge.
pub fn sunken(p: &egui::Painter, r: Rect, t: &Tokens) {
    let px = size::UI_PX;
    p.rect_filled(r, 0.0, t.bevel_light);
    p.rect_filled(Rect::from_min_max(r.min, r.max - vec2(px, px)), 0.0, t.border);
    p.rect_filled(Rect::from_min_max(r.min + vec2(px, px), r.max - vec2(px, px)), 0.0, t.field);
}

/// A one-UI-pixel line along the bottom (or top) edge of a panel.
pub fn edge(p: &egui::Painter, r: Rect, bottom: bool, t: &Tokens) {
    let px = size::UI_PX;
    let y = if bottom { r.max.y - px } else { r.min.y };
    p.rect_filled(Rect::from_min_max(egui::pos2(r.min.x, y), egui::pos2(r.max.x, y + px)), 0.0, t.border);
}

/// Outline only (for frames around lists and previews).
pub fn outline(p: &egui::Painter, r: Rect, t: &Tokens) {
    p.rect_stroke(r, 0.0, Stroke::new(size::UI_PX, t.border), StrokeKind::Inside);
}
