//! A compact colour picker that fits the colour bar: a saturation/value square, a hue strip and
//! an alpha strip, all dragged directly. The hue is kept by the caller so greys don't lose it.

use egui::ecolor::Hsva;
use egui::{Color32, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, pos2, vec2};
use pixelcraft_engine::color::Rgba8;

use crate::theme::Tokens;

const STRIP_H: f32 = 12.0;

fn to_hsva(c: Rgba8) -> Hsva {
    Hsva::from(Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a))
}

fn from_hsva(h: Hsva) -> Rgba8 {
    let [r, g, b, a] = Color32::from(h).to_srgba_unmultiplied();
    Rgba8::new(r, g, b, a)
}

fn hue_color(h: f32) -> Color32 {
    Color32::from(Hsva::new(h, 1.0, 1.0, 1.0))
}

/// A quad whose four corners have their own colours (top-left, top-right, bottom-left, bottom-right).
fn gradient_quad(r: Rect, tl: Color32, tr: Color32, bl: Color32, br: Color32) -> Mesh {
    let mut m = Mesh::default();
    for (p, c) in [(r.left_top(), tl), (r.right_top(), tr), (r.left_bottom(), bl), (r.right_bottom(), br)] {
        m.colored_vertex(p, c);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(1, 3, 2);
    m
}

/// The fraction (0–1) of `r` the pointer is at, while the area is pressed or dragged.
fn drag_fraction(resp: &egui::Response, r: Rect) -> Option<Pos2> {
    let p = resp.interact_pointer_pos().filter(|_| resp.is_pointer_button_down_on())?;
    Some(pos2(((p.x - r.min.x) / r.width()).clamp(0.0, 1.0), ((p.y - r.min.y) / r.height()).clamp(0.0, 1.0)))
}

fn marker(p: &egui::Painter, at: Pos2) {
    p.circle_stroke(at, 4.0, Stroke::new(2.0, Color32::BLACK));
    p.circle_stroke(at, 4.0, Stroke::new(1.0, Color32::WHITE));
}

/// Shows the picker `width` points wide; returns true when `color` changed.
pub fn show(ui: &mut egui::Ui, color: &mut Rgba8, hue: &mut f32, width: f32, t: &Tokens) -> bool {
    let mut hsva = to_hsva(*color);
    if hsva.s > 0.0 && hsva.v > 0.0 {
        *hue = hsva.h;
    }
    hsva.h = *hue;
    let mut changed = false;

    // Saturation (→) and value (↑).
    let (sv, resp) = ui.allocate_exact_size(vec2(width, width * 0.8), Sense::click_and_drag());
    let p = ui.painter();
    p.add(Shape::mesh(gradient_quad(sv, Color32::WHITE, hue_color(*hue), Color32::BLACK, Color32::BLACK)));
    p.rect_stroke(sv, 0.0, Stroke::new(1.0, t.border), StrokeKind::Outside);
    if let Some(f) = drag_fraction(&resp, sv) {
        hsva.s = f.x;
        hsva.v = 1.0 - f.y;
        changed = true;
    }
    marker(p, pos2(sv.min.x + hsva.s * sv.width(), sv.min.y + (1.0 - hsva.v) * sv.height()));

    // Hue.
    let (hr, resp) = ui.allocate_exact_size(vec2(width, STRIP_H), Sense::click_and_drag());
    let p = ui.painter();
    const SEGMENTS: usize = 6;
    for i in 0..SEGMENTS {
        let (h0, h1) = (i as f32 / SEGMENTS as f32, (i + 1) as f32 / SEGMENTS as f32);
        let seg = Rect::from_min_max(pos2(hr.min.x + h0 * hr.width(), hr.min.y), pos2(hr.min.x + h1 * hr.width(), hr.max.y));
        p.add(Shape::mesh(gradient_quad(seg, hue_color(h0), hue_color(h1), hue_color(h0), hue_color(h1))));
    }
    if let Some(f) = drag_fraction(&resp, hr) {
        *hue = f.x.min(0.999);
        hsva.h = *hue;
        changed = true;
    }
    let x = hr.min.x + *hue * hr.width();
    p.rect_stroke(
        Rect::from_center_size(pos2(x, hr.center().y), vec2(4.0, STRIP_H + 2.0)),
        1.0,
        Stroke::new(1.0, Color32::WHITE),
        StrokeKind::Outside,
    );

    // Alpha over a checkerboard.
    let (ar, resp) = ui.allocate_exact_size(vec2(width, STRIP_H), Sense::click_and_drag());
    let p = ui.painter();
    let sq = STRIP_H / 2.0;
    let mut x = 0.0;
    let mut i = 0;
    while x < ar.width() {
        for row in 0..2 {
            let c = if (i + row) % 2 == 0 { t.checker_a } else { t.checker_b };
            let r = Rect::from_min_size(ar.min + vec2(x, row as f32 * sq), vec2(sq.min(ar.width() - x), sq));
            p.rect_filled(r, 0.0, c);
        }
        x += sq;
        i += 1;
    }
    let opaque = Color32::from(Hsva { a: 1.0, ..hsva });
    p.add(Shape::mesh(gradient_quad(ar, Color32::TRANSPARENT, opaque, Color32::TRANSPARENT, opaque)));
    if let Some(f) = drag_fraction(&resp, ar) {
        hsva.a = f.x;
        changed = true;
    }
    let x = ar.min.x + hsva.a * ar.width();
    p.rect_stroke(
        Rect::from_center_size(pos2(x, ar.center().y), vec2(4.0, STRIP_H + 2.0)),
        1.0,
        Stroke::new(1.0, Color32::WHITE),
        StrokeKind::Outside,
    );

    if changed {
        *color = from_hsva(hsva);
    }
    changed
}
