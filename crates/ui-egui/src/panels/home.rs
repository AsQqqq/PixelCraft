//! The Home tab, shown at launch (`plan/aseprite/lifecycle.md` §1): links to start work, then
//! recent files and recent folders across the full width. (No news pane, unlike the target app.)

use egui::{Color32, Rect, RichText, Sense, pos2, vec2};

use crate::theme::Tokens;
use crate::{PixelApp, UiAction};

/// The app icon (rendered from `assets/app-icon/pixelcraft.svg`), as the Home tab's mark.
const ICON_PNG: &[u8] = include_bytes!("../../../../assets/app-icon/hicolor/64x64/apps/ai.storyteller.pixelcraft.png");

fn mark(ui: &mut egui::Ui, t: &Tokens) {
    let side = 48.0;
    let id = egui::Id::new("app_icon_texture");
    let tex: Option<egui::TextureHandle> = ui.ctx().data(|d| d.get_temp(id));
    let tex = tex.or_else(|| {
        let img = image::load_from_memory_with_format(ICON_PNG, image::ImageFormat::Png).ok()?.to_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
        let handle = ui.ctx().load_texture("app_icon", color, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(id, handle.clone()));
        Some(handle)
    });
    let (r, _) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
    match tex {
        Some(tex) => {
            ui.painter().image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            ui.painter().rect_filled(r, 6.0, t.accent);
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
