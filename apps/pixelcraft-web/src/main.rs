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
            .start(canvas, eframe::WebOptions::default(), Box::new(|_cc| Ok(Box::new(App(pixelcraft_ui_egui::PixelApp::default())))))
            .await;
        if let Err(e) = started {
            log::error!("PixelCraft failed to start: {e:?}");
        }
    });
}
