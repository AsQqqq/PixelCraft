//! The context bar under the tabs: options of the current tool.

use pixelcraft_engine::raster::SelectMode;

use crate::theme::{Tokens, size};
use crate::{PixelApp, Tool, UiAction};

/// Brush size field with `-` / `+` buttons (the keys do the same).
fn brush_size(app: &mut PixelApp, ui: &mut egui::Ui) {
    ui.label("Size");
    let mut s = app.ui.brush_size;
    ui.add(egui::DragValue::new(&mut s).range(1..=pixelcraft_engine::paint::MAX_BRUSH).suffix("px").speed(0.2));
    app.ui.brush_size = s;
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    egui::Panel::top("context_bar")
        .exact_size(size::CONTEXT_BAR_H)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(8, 4)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(app.ui.tool.label()).strong());
                ui.separator();
                match app.ui.tool {
                    Tool::Pencil | Tool::Eraser => {
                        brush_size(app, ui);
                        ui.label(egui::RichText::new("Simple Ink").color(t.text_dim));
                        ui.add_enabled(app.ui.brush_size == 1, egui::Checkbox::new(&mut app.ui.pixel_perfect, "Pixel-perfect"));
                    }
                    Tool::Spray | Tool::Line => brush_size(app, ui),
                    tool if tool.selects() => {
                        for (mode, label) in [
                            (SelectMode::Replace, "Replace"),
                            (SelectMode::Add, "Add"),
                            (SelectMode::Subtract, "Subtract"),
                            (SelectMode::Intersect, "Intersect"),
                        ] {
                            ui.selectable_value(&mut app.ui.select_mode, mode, label);
                        }
                        ui.label(
                            egui::RichText::new("Shift: add · Alt+Shift: subtract · Cmd+Shift: intersect · right button: subtract").color(t.text_dim),
                        );
                    }
                    Tool::Polygon => {
                        ui.label(egui::RichText::new("Click to add points · double-click or Enter to close · Esc cancels").color(t.text_dim));
                    }
                    Tool::Zoom | Tool::Hand => {
                        if ui.button("100%").clicked() {
                            app.act(UiAction::ZoomPreset(1));
                        }
                        if ui.button("Center").clicked() {
                            app.act(UiAction::Center);
                        }
                        if ui.button("Fit Screen").clicked() {
                            app.act(UiAction::Fit);
                        }
                    }
                    Tool::Eyedropper => {
                        ui.label(egui::RichText::new("Left click: foreground · Right click: background").color(t.text_dim));
                    }
                    other if !other.works() => {
                        ui.label(egui::RichText::new("This tool is not implemented yet").color(t.text_dim));
                    }
                    _ => {}
                }
            });
        });
}
