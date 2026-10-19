//! The Home tab, shown at launch (`plan/aseprite/lifecycle.md` §1): links to start work, then
//! recent files and recent folders across the full width. (No news pane, unlike the target app.)

use egui::{Color32, Rect, RichText, Sense, pos2, vec2};

use crate::theme::Tokens;
use crate::{PixelApp, UiAction};

/// The app mark: a small pixel heart drawn in code (the same art as the app icon).
fn mark(ui: &mut egui::Ui, t: &Tokens) {
    const ROWS: [&str; 9] =
        ["..##..##..", ".########.", "##########", "##########", ".########.", "..######..", "...####...", "....##....", ".........."];
    let (r, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
    ui.painter().rect_filled(r, 6.0, t.panel_alt);
    let px = 3.0;
    let origin = r.center() - vec2(5.0 * px, 4.5 * px);
    for (y, row) in ROWS.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == '#' {
                let p = origin + vec2(x as f32 * px, y as f32 * px);
                ui.painter().rect_filled(Rect::from_min_size(pos2(p.x, p.y), vec2(px, px)), 0.0, Color32::from_rgb(0xd2, 0x4b, 0x3c));
            }
        }
    }
}

fn link(ui: &mut egui::Ui, text: &str, enabled: bool, why: &str) -> bool {
    let r = ui.add_enabled(enabled, egui::Link::new(text));
    if enabled {
        r.clicked()
    } else {
        r.on_disabled_hover_text(why);
        false
    }
}

fn boxed(ui: &mut egui::Ui, t: &Tokens, title: &str, height: f32, body: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).color(t.text_dim));
    egui::Frame::NONE.fill(t.panel_alt).stroke(egui::Stroke::new(1.0, t.border)).inner_margin(8).show(ui, |ui| {
        ui.set_min_height(height);
        ui.set_width(ui.available_width());
        body(ui);
    });
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    egui::Frame::NONE.inner_margin(12).show(ui, |ui| {
        ui.horizontal(|ui| {
            mark(ui, t);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if link(ui, "New File...", true, "") {
                    app.act(UiAction::NewFile);
                }
                link(ui, "Open File...", false, "Opening files is not implemented yet");
                link(ui, "Recover Files...", false, "File recovery is not implemented yet");
            });
        });
        ui.add_space(10.0);
        let tall = (ui.available_height() - 60.0).max(120.0);
        boxed(ui, t, "Recent files:", tall * 0.5, |ui| {
            ui.label(RichText::new("No recent files yet").color(t.text_dim));
        });
        ui.add_space(6.0);
        boxed(ui, t, "Recent folders:", tall * 0.5 - 40.0, |ui| {
            ui.label(RichText::new("No recent folders yet").color(t.text_dim));
        });
    });
}
