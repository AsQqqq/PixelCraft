//! The tab strip under the menu bar: Home (when open), then one tab per sprite. A sprite with
//! unsaved changes shows ● instead of ×; either closes it (asking first when unsaved). With no
//! tabs at all, a small Home button stays at the left.

use crate::icons::{self, Icon};
use crate::theme::{Tokens, size};
use crate::{PixelApp, UiAction};

/// A small icon that reacts to clicks (tab closers, the Home button).
fn icon_button(ui: &mut egui::Ui, icon: Icon, side: f32, color: egui::Color32, t: &Tokens) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 3.0, t.button_hover);
    }
    icons::paint(ui.painter(), r, icon, color);
    resp
}

enum Click {
    Home,
    CloseHome,
    Doc(usize),
    Close(usize),
}

fn tab(ui: &mut egui::Ui, t: &Tokens, home: bool, label: &str, on: bool, closer: Icon, tip: &str) -> (bool, bool) {
    let mut out = (false, false);
    egui::Frame::NONE
        .fill(if on { t.panel } else { t.panel_alt })
        .corner_radius(egui::CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 })
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            let color = if on { t.text } else { t.text_dim };
            if home {
                out.0 |= icon_button(ui, Icon::Home, 14.0, color, t).clicked();
            }
            out.0 |= ui.add(egui::Label::new(egui::RichText::new(label).color(color)).sense(egui::Sense::click())).clicked();
            out.1 = icon_button(ui, closer, 14.0, color, t).on_hover_text(tip).clicked();
        });
    out
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let docs: Vec<(String, bool)> = app.session.docs.iter().map(|d| (d.name.clone(), d.is_dirty())).collect();
    let active = app.active_doc();
    let (home_open, home_on) = (app.ui.home_open, app.ui.home_active);
    let mut click = None;
    egui::Panel::top("tabs")
        .exact_size(size::TAB_BAR_H)
        .frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin { left: 4, right: 4, top: 2, bottom: 0 }))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if home_open {
                    let (open, close) = tab(ui, t, true, "Home", home_on, Icon::Close, "Close");
                    if open {
                        click = Some(Click::Home);
                    }
                    if close {
                        click = Some(Click::CloseHome);
                    }
                } else if docs.is_empty() && icon_button(ui, Icon::Home, 18.0, t.text_dim, t).on_hover_text("Home").clicked() {
                    click = Some(Click::Home);
                }
                for (i, (name, dirty)) in docs.iter().enumerate() {
                    let tip = if *dirty { "Unsaved changes — close" } else { "Close" };
                    let (open, close) = tab(ui, t, false, name, active == Some(i), if *dirty { Icon::Unsaved } else { Icon::Close }, tip);
                    if open {
                        click = Some(Click::Doc(i));
                    }
                    if close {
                        click = Some(Click::Close(i));
                    }
                }
            });
        });
    match click {
        Some(Click::Home) => app.act(UiAction::Home),
        Some(Click::CloseHome) => {
            app.ui.home_open = false;
            app.ui.home_active = false;
        }
        Some(Click::Doc(i)) => {
            app.session.active = Some(i);
            app.ui.home_active = false;
        }
        Some(Click::Close(i)) => app.request_close(i),
        None => {}
    }
}
