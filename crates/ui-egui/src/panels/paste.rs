//! Edit ▸ Paste with a transform box, like the target app: the copied pixels float over the
//! sprite inside a box until they are applied.
//!
//! - drag inside the box: move it (whole pixels);
//! - drag one of the eight handles: stretch it (Shift keeps the proportions);
//! - drag just outside a corner: rotate it about its centre (Shift snaps to 15°);
//! - Enter, or a click elsewhere on the canvas: apply (`edit.paste` with the box) on the layer and
//!   frame that are current then, so switching layer or frame first pastes there;
//! - Escape (or Delete, or Undo): drop it.
//!
//! The floating preview is the same nearest-neighbour transform the engine applies.

use egui::{Color32, Pos2, Rect, Stroke, Vec2, pos2, vec2};
use pixelcraft_engine::raster::transform::scale_rotate;
use serde_json::json;

use crate::PixelApp;
use crate::theme::{Tokens, size};
use crate::zoom::Viewport;

/// Screen distance (points) at which a handle catches the pointer.
const HANDLE_HIT: f32 = 7.0;
/// How far outside a corner (points) the rotate zone reaches.
const ROTATE_ZONE: f32 = 24.0;

/// The floating paste: the box in sprite pixels (top-left of the unrotated box) and its turn.
#[derive(Clone, Debug, PartialEq)]
pub struct Pasting {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Degrees, clockwise on screen.
    pub angle: f32,
    pub drag: Option<PasteDrag>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PasteDrag {
    /// Moving: the pointer and the box where the drag started.
    Move { from: Vec2, start: (f32, f32) },
    /// Stretching by handle (`hx`, `hy` ∈ -1, 0, 1) from the box as it was.
    Handle { hx: i8, hy: i8, start: Box<Pasting> },
    /// Turning: the angle and the pointer's bearing from the centre when the drag started.
    Rotate { start_angle: f32, from_bearing: f32 },
}

/// What the pointer is over.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Hit {
    Handle(i8, i8),
    Inside,
    Rotate,
    Outside,
}

impl Pasting {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x: x as f32, y: y as f32, w: w as f32, h: h as f32, angle: 0.0, drag: None }
    }

    fn center(&self) -> Vec2 {
        vec2(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// The box's size and position in whole pixels, as it will be applied.
    pub fn rounded(&self) -> (i32, i32, u32, u32) {
        (self.x.round() as i32, self.y.round() as i32, self.w.round().max(1.0) as u32, self.h.round().max(1.0) as u32)
    }

    /// A point in sprite pixels, in the box's own frame (unrotated, origin at its centre).
    fn local(&self, p: Vec2) -> Vec2 {
        rotate(p - self.center(), -self.angle)
    }

    /// The handle at local offset (`hx`, `hy`) (and the corners) in sprite pixels.
    fn handle(&self, hx: i8, hy: i8) -> Vec2 {
        self.center() + rotate(vec2(f32::from(hx) * self.w / 2.0, f32::from(hy) * self.h / 2.0), self.angle)
    }

    fn hit(&self, p: Vec2, zoom: f32) -> Hit {
        let reach = HANDLE_HIT / zoom;
        let l = self.local(p);
        let (hw, hh) = (self.w / 2.0, self.h / 2.0);
        let inside = l.x.abs() <= hw && l.y.abs() <= hh;
        // A box small on screen is all handles: there, its middle moves it.
        let small = self.w.min(self.h) * zoom < 4.0 * HANDLE_HIT;
        if small && inside && l.x.abs() <= hw / 2.0 && l.y.abs() <= hh / 2.0 {
            return Hit::Inside;
        }
        for (hx, hy) in HANDLES {
            if (self.handle(hx, hy) - p).length() <= reach {
                return Hit::Handle(hx, hy);
            }
        }
        if inside {
            return Hit::Inside;
        }
        let zone = ROTATE_ZONE / zoom;
        let near_corner = [(-1, -1), (1, -1), (1, 1), (-1, 1)].into_iter().any(|(hx, hy)| (self.handle(hx, hy) - p).length() <= zone);
        if near_corner { Hit::Rotate } else { Hit::Outside }
    }
}

const HANDLES: [(i8, i8); 8] = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

fn rotate(v: Vec2, deg: f32) -> Vec2 {
    let (s, c) = deg.to_radians().sin_cos();
    vec2(v.x * c - v.y * s, v.x * s + v.y * c)
}

fn bearing(v: Vec2) -> f32 {
    v.y.atan2(v.x).to_degrees()
}

impl PixelApp {
    /// Edit ▸ Paste from a menu or key: the copied pixels float in a transform box.
    pub fn start_paste(&mut self) {
        let (Some(s), Some(at)) = (self.session.clipboard.as_ref().map(|c| c.image.size()), self.session.paste_position()) else {
            self.status = Some("Nothing has been copied".into());
            return;
        };
        if self.active_doc().is_none() {
            return;
        }
        // A paste already floating is applied first, like the target app.
        self.apply_paste();
        self.ui.pasting = Some(Pasting::new(at.x, at.y, s.w, s.h));
        // A new clipboard image: the preview is made again.
        self.paste_preview = PreviewCache::default();
    }

    /// Applies the floating paste to the current layer and frame.
    pub fn apply_paste(&mut self) {
        let Some(p) = self.ui.pasting.take() else { return };
        let (x, y, w, h) = p.rounded();
        self.run("edit.paste", &json!({ "x": x, "y": y, "width": w, "height": h, "angle": p.angle }));
    }

    pub fn cancel_paste(&mut self) {
        self.ui.pasting = None;
    }
}

/// The transformed clipboard image for the preview, kept while the box keeps its size and turn.
#[derive(Default)]
pub struct PreviewCache {
    key: Option<(u32, u32, u32)>,
    tiles: crate::tiles::Tiles,
    /// Where the transformed image's top-left sits relative to the box's (whole pixels).
    offset: (i32, i32),
    size: (u32, u32),
}

/// Pointer and Enter/Escape handling while pasting.
pub fn interact(app: &mut PixelApp, ui: &egui::Ui, resp: &egui::Response, view: Rect, vp: Viewport, ssize: Vec2) {
    let Some(mut p) = app.ui.pasting.clone() else { return };
    let (enter, escape, shift) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape), i.modifiers.shift));
    if escape {
        app.cancel_paste();
        return;
    }
    if enter {
        app.apply_paste();
        return;
    }
    let origin = vp.sprite_rect(view, ssize).min;
    let to_sprite = |s: Pos2| (s - origin) / vp.zoom;
    let pointer = resp.hover_pos().or(resp.interact_pointer_pos()).map(to_sprite);

    if let Some(at) = pointer {
        let hit = p.hit(at, vp.zoom);
        if resp.hovered() && p.drag.is_none() {
            let icon = match hit {
                Hit::Inside => egui::CursorIcon::Move,
                Hit::Handle(0, _) => egui::CursorIcon::ResizeVertical,
                Hit::Handle(_, 0) => egui::CursorIcon::ResizeHorizontal,
                Hit::Handle(_, _) => egui::CursorIcon::ResizeNwSe,
                Hit::Rotate => egui::CursorIcon::Alias,
                Hit::Outside => egui::CursorIcon::Default,
            };
            ui.ctx().set_cursor_icon(icon);
        }
        // The press starts the drag (egui's own drag start comes a few points later).
        let pressed =
            resp.drag_started_by(egui::PointerButton::Primary) || (resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed()));
        if pressed && p.drag.is_none() {
            p.drag = match hit {
                Hit::Inside => Some(PasteDrag::Move { from: at, start: (p.x, p.y) }),
                Hit::Handle(hx, hy) => Some(PasteDrag::Handle { hx, hy, start: Box::new(Pasting { drag: None, ..p.clone() }) }),
                Hit::Rotate => Some(PasteDrag::Rotate { start_angle: p.angle, from_bearing: bearing(at - p.center()) }),
                Hit::Outside => {
                    // A click elsewhere applies the paste, as in the target app.
                    app.apply_paste();
                    return;
                }
            };
        }
        match p.drag.clone() {
            Some(PasteDrag::Move { from, start }) => {
                let d = at - from;
                p.x = (start.0 + d.x).round();
                p.y = (start.1 + d.y).round();
            }
            Some(PasteDrag::Handle { hx, hy, start }) => stretch(&mut p, &start, hx, hy, at, shift),
            Some(PasteDrag::Rotate { start_angle, from_bearing }) => {
                let mut a = start_angle + bearing(at - p.center()) - from_bearing;
                if shift {
                    a = (a / 15.0).round() * 15.0;
                }
                p.angle = a.rem_euclid(360.0);
            }
            None => {}
        }
    }
    if ui.input(|i| i.pointer.any_released()) {
        p.drag = None;
    }
    app.ui.pasting = Some(p);
}

/// Stretches the box by handle (`hx`, `hy`) to the pointer `at`, keeping the opposite side
/// (or corner) where it was. Sizes stay whole pixels, at least 1.
fn stretch(p: &mut Pasting, start: &Pasting, hx: i8, hy: i8, at: Vec2, keep_ratio: bool) {
    let l = start.local(at);
    let (fx, fy) = (f32::from(hx), f32::from(hy));
    // The fixed point, in the start box's frame.
    let anchor = vec2(-fx * start.w / 2.0, -fy * start.h / 2.0);
    let mut w = if hx != 0 { ((l.x - anchor.x) * fx).round().max(1.0) } else { start.w };
    let mut h = if hy != 0 { ((l.y - anchor.y) * fy).round().max(1.0) } else { start.h };
    if keep_ratio && start.w > 0.0 && start.h > 0.0 {
        let k = match (hx, hy) {
            (0, _) => h / start.h,
            (_, 0) => w / start.w,
            _ => (w / start.w).max(h / start.h),
        };
        w = (start.w * k).round().max(1.0);
        h = (start.h * k).round().max(1.0);
    }
    let center_local = anchor + vec2(fx * w / 2.0, fy * h / 2.0);
    let c = start.center() + rotate(center_local, start.angle);
    p.w = w;
    p.h = h;
    p.x = (c.x - w / 2.0).round();
    p.y = (c.y - h / 2.0).round();
}

/// Draws the floating pixels and the box with its handles.
pub fn draw(app: &mut PixelApp, ctx: &egui::Context, painter: &egui::Painter, view: Rect, vp: Viewport, ssize: Vec2, t: &Tokens) {
    let (Some(p), Some(clip)) = (app.ui.pasting.clone(), app.session.clipboard.as_ref()) else { return };
    let (x, y, w, h) = p.rounded();
    let key = (w, h, p.angle.to_bits());
    let cache = &mut app.paste_preview;
    if cache.key != Some(key) {
        match scale_rotate(&clip.image, w, h, p.angle) {
            Ok((img, _, off)) => {
                cache.tiles = crate::tiles::Tiles::upload(ctx, "paste", &img);
                cache.offset = (off.x, off.y);
                cache.size = (img.size().w, img.size().h);
            }
            Err(_) => cache.tiles = crate::tiles::Tiles::default(),
        }
        cache.key = Some(key);
    }
    let origin = vp.sprite_rect(view, ssize).min;
    let screen = |v: Vec2| origin + v * vp.zoom;
    let at = vec2((x + cache.offset.0) as f32, (y + cache.offset.1) as f32);
    let r = Rect::from_min_size(screen(at), vec2(cache.size.0 as f32, cache.size.1 as f32) * vp.zoom);
    cache.tiles.paint(painter, r);

    // The box: its outline, then the eight handles.
    let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)].map(|(hx, hy)| screen(p.handle(hx, hy)).to_vec2());
    let outline: Vec<Pos2> = corners.iter().map(|c| pos2(c.x, c.y)).collect();
    painter.add(egui::Shape::closed_line(outline.clone(), Stroke::new(size::UI_PX + 1.0, Color32::BLACK)));
    painter.add(egui::Shape::closed_line(outline, Stroke::new(1.0, t.accent)));
    let side = HANDLE_HIT;
    for (hx, hy) in HANDLES {
        let c = screen(p.handle(hx, hy));
        let hr = Rect::from_center_size(c, vec2(side, side));
        painter.rect_filled(hr, 0.0, Color32::WHITE);
        painter.rect_stroke(hr, 0.0, Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Inside);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_box_knows_its_handles_inside_and_rotate_zone() {
        let p = Pasting::new(10, 10, 4, 2);
        let z = 10.0;
        assert_eq!(p.hit(vec2(12.0, 11.0), z), Hit::Inside);
        assert_eq!(p.hit(vec2(14.0, 12.0), z), Hit::Handle(1, 1));
        assert_eq!(p.hit(vec2(12.0, 10.0), z), Hit::Handle(0, -1));
        assert_eq!(p.hit(vec2(15.5, 13.5), z), Hit::Rotate);
        assert_eq!(p.hit(vec2(30.0, 30.0), z), Hit::Outside);
    }

    #[test]
    fn stretching_keeps_the_opposite_corner() {
        let start = Pasting::new(10, 10, 4, 2);
        let mut p = start.clone();
        stretch(&mut p, &start, 1, 1, vec2(18.0, 16.0), false);
        assert_eq!(p.rounded(), (10, 10, 8, 6));
        // Shift keeps the 2:1 ratio.
        stretch(&mut p, &start, 1, 1, vec2(18.0, 11.0), true);
        assert_eq!(p.rounded(), (10, 10, 8, 4));
        // An edge handle changes one side only; the left edge pulls past nothing below 1.
        stretch(&mut p, &start, -1, 0, vec2(20.0, 11.0), false);
        assert_eq!(p.rounded().2, 1);
        assert_eq!(p.rounded().3, 2);
    }
}
