//! PixelCraft desktop app.
//!
//! Usage: `pixelcraft [--version]`
//!
//! Images open with File ▸ Open... or by dropping them on the window (`open_dialog.rs`).
//!
//! Opens on the Home tab, like the target app. Opening files, the control channel (`--control <port>`)
//! and native menus follow (`docs/roadmap.md`).
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod app_icon;
mod open_dialog;

use pixelcraft_ui_egui::PixelApp;

/// Matches the hicolor icon name (and a future `.desktop` file), so Wayland docks find the icon.
const APP_ID: &str = "ai.storyteller.pixelcraft";

struct App(PixelApp);

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.0.ui(ui);
        if self.0.take_open_request() {
            open_dialog::show(frame, ui.ctx().clone(), self.0.picked_sender());
        }
    }
}

fn main() -> eframe::Result {
    if std::env::args().skip(1).any(|a| a == "--version" || a == "-V") {
        println!("pixelcraft {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let viewport = egui::ViewportBuilder::default()
        .with_title("PixelCraft")
        .with_app_id(APP_ID)
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([640.0, 400.0]);
    // Inside a macOS app bundle the Dock shows the bundle's .icns instead.
    let viewport =
        if app_icon::sets_runtime_icon(std::env::current_exe().ok().as_deref()) { viewport.with_icon(app_icon::window_icon()) } else { viewport };
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "PixelCraft",
        options,
        Box::new(|_cc| {
            let mut app = PixelApp::default();
            app.can_open_dialog = true;
            Ok(Box::new(App(app)))
        }),
    )
}
