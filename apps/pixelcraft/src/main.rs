//! PixelCraft desktop app.
//!
//! Usage: `pixelcraft [--width <px>] [--height <px>]`
//!
//! Opens on the Home tab, like the target app. Opening files, the control channel (`--control <port>`)
//! and native menus follow (`docs/roadmap.md`).
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use pixelcraft_ui_egui::PixelApp;

struct App(PixelApp);

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("PixelCraft").with_inner_size([1280.0, 800.0]).with_min_inner_size([640.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native("PixelCraft", options, Box::new(|_cc| Ok(Box::new(App(PixelApp::default())))))
}
