//! PixelCraft's egui frontend: a thin shell over the engine.
//!
//! Panels read engine state and act only through [`PixelApp::run`] (engine commands, the same
//! ones the CLI, the control channel and MCP use), [`PixelApp::invoke`] (a command chosen by the
//! user: may open its dialog first) or [`PixelApp::act`] (view-only actions). Colours and sizes
//! come from [`theme`]. Nothing below this crate depends on egui, so the UI can be swapped.
//!
//! Layout (`docs/ui-design.md`): menu bar, tabs and context bar on top; colour bar on the left
//! and tool bar on the right, both full height; timeline and status bar at the bottom; the
//! canvas in the middle. The app starts on the Home tab; the lifecycle (new, close, quit with
//! unsaved changes) follows `plan/aseprite/lifecycle.md`.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

pub mod cursor;
pub mod icons;
pub mod keys;
pub mod panels;
pub mod pixel_font;
pub mod state;
pub mod theme;
pub mod zoom;

use pixelcraft_engine::Session;
use serde_json::{Value as Json, json};

pub use state::{Dialog, Tool, UiState};

pub const APP_NAME: &str = "PixelCraft";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// View-only actions: they change what the window shows, never the sprite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiAction {
    /// File ▸ New...: opens the New Sprite dialog.
    NewFile,
    /// File ▸ Close: closes the active tab, asking first if the sprite has unsaved changes.
    CloseTab,
    /// File ▸ Exit: quits, asking about each sprite with unsaved changes.
    Quit,
    /// View ▸ Home: opens and shows the Home tab.
    Home,
    ZoomIn,
    ZoomOut,
    /// A digit key's level (`zoom::preset_for_digit`).
    ZoomPreset(u8),
    Fit,
    Center,
    ToggleTimeline,
    TogglePlay,
    SwapColors,
    SetTool(Tool),
    /// `-` / `+`: brush size down or up by this many pixels.
    BrushSize(i32),
    /// F7 or the tool bar's bottom button.
    TogglePreview,
    /// Shift+Enter or the Preview window's ▶ / ■: play the animation in the Preview window.
    TogglePreviewPlay,
    /// Edit ▸ Preferences... (Cmd+, / Cmd+K).
    Preferences,
}

impl UiAction {
    /// Menu entries for the UI's own actions: (menu, label, shortcut, action).
    pub const MENU: &'static [(&'static str, &'static str, &'static str, UiAction)] = &[
        ("File", "Exit", "Ctrl+Q", UiAction::Quit),
        ("Edit", "Preferences...", "Ctrl+,", UiAction::Preferences),
        ("Frame", "Play Animation", "Enter", UiAction::TogglePlay),
        ("Frame", "Play Preview Animation", "Shift+Enter", UiAction::TogglePreviewPlay),
        ("View", "Zoom In", "Ctrl+=", UiAction::ZoomIn),
        ("View", "Zoom Out", "Ctrl+-", UiAction::ZoomOut),
        ("View", "Zoom 100%", "1", UiAction::ZoomPreset(1)),
        ("View", "Fit on Screen", "Ctrl+0", UiAction::Fit),
        ("View", "Scroll to Center", "Shift+Z", UiAction::Center),
        ("View", "Timeline", "Tab", UiAction::ToggleTimeline),
        ("View", "Preview", "F7", UiAction::TogglePreview),
        ("View", "Home", "", UiAction::Home),
    ];
}

pub struct PixelApp {
    pub session: Session,
    pub ui: UiState,
    /// The last command error or notice, shown in the status bar until the next command.
    pub status: Option<String>,
    /// A quit is in progress: closing each sprite with unsaved changes, one Warning at a time.
    pub quitting: bool,
    /// Every unsaved sprite was handled: the window may close now.
    pub quit_confirmed: bool,
    canvas: panels::canvas::CanvasCache,
    preview: panels::preview::PreviewCache,
    cursors: cursor::Cursors,
    /// Milliseconds spent on the current frame during playback.
    play_ms: f32,
    title: String,
    /// The pixel font and style are installed (once per context).
    styled: bool,
    /// Where interactive spots were drawn last frame (for UI tests and automation).
    pub hit: HitRects,
}

/// Screen rectangles of a few controls, recorded as they are drawn.
#[derive(Clone, Debug, Default)]
pub struct HitRects {
    /// The free space after the last tab (double-click: New Sprite).
    pub tab_free: Option<egui::Rect>,
    /// The Preview window's play/stop button.
    pub preview_play: Option<egui::Rect>,
}

impl Default for PixelApp {
    fn default() -> Self {
        Self::new(Session::new())
    }
}

impl PixelApp {
    /// The app as it launches: the Home tab, no sprite.
    pub fn new(session: Session) -> Self {
        Self {
            session,
            ui: UiState::default(),
            status: None,
            quitting: false,
            quit_confirmed: false,
            canvas: panels::canvas::CanvasCache::default(),
            preview: panels::preview::PreviewCache::default(),
            cursors: cursor::Cursors::default(),
            play_ms: 0.0,
            title: String::new(),
            styled: false,
            hit: HitRects::default(),
        }
    }

    /// A window with one new sprite open (for snapshots and tests).
    pub fn with_new_sprite() -> Self {
        let mut app = Self::default();
        if app.run("file.new", &json!({})).is_some() {
            app.ui.home_active = false;
        }
        app
    }

    /// Runs an engine command; an error goes to the status bar instead of interrupting the user.
    pub fn run(&mut self, id: &str, params: &Json) -> Option<Json> {
        match self.session.execute(id, params) {
            Ok(v) => {
                self.status = None;
                Some(v)
            }
            Err(e) => {
                log::warn!("{id}: {e}");
                self.status = Some(e.to_string());
                None
            }
        }
    }

    /// A command the user chose from a menu, a key or a button: commands that need the user's
    /// input or confirmation go through their dialog first; the rest run directly.
    pub fn invoke(&mut self, id: &str) {
        match id {
            "file.new" => self.act(UiAction::NewFile),
            "file.close" => self.act(UiAction::CloseTab),
            _ => {
                self.run(id, &json!({}));
            }
        }
    }

    /// The active document's index, unless Home is the active tab.
    pub fn active_doc(&self) -> Option<usize> {
        if self.ui.home_active { None } else { self.session.active }
    }

    /// Closes document `i`, asking first when it has unsaved changes.
    pub fn request_close(&mut self, i: usize) {
        let Some(d) = self.session.docs.get(i) else { return };
        if d.is_dirty() {
            let uid = d.uid;
            self.session.active = Some(i);
            self.ui.home_active = false;
            self.ui.dialog = Some(Dialog::Unsaved { uid, quitting: false });
        } else {
            self.close_now(i);
        }
    }

    /// Closes document `i` without asking. With no sprite left, Home shows if it is open.
    pub fn close_now(&mut self, i: usize) {
        if let Some(uid) = self.session.docs.get(i).map(|d| d.uid) {
            self.ui.viewports.remove(&uid);
        }
        self.run("file.close", &json!({ "document": i }));
        if self.session.docs.is_empty() {
            self.ui.home_active = self.ui.home_open;
        }
    }

    /// Runs a view-only action.
    pub fn act(&mut self, action: UiAction) {
        match action {
            UiAction::NewFile => {
                let form = self.ui.new_sprite.clone();
                let (width, height) = (form.width.to_string(), form.height.to_string());
                self.ui.dialog = Some(Dialog::NewSprite { form, width, height, focus: true });
            }
            UiAction::CloseTab => match self.active_doc() {
                Some(i) => self.request_close(i),
                None if self.ui.home_active => {
                    self.ui.home_open = false;
                    self.ui.home_active = false;
                }
                None => {}
            },
            UiAction::Quit => {
                // One Warning per sprite with unsaved changes, first one first; then close.
                self.quitting = true;
                match self.session.docs.iter().position(|d| d.is_dirty()) {
                    Some(i) => {
                        let uid = self.session.docs.get(i).map_or(0, |d| d.uid);
                        self.session.active = Some(i);
                        self.ui.home_active = false;
                        self.ui.dialog = Some(Dialog::Unsaved { uid, quitting: true });
                    }
                    None => self.quit_confirmed = true,
                }
            }
            UiAction::Home => {
                self.ui.home_open = true;
                self.ui.home_active = true;
            }
            UiAction::ZoomIn => panels::canvas::zoom_step(self, true),
            UiAction::ZoomOut => panels::canvas::zoom_step(self, false),
            UiAction::ZoomPreset(d) => {
                if let Some(z) = zoom::preset_for_digit(d) {
                    panels::canvas::zoom_set(self, z);
                }
            }
            UiAction::Fit => panels::canvas::fit(self),
            UiAction::Center => {
                if let Some(vp) = panels::canvas::viewport_mut(self) {
                    vp.center();
                }
            }
            UiAction::ToggleTimeline => self.ui.timeline_visible = !self.ui.timeline_visible,
            UiAction::TogglePlay => {
                self.ui.playing = !self.ui.playing && self.active_doc().is_some();
                self.play_ms = 0.0;
            }
            UiAction::SwapColors => std::mem::swap(&mut self.ui.fg, &mut self.ui.bg),
            UiAction::SetTool(t) => {
                self.ui.tool = t;
                if let Some(slot) = self.ui.group_tool.get_mut(t.group()) {
                    *slot = t;
                }
            }
            UiAction::BrushSize(d) => {
                self.ui.brush_size = (self.ui.brush_size as i32).saturating_add(d).clamp(1, pixelcraft_engine::paint::MAX_BRUSH as i32) as u32;
            }
            UiAction::TogglePreview => self.ui.show_preview = !self.ui.show_preview,
            UiAction::Preferences => {
                let draft = self.ui.prefs.clone();
                self.ui.dialog = Some(Dialog::Preferences { section: state::PrefSection::Cursors, draft });
            }
            UiAction::TogglePreviewPlay => {
                let editor_frame = self.active_doc().and_then(|i| self.session.docs.get(i)).map(|d| d.frame);
                let p = &mut self.ui.preview;
                p.playing = !p.playing && editor_frame.is_some();
                p.elapsed_ms = 0.0;
                // Start from the frame the editor shows; the preview window opens if it was hidden.
                if p.playing {
                    p.frame = editor_frame.unwrap_or(0);
                    self.ui.show_preview = true;
                }
            }
        }
    }

    /// The window title: `Sprite-0001 - PixelCraft v<version>`, or just the app on Home.
    pub fn window_title(&self) -> String {
        match self.active_doc().and_then(|i| self.session.docs.get(i)) {
            Some(d) => format!("{} - {APP_NAME} v{VERSION}", d.name),
            None => format!("{APP_NAME} v{VERSION}"),
        }
    }

    /// Advances playback by the frame's own duration.
    fn tick_playback(&mut self, ctx: &egui::Context) {
        if !self.ui.playing {
            return;
        }
        let Some(d) = self.active_doc().and_then(|i| self.session.docs.get(i)) else {
            self.ui.playing = false;
            return;
        };
        let duration = d.sprite.frames.get(d.frame).map_or(100, |f| f.duration_ms).max(1) as f32;
        self.play_ms += ctx.input(|i| i.stable_dt) * 1000.0;
        if self.play_ms >= duration {
            self.play_ms = 0.0;
            self.run("frame.goToNext", &json!({}));
        }
        ctx.request_repaint();
    }

    /// The window's close button (or Cmd+Q via the OS): ask about unsaved sprites first.
    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.quit_confirmed {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.ui.dialog.is_none() {
                self.act(UiAction::Quit);
            }
        }
        if self.quit_confirmed {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let t = theme::Tokens::get(&ctx);
        if !self.styled {
            theme::setup(&ctx, &t);
            self.styled = true;
        }
        self.handle_close_request(&ctx);
        if self.ui.dialog.is_none() {
            keys::handle(self, &ctx);
        }
        self.tick_playback(&ctx);
        let title = self.window_title();
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }

        panels::menu_bar::show(self, ui, &t);
        panels::tabs::show(self, ui, &t);
        let editing = self.active_doc().is_some();
        if editing {
            panels::context_bar::show(self, ui, &t);
        }
        panels::status_bar::show(self, ui, &t);
        if editing {
            panels::color_bar::show(self, ui, &t);
            panels::tool_bar::show(self, ui, &t);
            if self.ui.timeline_visible {
                panels::timeline::show(self, ui, &t);
            }
        }
        let fill = if editing { t.canvas_bg } else { t.window };
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(fill)).show(ui, |ui| {
            if editing {
                panels::canvas::show(self, ui, &t);
            } else if self.ui.home_active {
                panels::home::show(self, ui, &t);
            }
        });
        if editing && self.ui.show_preview {
            panels::preview::show(self, &ctx, &t);
        }
        panels::dialogs::show(self, &ctx, &t);
        // Last: the cursor the widgets asked for, as a pixel-art OS cursor (or the native one).
        // macOS sizes cursor images in points; other systems in physical pixels.
        let density = if cfg!(target_os = "macos") { 1.0 } else { ctx.pixels_per_point() };
        let scale = (f32::from(self.ui.prefs.cursor_scale.clamp(1, 3)) * theme::size::UI_PX * density).round() as u16;
        self.cursors.apply(&ctx, !self.ui.prefs.native_cursor, scale.max(1));
    }
}

#[cfg(test)]
mod tests;
