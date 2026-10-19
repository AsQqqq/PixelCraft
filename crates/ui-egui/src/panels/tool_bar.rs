//! The tool bar on the right, full height: one icon button per tool (drawn in code, `icons.rs`).

use super::shortcut_text;
use crate::icons::{self, Icon};
use crate::theme::{Tokens, size};
use crate::{PixelApp, Tool, UiAction};

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    egui::Panel::right("tool_bar").exact_size(size::TOOL_BAR_W).resizable(false).frame(egui::Frame::NONE.fill(t.panel).inner_margin(3)).show(
        ui,
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
            for tool in Tool::ALL {
                let tip =
                    format!("{} ({}){}", tool.label(), shortcut_text(tool.shortcut()), if tool.works() { "" } else { " — not implemented yet" });
                if icons::button(ui, size::TOOL_BUTTON, Icon::Tool(tool), app.ui.tool == tool, t).on_hover_text(tip).clicked() {
                    app.act(UiAction::SetTool(tool));
                }
            }
        },
    );
}
