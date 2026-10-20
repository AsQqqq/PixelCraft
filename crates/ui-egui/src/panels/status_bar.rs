//! The status bar: the pointer's pixel and the sprite size on the left (or the last message),
//! the current frame and the zoom level on the right.

use crate::theme::{Tokens, size};
use crate::{PixelApp, zoom};

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let doc = app.active_doc().and_then(|i| app.session.docs.get(i));
    let left = match (&app.status, doc, app.ui.hover_tool) {
        (Some(msg), _, _) => msg.clone(),
        // Over a tool bar button: the tool and its key.
        (None, Some(_), Some(tool)) => format!("{} Tool    {}", tool.label(), super::shortcut_text(tool.shortcut())),
        (None, Some(d), None) => {
            // The pointer's pixel while over the canvas, else the sprite's name; then the sprite
            // size and, with a selection, its size.
            let lead = app.ui.hover_pixel.map_or(d.name.clone(), |(x, y)| format!("+ {x} {y}"));
            let sel = d.selection.as_ref().map_or(String::new(), |m| {
                let b = m.bounds();
                format!("    Selection {} {}", b.w, b.h)
            });
            format!("{lead}    □ {} {}{sel}", d.sprite.size.w, d.sprite.size.h)
        }
        // Home or no tabs: the version line, centred like the target app.
        (None, None, _) => format!("{} {} by the ArtCraft team", crate::APP_NAME, crate::VERSION),
    };
    let right = doc.map(|d| {
        let z = app.ui.viewports.get(&d.uid).map_or(1.0, |v| v.zoom);
        format!("Frame: {}/{}    {}", d.frame + 1, d.sprite.frames.len(), zoom::label(z))
    });
    egui::Panel::bottom("status_bar")
        .exact_size(size::STATUS_BAR_H)
        .frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin::symmetric(8, 3)))
        .show(ui, |ui| {
            if doc.is_none() && app.status.is_none() {
                ui.centered_and_justified(|ui| ui.label(egui::RichText::new(left).color(t.text_dim)));
                return;
            }
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(left).color(t.text_dim));
                if let Some(right) = right {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(egui::RichText::new(right).monospace()));
                }
            });
        });
}
