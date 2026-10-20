//! PixelCraft in the browser: the same UI crate, run by eframe's web runner.
//!
//! Build with trunk (`cd apps/pixelcraft-web && trunk serve`). On native targets this binary only
//! explains that.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("pixelcraft-web runs in the browser: cd apps/pixelcraft-web && trunk serve");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    struct App(pixelcraft_ui_egui::PixelApp);
    impl eframe::App for App {
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            self.0.ui(ui);
            // File ▸ Open...: the browser's file picker; the files arrive as bytes.
            if self.0.take_open_request() {
                let (tx, ctx) = (self.0.picked_sender(), ui.ctx().clone());
                wasm_bindgen_futures::spawn_local(async move {
                    let dialog = rfd::AsyncFileDialog::new().add_filter("Images", pixelcraft_engine::io::IMAGE_EXTENSIONS);
                    for file in dialog.pick_files().await.unwrap_or_default() {
                        let bytes = file.read().await;
                        let _ = tx.send(pixelcraft_ui_egui::open::Picked::Bytes { name: file.file_name(), bytes });
                    }
                    ctx.request_repaint();
                });
            }
        }
    }

    wasm_bindgen_futures::spawn_local(async {
        let Some(canvas) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("pixelcraft"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            log::error!("no <canvas id=\"pixelcraft\"> on the page");
            return;
        };
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| {
                    let mut app = pixelcraft_ui_egui::PixelApp::default();
                    app.can_open_dialog = true;
                    Ok(Box::new(App(app)))
                }),
            )
            .await;
        if let Err(e) = started {
            log::error!("PixelCraft failed to start: {e:?}");
        }
    });
}
