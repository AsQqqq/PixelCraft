//! The window's panels, one module each, in the target app's arrangement.

pub mod canvas;
pub mod color_bar;
pub mod color_picker;
pub mod context_bar;
pub mod dialogs;
pub mod home;
pub mod menu_bar;
pub mod paste;
pub mod preview;
pub mod status_bar;
pub mod tabs;
pub mod timeline;
pub mod tool_bar;

/// A shortcut as the platform writes it: `Ctrl+Z` reads `Cmd+Z` on macOS.
pub fn shortcut_text(s: &str) -> String {
    if cfg!(target_os = "macos") { s.replace("Ctrl+", "Cmd+") } else { s.to_string() }
}
