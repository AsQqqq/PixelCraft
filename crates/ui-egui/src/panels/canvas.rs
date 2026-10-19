//! The sprite editor: the current frame, composited by the engine, drawn over a transparency
//! checkerboard at the viewport's zoom and pan.
//!
//! Navigation follows the target app: the wheel zooms one ladder step per notch around the
//! pointer (Shift+wheel scrolls sideways), Space or the Hand tool or the middle button pans,
//! the Zoom tool zooms in on click and out on right click. Pencil and eraser strokes go through
//! the `pixel.stroke` command; left click paints the foreground colour, right click the background.

use std::sync::Arc;

use egui::{Event, MouseWheelUnit, PointerButton, Pos2, Rect, Vec2, pos2, vec2};
use pixelcraft_engine::color::Rgba8;
use pixelcraft_engine::compose::render_frame;
use pixelcraft_engine::doc::Sprite;
use pixelcraft_engine::geom::Point;
use pixelcraft_engine::raster::Image;
use serde_json::json;

use crate::theme::{Tokens, size};
use crate::zoom::{self, Viewport};
use crate::{PixelApp, Tool};

/// Points of trackpad scrolling that make one zoom step.
const POINTS_PER_STEP: f32 = 50.0;

/// What the canvas remembers between frames.
#[derive(Default)]
pub struct CanvasCache {
    /// The sprite and frame the texture shows; holding the `Arc` keeps its address from being reused.
    key: Option<(Arc<Sprite>, usize)>,
    texture: Option<egui::TextureHandle>,
    /// The composited frame, for the eyedropper.
    image: Option<Image>,
    last_pixel: Option<(i32, i32)>,
    /// The editor area and the pointer in it, for keyboard zoom.
    view: Option<Rect>,
    pointer: Option<Pos2>,
    wheel: f32,
}

fn active_size(app: &PixelApp) -> Option<(u64, Vec2)> {
    app.session.active().map(|d| (d.uid, vec2(d.sprite.size.w as f32, d.sprite.size.h as f32)))
}

pub fn viewport_mut(app: &mut PixelApp) -> Option<&mut Viewport> {
    let (uid, _) = active_size(app)?;
    app.ui.viewports.get_mut(&uid)
}

/// The point keyboard zoom keeps fixed: the pointer when it is over the editor, else the centre.
fn anchor(app: &PixelApp, view: Rect) -> Pos2 {
    app.canvas.pointer.filter(|p| view.contains(*p)).unwrap_or(view.center())
}

pub fn zoom_step(app: &mut PixelApp, inward: bool) {
    let Some(vp) = viewport_mut(app) else { return };
    let z = if inward { zoom::step_in(vp.zoom) } else { zoom::step_out(vp.zoom) };
    zoom_set(app, z);
}

pub fn zoom_set(app: &mut PixelApp, z: f32) {
    let (Some((uid, sprite)), Some(view)) = (active_size(app), app.canvas.view) else { return };
    let a = anchor(app, view);
    if let Some(vp) = app.ui.viewports.get_mut(&uid) {
        vp.zoom_to(z, view, sprite, a);
    }
}

pub fn fit(app: &mut PixelApp) {
    let (Some((uid, sprite)), Some(view)) = (active_size(app), app.canvas.view) else { return };
    if let Some(vp) = app.ui.viewports.get_mut(&uid) {
        vp.fit(view, sprite);
    }
}

/// Wheel events this frame: unit, delta (the direction content moves) and modifiers.
fn wheel_events(ui: &egui::Ui) -> Vec<(MouseWheelUnit, Vec2, egui::Modifiers)> {
    ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                Event::MouseWheel { unit, delta, modifiers, .. } => Some((*unit, *delta, *modifiers)),
                _ => None,
            })
            .collect()
    })
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let view = ui.max_rect();
    app.canvas.view = Some(view);
    let Some(d) = app.session.active() else {
        app.ui.hover_pixel = None;
        ui.centered_and_justified(|ui| ui.label(egui::RichText::new("File → New to start a sprite").color(t.text_dim)));
        return;
    };
    let (uid, sprite, frame) = (d.uid, Arc::clone(&d.sprite), d.frame);
    let ssize = vec2(sprite.size.w as f32, sprite.size.h as f32);

    // A new document opens at 100 %, centred, like the target app (Cmd+0 fits it).
    let mut vp = *app.ui.viewports.entry(uid).or_default();

    // Re-render only when the sprite or the frame changed.
    let fresh = app.canvas.key.as_ref().is_some_and(|(s, f)| Arc::ptr_eq(s, &sprite) && *f == frame);
    if !fresh && let Ok(img) = render_frame(&sprite, frame) {
        let px = [img.size().w as usize, img.size().h as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(px, img.bytes());
        // Sharp pixels when zoomed in, filtered when zoomed out.
        let options = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Linear,
            ..egui::TextureOptions::NEAREST
        };
        app.canvas.texture = Some(ui.ctx().load_texture("canvas", color, options));
        app.canvas.image = Some(img);
        app.canvas.key = Some((Arc::clone(&sprite), frame));
    }

    let resp = ui.allocate_rect(view, egui::Sense::click_and_drag());
    let pointer = resp.hover_pos().or(resp.interact_pointer_pos());
    app.canvas.pointer = pointer;
    let inside = |(x, y): (i32, i32)| x >= 0 && y >= 0 && (x as f32) < ssize.x && (y as f32) < ssize.y;
    app.ui.hover_pixel = resp.hover_pos().map(|p| vp.pixel_at(view, ssize, p)).filter(|&px| inside(px));

    // Quick tools: Space = hand, Alt = eyedropper, while held.
    let (space, alt) = ui.input(|i| (i.key_down(egui::Key::Space), i.modifiers.alt));
    let tool = if space {
        Tool::Hand
    } else if alt {
        Tool::Eyedropper
    } else {
        app.ui.tool
    };
    let panning = resp.dragged_by(PointerButton::Middle) || (tool == Tool::Hand && resp.dragged_by(PointerButton::Primary));
    if panning {
        vp.pan += resp.drag_delta();
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if tool == Tool::Hand && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    // Wheel: zoom around the pointer; Shift scrolls sideways.
    if let Some(p) = resp.hover_pos() {
        for (unit, delta, m) in wheel_events(ui) {
            if m.shift {
                let scale = if unit == MouseWheelUnit::Point { 1.0 } else { 40.0 };
                vp.pan.x += (delta.x + delta.y) * scale;
                continue;
            }
            if m.command || m.alt {
                continue; // brush size and palette index: not yet
            }
            let steps = match unit {
                MouseWheelUnit::Point => {
                    app.canvas.wheel += delta.y;
                    let n = (app.canvas.wheel / POINTS_PER_STEP).trunc();
                    app.canvas.wheel -= n * POINTS_PER_STEP;
                    n as i32
                }
                _ => delta.y.signum() as i32,
            };
            // By default content moving down (wheel up / swipe down) zooms in; the
            // "Invert Wheel Zoom" setting flips it.
            let inward = (steps > 0) != app.ui.invert_wheel_zoom;
            for _ in 0..steps.unsigned_abs() {
                let z = if inward { zoom::step_in(vp.zoom) } else { zoom::step_out(vp.zoom) };
                vp.zoom_to(z, view, ssize, p);
            }
        }
    }

    // Tools.
    let (primary, secondary) = ui.input(|i| (i.pointer.primary_down(), i.pointer.secondary_down()));
    let down_on = resp.is_pointer_button_down_on() && (primary || secondary) && !panning;
    match tool {
        Tool::Pencil | Tool::Eraser if down_on => {
            if let Some(p) = pointer {
                let px = vp.pixel_at(view, ssize, p);
                let from = app.canvas.last_pixel.unwrap_or(px);
                if app.canvas.last_pixel != Some(px) {
                    let color = match tool {
                        Tool::Eraser => Rgba8::TRANSPARENT,
                        _ if secondary => app.ui.bg,
                        _ => app.ui.fg,
                    };
                    let index = if tool == Tool::Eraser { 0 } else { sprite.palette.colors.iter().position(|&c| c == color).unwrap_or(1) };
                    app.run("pixel.stroke", &json!({ "x0": from.0, "y0": from.1, "x1": px.0, "y1": px.1, "color": color.to_hex(), "index": index }));
                    app.canvas.last_pixel = Some(px);
                }
            }
        }
        Tool::Zoom if resp.clicked() || resp.secondary_clicked() => {
            if let Some(p) = pointer {
                let z = if resp.clicked() { zoom::step_in(vp.zoom) } else { zoom::step_out(vp.zoom) };
                vp.zoom_to(z, view, ssize, p);
            }
        }
        Tool::Eyedropper if resp.clicked() || resp.secondary_clicked() => {
            if let (Some((x, y)), Some(img)) = (app.ui.hover_pixel, &app.canvas.image) {
                let c = img.rgba(Point::new(x, y), &sprite.palette);
                if resp.clicked() {
                    app.ui.fg = c;
                } else {
                    app.ui.bg = c;
                }
            }
        }
        other if !other.works() && resp.clicked() => {
            app.status = Some(format!("The {} tool is not implemented yet", other.label()));
        }
        _ => {}
    }
    if !down_on {
        app.canvas.last_pixel = None;
    }
    app.ui.viewports.insert(uid, vp);

    paint(ui, view, vp, ssize, app.canvas.texture.as_ref(), t);
}

fn paint(ui: &egui::Ui, view: Rect, vp: Viewport, ssize: Vec2, texture: Option<&egui::TextureHandle>, t: &Tokens) {
    let painter = ui.painter_at(view);
    painter.rect_filled(view, 0.0, t.canvas_bg);
    let r = vp.sprite_rect(view, ssize);
    let visible = r.intersect(view);
    if visible.is_positive() {
        // Checkerboard squares of 16×16 sprite pixels, so they scale with the zoom; doubled while
        // smaller than CHECKER_MIN points. Only the visible ones are drawn.
        let mut s = size::CHECKER_SPRITE_PX * vp.zoom;
        while s < size::CHECKER_MIN {
            s *= 2.0;
        }
        let first = ((visible.min - r.min) / s).floor();
        let last = ((visible.max - r.min) / s).ceil();
        for y in first.y as i64..last.y as i64 {
            for x in first.x as i64..last.x as i64 {
                let c = if (x + y) % 2 == 0 { t.checker_a } else { t.checker_b };
                let sq = Rect::from_min_size(r.min + vec2(x as f32 * s, y as f32 * s), vec2(s, s)).intersect(r);
                painter.rect_filled(sq, 0.0, c);
            }
        }
        if let Some(tex) = texture {
            painter.image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        }
    }
    painter.rect_stroke(r, 0.0, egui::Stroke::new(1.0, t.canvas_border), egui::StrokeKind::Outside);
}
