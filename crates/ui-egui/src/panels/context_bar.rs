//! The context bar under the tabs: options of the current tool.

use crate::theme::{Tokens, size};
use crate::{PixelApp, Tool, UiAction};

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
                        ui.label("Size");
                        ui.add_enabled(false, egui::Button::new("1px")).on_disabled_hover_text("Brush sizes are not implemented yet");
                        ui.label(egui::RichText::new("Simple Ink").color(t.text_dim));
                        ui.checkbox(&mut app.ui.pixel_perfect, "Pixel-perfect");
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
