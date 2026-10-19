//! The UI icon set, drawn in code (original art; no image files). Each icon fills a square rect.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Tool(Tool),
    First,
    Previous,
    Play,
    Pause,
    Next,
    Last,
    Eye,
    EyeClosed,
    Lock,
    Home,
    /// A tab's close button.
    Close,
    /// A tab's close button while the sprite has unsaved changes.
    Unsaved,
}

/// A point at fractions (0–1) of `r`.
fn at(r: Rect, x: f32, y: f32) -> Pos2 {
    pos2(r.min.x + r.width() * x, r.min.y + r.height() * y)
}

pub fn paint(p: &Painter, r: Rect, icon: Icon, c: Color32) {
    let r = r.shrink(r.width() * 0.18);
    let w = (r.width() / 9.0).max(1.0);
    let s = Stroke::new(w, c);
    let tri = |pts: Vec<Pos2>| Shape::convex_polygon(pts, c, Stroke::NONE);
    match icon {
        Icon::Tool(Tool::Pencil) => {
            p.line_segment([at(r, 0.85, 0.15), at(r, 0.25, 0.75)], Stroke::new(w * 2.2, c));
            p.add(tri(vec![at(r, 0.25, 0.75), at(r, 0.1, 0.9), at(r, 0.32, 0.86)]));
        }
        Icon::Tool(Tool::Eraser) => {
            p.add(Shape::convex_polygon(vec![at(r, 0.55, 0.1), at(r, 0.9, 0.45), at(r, 0.45, 0.9), at(r, 0.1, 0.55)], Color32::TRANSPARENT, s));
            p.add(tri(vec![at(r, 0.55, 0.1), at(r, 0.9, 0.45), at(r, 0.68, 0.67), at(r, 0.33, 0.32)]));
        }
        Icon::Tool(Tool::Eyedropper) => {
            p.line_segment([at(r, 0.15, 0.85), at(r, 0.6, 0.4)], Stroke::new(w * 1.5, c));
            p.circle_filled(at(r, 0.72, 0.28), r.width() * 0.17, c);
        }
        Icon::Tool(Tool::Zoom) => {
            p.circle_stroke(at(r, 0.42, 0.42), r.width() * 0.3, s);
            p.line_segment([at(r, 0.64, 0.64), at(r, 0.92, 0.92)], Stroke::new(w * 2.0, c));
        }
        Icon::Tool(Tool::Hand) => {
            // Palm, four fingers and a thumb.
            p.rect_filled(Rect::from_min_max(at(r, 0.25, 0.5), at(r, 0.78, 0.95)), r.width() * 0.15, c);
            for (i, top) in [0.12, 0.02, 0.06, 0.18].iter().enumerate() {
                let x = 0.31 + i as f32 * 0.14;
                p.line_segment([at(r, x, *top), at(r, x, 0.6)], Stroke::new(w * 1.25, c));
            }
            p.line_segment([at(r, 0.28, 0.72), at(r, 0.06, 0.48)], Stroke::new(w * 1.25, c));
        }
        Icon::Tool(Tool::Move) => {
            p.line_segment([at(r, 0.5, 0.05), at(r, 0.5, 0.95)], s);
            p.line_segment([at(r, 0.05, 0.5), at(r, 0.95, 0.5)], s);
            p.add(tri(vec![at(r, 0.5, 0.0), at(r, 0.65, 0.2), at(r, 0.35, 0.2)]));
            p.add(tri(vec![at(r, 0.5, 1.0), at(r, 0.35, 0.8), at(r, 0.65, 0.8)]));
            p.add(tri(vec![at(r, 0.0, 0.5), at(r, 0.2, 0.35), at(r, 0.2, 0.65)]));
            p.add(tri(vec![at(r, 1.0, 0.5), at(r, 0.8, 0.65), at(r, 0.8, 0.35)]));
        }
        Icon::Tool(Tool::Bucket) => {
            p.add(Shape::convex_polygon(vec![at(r, 0.45, 0.1), at(r, 0.85, 0.5), at(r, 0.5, 0.85), at(r, 0.1, 0.45)], Color32::TRANSPARENT, s));
            p.circle_filled(at(r, 0.85, 0.8), r.width() * 0.12, c);
        }
        Icon::Tool(Tool::Line) => {
            p.line_segment([at(r, 0.1, 0.9), at(r, 0.9, 0.1)], Stroke::new(w * 1.5, c));
        }
        Icon::Tool(Tool::Rectangle) => {
            p.rect_stroke(Rect::from_min_max(at(r, 0.1, 0.2), at(r, 0.9, 0.8)), 0.0, s, egui::StrokeKind::Inside);
        }
        Icon::Tool(Tool::Ellipse) => {
            p.circle_stroke(r.center(), r.width() * 0.42, s);
        }
        Icon::First => {
            p.rect_filled(Rect::from_min_max(at(r, 0.15, 0.15), at(r, 0.27, 0.85)), 0.0, c);
            p.add(tri(vec![at(r, 0.3, 0.5), at(r, 0.85, 0.15), at(r, 0.85, 0.85)]));
        }
        Icon::Previous => {
            p.add(tri(vec![at(r, 0.2, 0.5), at(r, 0.75, 0.15), at(r, 0.75, 0.85)]));
        }
        Icon::Play => {
            p.add(tri(vec![at(r, 0.25, 0.12), at(r, 0.85, 0.5), at(r, 0.25, 0.88)]));
        }
        Icon::Pause => {
            p.rect_filled(Rect::from_min_max(at(r, 0.22, 0.15), at(r, 0.42, 0.85)), 0.0, c);
            p.rect_filled(Rect::from_min_max(at(r, 0.58, 0.15), at(r, 0.78, 0.85)), 0.0, c);
        }
        Icon::Next => {
            p.add(tri(vec![at(r, 0.25, 0.15), at(r, 0.8, 0.5), at(r, 0.25, 0.85)]));
        }
        Icon::Last => {
            p.add(tri(vec![at(r, 0.15, 0.15), at(r, 0.7, 0.5), at(r, 0.15, 0.85)]));
            p.rect_filled(Rect::from_min_max(at(r, 0.73, 0.15), at(r, 0.85, 0.85)), 0.0, c);
        }
        Icon::Eye | Icon::EyeClosed => {
            let lid = [at(r, 0.0, 0.5), at(r, 0.25, 0.25), at(r, 0.75, 0.25), at(r, 1.0, 0.5), at(r, 0.75, 0.75), at(r, 0.25, 0.75)];
            p.add(Shape::closed_line(lid.to_vec(), s));
            if icon == Icon::Eye {
                p.circle_filled(r.center(), r.width() * 0.16, c);
            } else {
                p.line_segment([at(r, 0.1, 0.9), at(r, 0.9, 0.1)], s);
            }
        }
        Icon::Home => {
            p.add(tri(vec![at(r, 0.5, 0.05), at(r, 0.98, 0.5), at(r, 0.02, 0.5)]));
            p.rect_filled(Rect::from_min_max(at(r, 0.18, 0.48), at(r, 0.82, 0.95)), 0.0, c);
        }
        Icon::Close => {
            p.line_segment([at(r, 0.15, 0.15), at(r, 0.85, 0.85)], s);
            p.line_segment([at(r, 0.85, 0.15), at(r, 0.15, 0.85)], s);
        }
        Icon::Unsaved => {
            p.circle_filled(r.center(), r.width() * 0.38, c);
        }
        Icon::Lock => {
            p.rect_filled(Rect::from_min_max(at(r, 0.2, 0.45), at(r, 0.8, 0.9)), 1.0, c);
            p.rect_stroke(Rect::from_min_max(at(r, 0.32, 0.12), at(r, 0.68, 0.5)), r.width() * 0.15, s, egui::StrokeKind::Inside);
        }
    }
}

/// A square icon button; `selected` paints it in the accent colour.
pub fn button(ui: &mut egui::Ui, side: f32, icon: Icon, selected: bool, t: &crate::theme::Tokens) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(side, side), egui::Sense::click());
    let bg = if selected {
        t.accent
    } else if resp.hovered() {
        t.button_hover
    } else {
        t.button
    };
    ui.painter().rect_filled(r.shrink(1.0), 3.0, bg);
    paint(ui.painter(), r, icon, if selected { t.on_accent } else { t.text });
    resp
}
