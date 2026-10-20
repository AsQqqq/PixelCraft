//! The floating Preview window (F7), as the target app has it: the whole sprite on the editor
//! background, with three buttons in its title bar — fit, play/stop, close. ▶ (or Shift+Enter)
//! plays the animation **in the window**, looping over every frame at each frame's duration,
//! while the editor stays on its frame; ■ stops and shows the editor's frame again. The wheel
//! zooms the preview on its own; dragging pans it; the fit button fits it again.

use std::sync::Arc;

use egui::{Rect, Sense, pos2, vec2};
use pixelcraft_engine::compose::render_frame;
use pixelcraft_engine::doc::Sprite;

use crate::icons::{self, Icon};
use crate::theme::{self, Tokens, size};
use crate::{PixelApp, UiAction, zoom};

/// The texture of the frame the preview shows (separate from the editor's).
#[derive(Default)]
pub struct PreviewCache {
    key: Option<(Arc<Sprite>, usize)>,
    texture: Option<egui::TextureHandle>,
}

/// Advances preview playback by `dt` seconds: the next frame once the current one's duration
/// has passed, wrapping at the end.
pub fn advance(p: &mut crate::state::PreviewState, sprite: &Sprite, dt: f32) {
    let n = sprite.frames.len().max(1);
    p.frame = p.frame.min(n - 1);
    p.elapsed_ms += dt * 1000.0;
    // Several frames may pass in one long step; bound the loop by the frame count.
    for _ in 0..n {
        let duration = sprite.frames.get(p.frame).map_or(100, |f| f.duration_ms).max(1) as f32;
        if p.elapsed_ms < duration {
            break;
        }
        p.elapsed_ms -= duration;
        p.frame = (p.frame + 1) % n;
    }
}

pub fn show(app: &mut PixelApp, ctx: &egui::Context, t: &Tokens) {
    let Some(doc) = app.active_doc().and_then(|i| app.session.docs.get(i)) else { return };
    let sprite = Arc::clone(&doc.sprite);
    let editor_frame = doc.frame;
    let ssize = vec2(sprite.size.w as f32, sprite.size.h as f32);

    if app.ui.preview.playing {
        advance(&mut app.ui.preview, &sprite, ctx.input(|i| i.stable_dt));
        ctx.request_repaint();
    }
    let frame = if app.ui.preview.playing { app.ui.preview.frame } else { editor_frame };
    let fresh = app.preview.key.as_ref().is_some_and(|(s, f)| Arc::ptr_eq(s, &sprite) && *f == frame);
    if !fresh && let Ok(img) = render_frame(&sprite, frame) {
        let px = [img.size().w as usize, img.size().h as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(px, img.bytes());
        let options = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Linear,
            ..egui::TextureOptions::NEAREST
        };
        app.preview.texture = Some(ctx.load_texture("preview", color, options));
        app.preview.key = Some((Arc::clone(&sprite), frame));
    }
    let Some(tex) = app.preview.texture.as_ref().map(egui::TextureHandle::id) else { return };

    let screen = ctx.content_rect();
    let mut actions = Vec::new();
    let mut view = app.ui.preview.clone();
    let mut play_rect = None;
    egui::Window::new("Preview")
        .title_bar(false)
        .default_size(vec2(220.0, 130.0))
        .min_size(vec2(120.0, 80.0))
        .default_pos(pos2(screen.max.x - 520.0, screen.max.y - 280.0))
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(size::UI_PX, t.border)).inner_margin(size::UI_PX as i8 * 2))
        .show(ctx, |ui| {
            // Title row: the name, then fit, play/stop, close.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = size::UI_PX;
                ui.label("Preview");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let side = 18.0;
                    if icons::button(ui, side, Icon::Close, false, t).on_hover_text("Close (F7)").clicked() {
                        actions.push(UiAction::TogglePreview);
                    }
                    let (icon, tip) = if view.playing { (Icon::Stop, "Stop") } else { (Icon::Play, "Play (Shift+Enter)") };
                    let play = icons::button(ui, side, icon, view.playing, t);
                    play_rect = Some(play.rect);
                    if play.on_hover_text(tip).clicked() {
                        actions.push(UiAction::TogglePreviewPlay);
                    }
                    if icons::button(ui, side, Icon::Fit, view.zoom.is_none(), t).on_hover_text("Fit").clicked() {
                        view.zoom = None;
                        view.pan = egui::Vec2::ZERO;
                    }
                });
            });
            let (area, resp) = ui.allocate_exact_size(ui.available_size().max(vec2(80.0, 50.0)), Sense::click_and_drag());
            let p = ui.painter_at(area);
            theme::sunken(&p, area, t);
            let inner = area.shrink(size::UI_PX * 2.0);
            p.rect_filled(inner, 0.0, t.canvas_bg);
            let fit = (inner.width() / ssize.x).min(inner.height() / ssize.y).max(zoom::MIN);
            let k = view.zoom.unwrap_or(fit);
            // The wheel zooms the preview along the editor's ladder; dragging pans it.
            if resp.hovered() {
                // One ladder step per wheel event, same direction rule as the editor.
                let steps: Vec<f32> = ui.input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| if let egui::Event::MouseWheel { delta, .. } = e { (delta.y != 0.0).then_some(delta.y) } else { None })
                        .collect()
                });
                for dy in steps {
                    let inward = (dy > 0.0) != app.ui.prefs.invert_wheel_zoom;
                    let z = view.zoom.unwrap_or(k);
                    view.zoom = Some(if inward { zoom::step_in(z) } else { zoom::step_out(z) });
                }
            }
            if resp.dragged() {
                view.pan += resp.drag_delta();
                view.zoom.get_or_insert(k);
            }
            let k = view.zoom.unwrap_or(fit);
            let r = Rect::from_center_size(inner.center() + view.pan, ssize * k);
            p.rect_filled(r, 0.0, t.checker_a);
            let half = Rect::from_min_size(r.min, r.size() / 2.0);
            p.rect_filled(half, 0.0, t.checker_b);
            p.rect_filled(half.translate(r.size() / 2.0), 0.0, t.checker_b);
            p.image(tex, r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        });
    app.ui.preview = view;
    app.hit.preview_play = play_rect;
    for a in actions {
        app.act(a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PreviewState;
    use pixelcraft_engine::color::ColorMode;
    use pixelcraft_engine::doc::Frame;
    use pixelcraft_engine::geom::Size;

    #[test]
    fn playback_steps_by_each_frames_duration_and_loops() {
        let mut s = Sprite::new(Size::new(4, 4), ColorMode::Rgba).unwrap();
        s.frames = vec![Frame { duration_ms: 100 }, Frame { duration_ms: 300 }, Frame { duration_ms: 100 }];
        let mut p = PreviewState::default();
        advance(&mut p, &s, 0.05);
        assert_eq!(p.frame, 0);
        advance(&mut p, &s, 0.06);
        assert_eq!(p.frame, 1);
        advance(&mut p, &s, 0.25);
        assert_eq!(p.frame, 1, "frame 2 lasts 300 ms");
        advance(&mut p, &s, 0.1);
        assert_eq!(p.frame, 2);
        advance(&mut p, &s, 0.1);
        assert_eq!(p.frame, 0, "wraps to the first frame");
        advance(&mut p, &s, 1000.0);
        assert!(p.frame < 3);
    }
}
