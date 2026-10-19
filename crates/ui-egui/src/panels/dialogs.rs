//! Modal dialogs, as the target app shows them (`plan/aseprite/lifecycle.md`):
//!
//! - **New Sprite**: width and height (remembered), colour mode and background toggles,
//!   Advanced Options, OK / Cancel. Enter = OK, Escape = Cancel.
//! - **Warning** (unsaved changes): "Saving changes to the sprite "…" before closing?" (or
//!   quitting) with Save / Don't Save / Cancel.

use egui::{Align, Layout, RichText};
use serde_json::json;

use crate::state::{Dialog, NewSpriteForm};
use crate::theme::Tokens;
use crate::{PixelApp, UiAction};

const MODES: [(&str, &str); 3] = [("rgba", "RGBA"), ("grayscale", "Grayscale"), ("indexed", "Indexed")];
const BACKGROUNDS: [(&str, &str); 3] = [("transparent", "Transparent"), ("white", "White"), ("black", "Black")];

pub fn show(app: &mut PixelApp, ctx: &egui::Context, t: &Tokens) {
    let Some(dialog) = app.ui.dialog.clone() else { return };
    match dialog {
        Dialog::NewSprite { form, width, height, focus } => new_sprite(app, ctx, t, form, width, height, focus),
        Dialog::Unsaved { uid, quitting } => unsaved(app, ctx, uid, quitting),
    }
}

fn toggles(ui: &mut egui::Ui, value: &mut String, options: &[(&str, &str)]) {
    ui.horizontal(|ui| {
        for (id, label) in options {
            if ui.selectable_label(value == id, *label).clicked() {
                *value = id.to_string();
            }
        }
    });
}

fn new_sprite(app: &mut PixelApp, ctx: &egui::Context, t: &Tokens, mut form: NewSpriteForm, mut width: String, mut height: String, focus: bool) {
    let (enter, escape) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
    let mut ok = enter;
    let mut cancel = escape;
    egui::Modal::new(egui::Id::new("new_sprite")).show(ctx, |ui| {
        ui.set_width(240.0);
        ui.heading("New Sprite");
        ui.add_space(4.0);
        ui.label(RichText::new("Size:").color(t.text_dim));
        egui::Grid::new("new_sprite_size").num_columns(3).show(ui, |ui| {
            ui.label("Width:");
            let w = ui.add(egui::TextEdit::singleline(&mut width).desired_width(80.0));
            ui.label(RichText::new("px").color(t.text_dim));
            ui.end_row();
            ui.label("Height:");
            ui.add(egui::TextEdit::singleline(&mut height).desired_width(80.0));
            ui.label(RichText::new("px").color(t.text_dim));
            ui.end_row();
            // Width starts focused with its text selected, ready to type over.
            if focus {
                w.request_focus();
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), w.id) {
                    let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(width.chars().count()));
                    state.cursor.set_char_range(Some(all));
                    state.store(ui.ctx(), w.id);
                }
            }
        });
        ui.add_space(4.0);
        ui.label(RichText::new("Color Mode:").color(t.text_dim));
        toggles(ui, &mut form.color_mode, &MODES);
        ui.label(RichText::new("Background:").color(t.text_dim));
        toggles(ui, &mut form.background, &BACKGROUNDS);
        ui.checkbox(&mut form.advanced, "Advanced Options");
        if form.advanced {
            ui.label(RichText::new("Pixel aspect ratio: square (1:1)").color(t.text_dim));
        }
        ui.add_space(6.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                cancel = true;
            }
            if ui.add(egui::Button::new("OK").fill(t.accent)).clicked() {
                ok = true;
            }
        });
    });
    let parse = |s: &str| s.trim().trim_end_matches("px").trim().parse::<u32>().ok().filter(|&n| (1..=65_535).contains(&n));
    if cancel {
        app.ui.dialog = None;
        return;
    }
    if ok {
        match (parse(&width), parse(&height)) {
            (Some(w), Some(h)) => {
                form.width = w;
                form.height = h;
                app.ui.new_sprite = form.clone();
                app.ui.dialog = None;
                let params = json!({ "width": w, "height": h, "colorMode": form.color_mode, "background": form.background });
                if app.run("file.new", &params).is_some() {
                    app.ui.home_active = false;
                }
            }
            _ => {
                app.status = Some("Width and height must be whole numbers from 1 to 65535".into());
                app.ui.dialog = Some(Dialog::NewSprite { form, width, height, focus: false });
            }
        }
        return;
    }
    app.ui.dialog = Some(Dialog::NewSprite { form, width, height, focus: false });
}

fn unsaved(app: &mut PixelApp, ctx: &egui::Context, uid: u64, quitting: bool) {
    let Some(name) = app.session.docs.iter().find(|d| d.uid == uid).map(|d| d.name.clone()) else {
        app.ui.dialog = None;
        return;
    };
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    let mut choice = escape.then_some("cancel");
    egui::Modal::new(egui::Id::new("unsaved")).show(ctx, |ui| {
        ui.set_width(300.0);
        ui.heading("Warning");
        ui.add_space(4.0);
        ui.label("Saving changes to the sprite");
        ui.label(format!("\"{name}\" before {}?", if quitting { "quitting" } else { "closing" }));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_enabled(false, egui::Button::new("Save")).on_disabled_hover_text("Saving .aseprite files is not implemented yet");
            if ui.button("Don't Save").clicked() {
                choice = Some("discard");
            }
            if ui.button("Cancel").clicked() {
                choice = Some("cancel");
            }
        });
    });
    match choice {
        Some("discard") => {
            app.ui.dialog = None;
            if let Some(i) = app.session.docs.iter().position(|d| d.uid == uid) {
                app.close_now(i);
            }
            if quitting {
                app.act(UiAction::Quit);
            }
        }
        Some(_) => {
            app.ui.dialog = None;
            app.quitting = false;
        }
        None => {}
    }
}
