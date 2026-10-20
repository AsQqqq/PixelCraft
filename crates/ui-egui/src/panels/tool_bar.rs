//! The tool bar on the right, full height: twelve group buttons, each showing its group's
//! last-used tool. Right-click (or a long press) opens a strip with the whole group to the
//! button's left. Hovering shows the tool's tooltip and puts its name and key in the status bar.
//! At the bottom: show/hide Preview and show/hide Timeline.

use egui::{Align2, Area, Id, Order, vec2};

use super::shortcut_text;
use crate::icons::{self, Icon};
use crate::state::TOOL_GROUPS;
use crate::theme::{Tokens, size};
use crate::{PixelApp, Tool, UiAction};

const LONG_PRESS_S: f64 = 0.4;

/// The tooltip text: name, button roles, shortcut (as the target app words it).
pub fn tooltip(tool: Tool) -> String {
    let mut s = format!("{} Tool", tool.label());
    if !tool.buttons().is_empty() {
        s.push(':');
        for line in tool.buttons() {
            s.push_str("\n* ");
            s.push_str(line);
        }
        s.push('\n');
    }
    s.push_str(&format!("\nShortcut: {}", shortcut_text(tool.shortcut())));
    if !tool.works() {
        s.push_str("\n(not implemented yet)");
    }
    s
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let popup_id = Id::new("tool_group_popup");
    let mut open_group: Option<usize> = ui.ctx().data(|d| d.get_temp(popup_id)).flatten();
    let mut hover = None;
    let mut pick = None;
    let mut toggles = (false, false);
    let mut anchors = [egui::Pos2::ZERO; 12];

    egui::Panel::right("tool_bar").exact_size(size::TOOL_BAR_W).resizable(false).frame(egui::Frame::NONE.fill(t.panel).inner_margin(3)).show(
        ui,
        |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
            for (g, group) in TOOL_GROUPS.iter().enumerate() {
                let shown = app.ui.group_tool.get(g).copied().unwrap_or(Tool::Pencil);
                let active = group.contains(&app.ui.tool);
                let resp = icons::button(ui, size::TOOL_BUTTON, Icon::Tool(shown), active, t);
                if let Some(a) = anchors.get_mut(g) {
                    *a = resp.rect.left_center();
                }
                if resp.hovered() {
                    hover = Some(shown);
                }
                let resp = resp.on_hover_text(tooltip(shown));
                let long_press =
                    resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.press_start_time().is_some_and(|s| i.time - s > LONG_PRESS_S));
                if group.len() > 1 && (resp.secondary_clicked() || long_press) {
                    open_group = Some(g);
                } else if resp.clicked() {
                    pick = Some(shown);
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let tl = icons::button(ui, size::TOOL_BUTTON, Icon::Timeline, app.ui.timeline_visible, t);
                if tl.on_hover_text(if app.ui.timeline_visible { "Hide Timeline" } else { "Show Timeline" }).clicked() {
                    toggles.1 = true;
                }
                let pv = icons::button(ui, size::TOOL_BUTTON, Icon::Preview, app.ui.show_preview, t);
                if pv.on_hover_text(if app.ui.show_preview { "Hide Preview" } else { "Show Preview" }).clicked() {
                    toggles.0 = true;
                }
            });
        },
    );

    // The group strip, to the left of its button.
    if let Some(g) = open_group {
        let group = TOOL_GROUPS.get(g).copied().unwrap_or(&[]);
        let anchor = anchors.get(g).copied().unwrap_or_default();
        let area = Area::new(Id::new("tool_group_strip"))
            .order(Order::Foreground)
            .pivot(Align2::RIGHT_CENTER)
            .fixed_pos(anchor - vec2(2.0, 0.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::NONE.fill(t.panel_alt).stroke(egui::Stroke::new(1.0, t.border)).inner_margin(2).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for &tool in group {
                            let r = icons::button(ui, size::TOOL_BUTTON, Icon::Tool(tool), tool == app.ui.tool, t);
                            if r.hovered() {
                                hover = Some(tool);
                            }
                            if r.on_hover_text(tooltip(tool)).clicked() {
                                pick = Some(tool);
                            }
                        }
                    });
                });
            });
        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let clicked_away = ui.input(|i| i.pointer.any_pressed())
            && !area.response.hovered()
            && !anchors.iter().any(|a| ui.input(|i| i.pointer.interact_pos().is_some_and(|p| p.distance(*a) < size::TOOL_BUTTON)));
        if pick.is_some() || escape || clicked_away {
            open_group = None;
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(popup_id, open_group));

    app.ui.hover_tool = hover;
    if let Some(tool) = pick {
        app.act(UiAction::SetTool(tool));
    }
    if toggles.0 {
        app.act(UiAction::TogglePreview);
    }
    if toggles.1 {
        app.act(UiAction::ToggleTimeline);
    }
}
