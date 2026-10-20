//! The timeline at the bottom: transport buttons, then a grid of layers (top layer first) against
//! frames. The left column holds each layer's eye, lock and name; a dot in a cell marks a cel.
//! Clicks go through engine commands (`timeline.setCel`, `layer.toggleVisible`, `frame.goTo*`).

use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::icons::{self, Icon};
use crate::theme::{Tokens, size};
use crate::{PixelApp, UiAction};

const EYE_W: f32 = 22.0;
const LOCK_W: f32 = 22.0;

struct Row {
    name: String,
    visible: bool,
    locked: bool,
    /// Which frames have a cel on this layer.
    cels: Vec<bool>,
}

enum Click {
    Eye(usize),
    Layer(usize),
    Frame(usize),
    Cel(usize, usize),
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let Some(d) = app.session.active() else { return };
    let frames = d.sprite.frames.len();
    let rows: Vec<Row> = d
        .sprite
        .layers
        .iter()
        .enumerate()
        .map(|(li, l)| Row {
            name: l.name.clone(),
            visible: l.visible,
            locked: l.locked,
            cels: (0..frames).map(|fi| d.sprite.cel(li, fi).is_some()).collect(),
        })
        .collect();
    let (cur_layer, cur_frame) = (d.layer, d.frame);
    let playing = app.ui.playing;
    let mut click = None;
    let mut transport = None;

    egui::Panel::bottom("timeline")
        .default_size(size::TIMELINE_H)
        .min_size(size::TIMELINE_ROW_H * 3.0)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.panel_alt).inner_margin(4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let side = size::TIMELINE_ROW_H;
                let buttons = [
                    (Icon::First, "First Frame (Home)", "frame.goToFirst"),
                    (Icon::Previous, "Previous Frame (Left)", "frame.goToPrevious"),
                    (if playing { Icon::Pause } else { Icon::Play }, "Play Animation (Enter)", ""),
                    (Icon::Next, "Next Frame (Right)", "frame.goToNext"),
                    (Icon::Last, "Last Frame (End)", "frame.goToLast"),
                ];
                for (icon, tip, id) in buttons {
                    if icons::button(ui, side, icon, icon == Icon::Pause, t).on_hover_text(tip).clicked() {
                        transport = Some(id);
                    }
                }
            });
            ui.add_space(2.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let row_h = size::TIMELINE_ROW_H;
                let grid = vec2(size::TIMELINE_LAYER_W + frames as f32 * size::TIMELINE_CEL_W, (rows.len() + 1) as f32 * row_h);
                let (area, resp) = ui.allocate_exact_size(grid, Sense::click());
                let p = ui.painter();
                let x0 = area.min.x + size::TIMELINE_LAYER_W;
                let cel_rect = |row: usize, f: usize| {
                    Rect::from_min_size(
                        pos2(x0 + f as f32 * size::TIMELINE_CEL_W, area.min.y + row as f32 * row_h),
                        vec2(size::TIMELINE_CEL_W, row_h),
                    )
                };

                // Header: frame numbers, the current one highlighted.
                for f in 0..frames {
                    let r = cel_rect(0, f);
                    if f == cur_frame {
                        p.rect_filled(r.shrink(1.0), 2.0, t.accent);
                    }
                    p.text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        (f + 1).to_string(),
                        FontId::proportional(crate::pixel_font::SIZE),
                        if f == cur_frame { t.on_accent } else { t.text_dim },
                    );
                }

                // Layer rows, top layer first.
                for (row, li) in (0..rows.len()).rev().enumerate() {
                    let Some(layer) = rows.get(li) else { continue };
                    let y = area.min.y + (row + 1) as f32 * row_h;
                    let line = Rect::from_min_size(pos2(area.min.x, y), vec2(grid.x, row_h));
                    if li == cur_layer {
                        p.rect_filled(line, 0.0, t.row_active);
                    }
                    let eye = Rect::from_min_size(pos2(area.min.x, y), vec2(EYE_W, row_h));
                    icons::paint(p, eye, if layer.visible { Icon::Eye } else { Icon::EyeClosed }, t.text);
                    if layer.locked {
                        icons::paint(p, Rect::from_min_size(pos2(area.min.x + EYE_W, y), vec2(LOCK_W, row_h)), Icon::Lock, t.text);
                    }
                    let name_color = if layer.visible { t.text } else { t.text_dim };
                    p.text(
                        pos2(area.min.x + EYE_W + LOCK_W + 4.0, y + row_h / 2.0),
                        Align2::LEFT_CENTER,
                        &layer.name,
                        FontId::proportional(crate::pixel_font::SIZE),
                        name_color,
                    );
                    for f in 0..frames {
                        let r = cel_rect(row + 1, f);
                        if li == cur_layer && f == cur_frame {
                            p.rect_filled(r.shrink(1.0), 2.0, t.cel_active);
                        }
                        if layer.cels.get(f).copied().unwrap_or(false) {
                            p.circle_filled(r.center(), 3.5, t.text);
                        }
                    }
                    p.line_segment([pos2(area.min.x, y + row_h), pos2(area.max.x, y + row_h)], Stroke::new(1.0, t.border));
                }
                p.line_segment([pos2(x0, area.min.y), pos2(x0, area.max.y)], Stroke::new(1.0, t.border));

                if resp.clicked()
                    && let Some(pos) = resp.interact_pointer_pos()
                {
                    click = hit(pos, area, rows.len(), frames);
                }
            });
        });

    match transport {
        Some("") => app.act(UiAction::TogglePlay),
        Some(id) => {
            app.run(id, &json!({}));
        }
        None => {}
    }
    match click {
        Some(Click::Eye(l)) => {
            app.run("layer.toggleVisible", &json!({ "layer": l }));
        }
        Some(Click::Layer(l)) => {
            app.run("timeline.setCel", &json!({ "layer": l }));
        }
        Some(Click::Frame(f)) => {
            app.run("timeline.setCel", &json!({ "frame": f }));
        }
        Some(Click::Cel(l, f)) => {
            app.run("timeline.setCel", &json!({ "layer": l, "frame": f }));
        }
        None => {}
    }
}

/// What a click at `pos` on the grid hit.
fn hit(pos: Pos2, area: Rect, layers: usize, frames: usize) -> Option<Click> {
    let row = ((pos.y - area.min.y) / size::TIMELINE_ROW_H).floor();
    let x = pos.x - area.min.x;
    let frame = ((x - size::TIMELINE_LAYER_W) / size::TIMELINE_CEL_W).floor();
    let frame = (x >= size::TIMELINE_LAYER_W && frame >= 0.0 && (frame as usize) < frames).then_some(frame as usize);
    if row < 1.0 {
        return frame.map(Click::Frame);
    }
    // Rows are drawn top layer first.
    let layer = layers.checked_sub(row as usize)?;
    Some(match frame {
        Some(f) => Click::Cel(layer, f),
        None if x < EYE_W => Click::Eye(layer),
        None => Click::Layer(layer),
    })
}
