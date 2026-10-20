//! Opening images: File ▸ Open... (Cmd+O, the Home tab's link) and files dropped on the window.
//! Each file opens as a new sprite tab (`file.open`, or `Session::open_image` for a browser drop,
//! which has bytes but no path). Adding an image to an open sprite comes later.
//!
//! The UI crate shows no OS dialog itself (the UI layer stays free of rfd): the host app sets
//! [`PixelApp::can_open_dialog`], takes the request with [`PixelApp::take_open_request`], shows
//! its dialog and sends what was picked through [`PixelApp::picked_sender`].

use std::sync::mpsc::{Receiver, Sender, channel};

use serde_json::json;

use crate::PixelApp;
use crate::theme::Tokens;

/// A file the user picked in a dialog or dropped on the window.
#[derive(Clone, Debug)]
pub enum Picked {
    Path(String),
    /// A file without a path (the browser): its name and contents.
    Bytes {
        name: String,
        bytes: Vec<u8>,
    },
}

/// The channel the host's dialog sends picked files through.
pub struct Inbox {
    tx: Sender<Picked>,
    rx: Receiver<Picked>,
}

impl Default for Inbox {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }
}

impl PixelApp {
    /// File ▸ Open...: asks the host for its Open dialog, or says how else to open a file.
    pub fn request_open(&mut self) {
        if self.can_open_dialog {
            self.open_requested = true;
        } else {
            self.status = Some("No Open dialog here: drag an image onto the window".into());
        }
    }

    /// The host's side: true once per File ▸ Open... the user chose.
    pub fn take_open_request(&mut self) -> bool {
        std::mem::take(&mut self.open_requested)
    }

    /// Where the host's dialog sends what the user picked (from any thread; then repaint).
    pub fn picked_sender(&self) -> Sender<Picked> {
        self.inbox.tx.clone()
    }

    /// Opens one picked or dropped file as a new tab; a failure goes to the status bar.
    pub fn open_picked(&mut self, picked: Picked) {
        let opened = match picked {
            Picked::Path(path) => self.run("file.open", &json!({ "path": path })).is_some(),
            Picked::Bytes { name, bytes } => match self.session.open_image(&name, None, &bytes) {
                Ok(_) => {
                    self.status = None;
                    true
                }
                Err(e) => {
                    self.status = Some(format!("{name}: {e}"));
                    false
                }
            },
        };
        if opened {
            self.ui.home_active = false;
            if let Some(uid) = self.session.active().map(|d| d.uid) {
                self.ui.fit_on_open.insert(uid);
            }
        }
    }

    /// Start of frame: files the dialog picked and files dropped on the window.
    pub(crate) fn open_incoming(&mut self, ctx: &egui::Context) {
        let dropped = self.take_dropped(ctx);
        let picked: Vec<Picked> = self.inbox.rx.try_iter().chain(dropped).collect();
        if picked.is_empty() {
            return;
        }
        for p in picked {
            self.open_picked(p);
        }
        // Files dragged in from another app: PixelCraft comes to the front.
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Native drops carry the file's path.
    #[cfg(not(target_arch = "wasm32"))]
    fn take_dropped(&mut self, ctx: &egui::Context) -> Vec<Picked> {
        ctx.input(|i| i.raw.dropped_files.iter().map(|f| Picked::Path(f.path().to_string_lossy().into_owned())).collect())
    }

    /// The browser's drops are read asynchronously and arrive through the inbox a frame later.
    #[cfg(target_arch = "wasm32")]
    fn take_dropped(&mut self, ctx: &egui::Context) -> Vec<Picked> {
        for f in ctx.input(|i| i.raw.dropped_files.clone()) {
            let (tx, ctx) = (self.inbox.tx.clone(), ctx.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let name = f.path().to_string_lossy().into_owned();
                match f.bytes_async().await {
                    Ok(bytes) => {
                        let _ = tx.send(Picked::Bytes { name, bytes });
                        ctx.request_repaint();
                    }
                    Err(e) => log::warn!("can't read the dropped file {name}: {e}"),
                }
            });
        }
        Vec::new()
    }
}

/// While files are dragged over the window: a frame and a hint over everything.
pub(crate) fn drop_hint(ctx: &egui::Context, t: &Tokens) {
    if ctx.input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }
    let screen = ctx.content_rect();
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("drop_hint")));
    p.rect_filled(screen, 0.0, t.window.gamma_multiply(0.75));
    p.rect_stroke(screen.shrink(8.0), 0.0, egui::Stroke::new(crate::theme::size::UI_PX * 2.0, t.accent), egui::StrokeKind::Inside);
    p.text(screen.center(), egui::Align2::CENTER_CENTER, "Drop to open as a new sprite", egui::FontId::proportional(crate::pixel_font::SIZE), t.text);
    ctx.request_repaint();
}
