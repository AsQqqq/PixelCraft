//! Keyboard shortcuts: the UI's own (tools, zoom, panels) and every engine command's
//! `shortcut`, matched exactly (extra modifiers don't match) on the **physical** key, so
//! shortcuts work the same under any keyboard layout.

use egui::{Event, Key, Modifiers};
use pixelcraft_engine::command_specs;

use crate::{PixelApp, Tool, UiAction};

/// A key plus the exact modifiers held. `command` is Cmd on macOS and Ctrl elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub key: Key,
    pub command: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Chord {
    pub const fn plain(key: Key) -> Self {
        Self { key, command: false, shift: false, alt: false }
    }
    pub const fn cmd(key: Key) -> Self {
        Self { key, command: true, shift: false, alt: false }
    }
    pub const fn shift(key: Key) -> Self {
        Self { key, command: false, shift: true, alt: false }
    }

    fn from_event(key: Key, m: Modifiers) -> Self {
        Self { key, command: m.command, shift: m.shift, alt: m.alt }
    }

    /// Parses the engine's shortcut strings: `Ctrl+Shift+N`, `Alt+N`, `Left`, `Home`…
    /// (`Ctrl` means the platform's command key.)
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts: Vec<&str> = s.split('+').collect();
        let name = parts.pop()?;
        let key = Key::from_name(name)?;
        let mut c = Self::plain(key);
        for m in parts {
            match m {
                "Ctrl" | "Cmd" => c.command = true,
                "Shift" => c.shift = true,
                "Alt" => c.alt = true,
                _ => return None,
            }
        }
        Some(c)
    }
}

/// The UI's own shortcuts (the target app's defaults).
fn ui_chord(c: Chord) -> Option<UiAction> {
    use Key::*;
    let digit = |k: Key| match k {
        Backtick => Some(0),
        Num1 => Some(1),
        Num2 => Some(2),
        Num3 => Some(3),
        Num4 => Some(4),
        Num5 => Some(5),
        Num6 => Some(6),
        _ => None,
    };
    Some(match (c.command, c.shift, c.alt, c.key) {
        (true, _, false, Equals | Plus) => UiAction::ZoomIn,
        (true, false, false, Minus) => UiAction::ZoomOut,
        (true, false, false, Num0) => UiAction::Fit,
        (true, false, false, Q) => UiAction::Quit,
        (false, true, false, Z) => UiAction::Center,
        (false, false, false, Tab) => UiAction::ToggleTimeline,
        (false, false, false, X) => UiAction::SwapColors,
        (false, false, false, Enter) => UiAction::TogglePlay,
        (false, true, false, U) => UiAction::SetTool(Tool::Ellipse),
        (false, false, false, k) if digit(k).is_some() => UiAction::ZoomPreset(digit(k)?),
        (false, false, false, k) => UiAction::SetTool(Tool::ALL.into_iter().find(|t| Chord::parse(t.shortcut()) == Some(Chord::plain(k)))?),
        _ => return None,
    })
}

/// The key chords pressed this frame.
fn pressed(ctx: &egui::Context) -> Vec<Chord> {
    ctx.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                Event::Key { key, physical_key, pressed: true, modifiers, .. } => Some(Chord::from_event(physical_key.unwrap_or(*key), *modifiers)),
                _ => None,
            })
            .collect()
    })
}

/// Runs whatever this frame's key presses are bound to. Does nothing while a text field has focus.
pub fn handle(app: &mut PixelApp, ctx: &egui::Context) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let editing = app.active_doc().is_some();
    for chord in pressed(ctx) {
        if let Some(action) = ui_chord(chord) {
            // On Home only the app-level actions apply.
            if editing || matches!(action, UiAction::NewFile | UiAction::Quit | UiAction::Home) {
                app.act(action);
            }
            continue;
        }
        let hit = command_specs().iter().find(|c| c.shortcut.and_then(Chord::parse) == Some(chord));
        if let Some(spec) = hit {
            app.invoke(spec.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_engine_shortcuts() {
        assert_eq!(Chord::parse("Ctrl+Shift+N"), Some(Chord { key: Key::N, command: true, shift: true, alt: false }));
        assert_eq!(Chord::parse("Left"), Some(Chord::plain(Key::ArrowLeft)));
        assert_eq!(Chord::parse("Hyper+N"), None);
        assert_eq!(Chord::parse(""), None);
    }

    #[test]
    fn every_engine_shortcut_parses() {
        for c in command_specs() {
            if let Some(s) = c.shortcut {
                assert!(Chord::parse(s).is_some(), "{}: `{s}`", c.id);
            }
        }
    }

    #[test]
    fn tool_keys_and_modifiers_are_exact() {
        assert_eq!(ui_chord(Chord::plain(Key::B)), Some(UiAction::SetTool(Tool::Pencil)));
        assert_eq!(ui_chord(Chord::plain(Key::U)), Some(UiAction::SetTool(Tool::Rectangle)));
        assert_eq!(ui_chord(Chord::shift(Key::U)), Some(UiAction::SetTool(Tool::Ellipse)));
        assert_eq!(ui_chord(Chord::cmd(Key::B)), None);
        assert_eq!(ui_chord(Chord::plain(Key::Num3)), Some(UiAction::ZoomPreset(3)));
        assert_eq!(ui_chord(Chord::cmd(Key::Equals)), Some(UiAction::ZoomIn));
        // Engine shortcuts are not shadowed by tool keys.
        assert_eq!(ui_chord(Chord::shift(Key::N)), None);
    }
}
