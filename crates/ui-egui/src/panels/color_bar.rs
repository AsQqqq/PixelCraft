//! The colour bar on the left, full height: the sprite's palette (click = foreground,
//! right click = background), the foreground/background swatches and a colour picker.

use pixelcraft_engine::color::Rgba8;

use crate::theme::{Tokens, size};
use crate::{PixelApp, UiAction};

fn c32(c: Rgba8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let palette = app.session.active().map(|d| d.sprite.palette.colors.clone()).unwrap_or_default();
    egui::Panel::left("color_bar").exact_size(size::COLOR_BAR_W).resizable(false).frame(egui::Frame::NONE.fill(t.panel).inner_margin(6)).show(
        ui,
        |ui| {
            // Palette grid.
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            for (row, chunk) in palette.chunks(size::PALETTE_COLUMNS).enumerate() {
                ui.horizontal(|ui| {
                    for (col, &c) in chunk.iter().enumerate() {
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(size::SWATCH, size::SWATCH), egui::Sense::click());
                        ui.painter().rect_filled(r, 0.0, c32(c));
                        if c == app.ui.fg {
                            let corner = [r.left_top(), r.left_top() + egui::vec2(9.0, 0.0), r.left_top() + egui::vec2(0.0, 9.0)];
                            ui.painter().add(egui::Shape::convex_polygon(corner.to_vec(), egui::Color32::WHITE, egui::Stroke::new(1.0, t.border)));
                        }
                        let resp = resp.on_hover_text(format!("Index {} · {}", row * size::PALETTE_COLUMNS + col, c.to_hex()));
                        if resp.clicked() {
                            app.ui.fg = c;
                        }
                        if resp.secondary_clicked() {
                            app.ui.bg = c;
                        }
                    }
                });
            }
            ui.add_space(10.0);
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            // Foreground over background; X swaps them.
            let (r, resp) = ui.allocate_exact_size(egui::vec2(56.0, 44.0), egui::Sense::click());
            let bg = egui::Rect::from_min_size(r.min + egui::vec2(20.0, 16.0), egui::vec2(28.0, 28.0));
            let fg = egui::Rect::from_min_size(r.min, egui::vec2(28.0, 28.0));
            ui.painter().rect_filled(bg, 2.0, c32(app.ui.bg));
            ui.painter().rect_stroke(bg, 2.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Outside);
            ui.painter().rect_filled(fg, 2.0, c32(app.ui.fg));
            ui.painter().rect_stroke(fg, 2.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Outside);
            if resp.on_hover_text("Foreground / background (X swaps)").clicked() {
                app.act(UiAction::SwapColors);
            }

            // Colour picker for the foreground.
            ui.spacing_mut().item_spacing.y = 6.0;
            super::color_picker::show(ui, &mut app.ui.fg, &mut app.ui.picker_hue, size::COLOR_BAR_W - 12.0, t);
            let index = palette.iter().position(|&p| p == app.ui.fg);
            ui.label(egui::RichText::new(index.map_or("Not in palette".to_string(), |i| format!("Idx-{i}"))).color(t.text_dim));
            ui.label(egui::RichText::new(app.ui.fg.to_hex()).monospace());
        },
    );
}
