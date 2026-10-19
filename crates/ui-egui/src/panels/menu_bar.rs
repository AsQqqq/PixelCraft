//! The menu bar, built from the engine's command catalog plus the UI's own view actions: a
//! command whose `menu` path starts with a top-level menu name appears there, enabled or greyed
//! out as the engine says.

use super::shortcut_text;
use crate::theme::Tokens;
use crate::{PixelApp, UiAction};
use pixelcraft_engine::command_specs;

const MENUS: [&str; 8] = ["File", "Edit", "Sprite", "Layer", "Frame", "Select", "View", "Help"];

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    egui::Panel::top("menu_bar").frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin::symmetric(6, 2))).show(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            for menu in MENUS {
                ui.menu_button(menu, |ui| {
                    let mut any = false;
                    for spec in command_specs().iter().filter(|c| c.menu.first() == Some(&menu)) {
                        any = true;
                        let enabled = (spec.enabled)(&app.session).is_ok();
                        let mut button = egui::Button::new(spec.label);
                        if let Some(sc) = spec.shortcut {
                            button = button.shortcut_text(shortcut_text(sc));
                        }
                        if ui.add_enabled(enabled, button).clicked() {
                            app.invoke(spec.id);
                            ui.close();
                        }
                    }
                    for &(m, label, sc, action) in UiAction::MENU {
                        if m == menu {
                            any = true;
                            if ui.add(egui::Button::new(label).shortcut_text(if sc.is_empty() { String::new() } else { shortcut_text(sc) })).clicked()
                            {
                                app.act(action);
                                ui.close();
                            }
                        }
                    }
                    if menu == "View" {
                        ui.separator();
                        if ui.checkbox(&mut app.ui.invert_wheel_zoom, "Invert Wheel Zoom").changed() {
                            ui.close();
                        }
                        any = true;
                    }
                    if !any {
                        ui.label(egui::RichText::new("Nothing here yet").color(t.text_dim));
                    }
                });
            }
        });
    });
}
