//! File ▸ Open...: the OS's Open dialog (rfd), without blocking the event loop. The dialog is
//! created here on the main thread with the window as its parent (a sheet on macOS), and a
//! helper thread waits for the answer and hands each picked path to the UI.

use std::future::Future;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use pixelcraft_engine::io::IMAGE_EXTENSIONS;
use pixelcraft_ui_egui::open::Picked;

/// The image extensions, also in capitals: Linux dialogs match `*.ext` case-sensitively.
fn extensions() -> Vec<String> {
    IMAGE_EXTENSIONS.iter().flat_map(|e| [e.to_string(), e.to_uppercase()]).collect()
}

pub fn show(frame: &eframe::Frame, ctx: egui::Context, tx: Sender<Picked>) {
    let picked = rfd::AsyncFileDialog::new().set_parent(frame).set_title("Open").add_filter("Images", &extensions()).pick_files();
    let waited = std::thread::Builder::new().name("open dialog".into()).spawn(move || {
        for file in block_on(picked).unwrap_or_default() {
            let _ = tx.send(Picked::Path(file.path().to_string_lossy().into_owned()));
        }
        ctx.request_repaint();
    });
    if let Err(e) = waited {
        log::error!("couldn't wait for the Open dialog: {e}");
    }
}

/// Runs `future` to completion on this thread, sleeping until it is woken.
fn block_on<T>(future: impl Future<Output = T>) -> T {
    struct Unpark(std::thread::Thread);
    impl std::task::Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = std::task::Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        // Spurious wake-ups just poll again.
        std::thread::park();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn filters_list_both_cases() {
        let e = super::extensions();
        assert!(e.contains(&"png".to_string()) && e.contains(&"PNG".to_string()) && e.contains(&"jpeg".to_string()));
    }
}
