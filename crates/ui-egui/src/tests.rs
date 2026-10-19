//! The lifecycle observed in the target app (`plan/aseprite/lifecycle.md`), as UI-state tests.

use serde_json::json;

use super::*;

fn with_dirty_sprite() -> PixelApp {
    let mut app = PixelApp::with_new_sprite();
    app.run("pixel.stroke", &json!({ "x0": 1, "y0": 1 }));
    app
}

#[test]
fn launch_shows_home_and_no_sprite() {
    let app = PixelApp::default();
    assert!(app.ui.home_open && app.ui.home_active);
    assert!(app.session.docs.is_empty());
    assert_eq!(app.window_title(), format!("PixelCraft v{VERSION}"));
}

#[test]
fn file_new_opens_the_dialog_with_the_last_values() {
    let mut app = PixelApp::default();
    app.ui.new_sprite.width = 64;
    app.invoke("file.new");
    assert!(matches!(&app.ui.dialog, Some(Dialog::NewSprite { width, focus: true, .. }) if width == "64"));
    assert!(app.session.docs.is_empty(), "the dialog creates nothing until OK");
}

#[test]
fn closing_an_untouched_sprite_does_not_ask_and_returns_home() {
    let mut app = PixelApp::with_new_sprite();
    assert_eq!(app.window_title(), format!("Sprite-0001 - PixelCraft v{VERSION}"));
    app.invoke("file.close");
    assert!(app.ui.dialog.is_none());
    assert!(app.session.docs.is_empty() && app.ui.home_active);
}

#[test]
fn closing_a_modified_sprite_asks_and_cancel_keeps_it() {
    let mut app = with_dirty_sprite();
    app.invoke("file.close");
    assert!(matches!(app.ui.dialog, Some(Dialog::Unsaved { quitting: false, .. })));
    assert_eq!(app.session.docs.len(), 1);
}

#[test]
fn undo_back_to_new_makes_close_silent_again() {
    let mut app = with_dirty_sprite();
    app.run("edit.undo", &json!({}));
    app.invoke("file.close");
    assert!(app.ui.dialog.is_none() && app.session.docs.is_empty());
}

#[test]
fn quitting_asks_about_each_unsaved_sprite_then_closes() {
    let mut app = with_dirty_sprite();
    app.run("file.new", &json!({}));
    app.act(UiAction::Quit);
    let Some(Dialog::Unsaved { uid, quitting: true }) = app.ui.dialog.clone() else { panic!("expected the quit warning") };
    // "Don't Save" on the only modified sprite: it closes, nothing else needs asking.
    let i = app.session.docs.iter().position(|d| d.uid == uid).unwrap();
    app.ui.dialog = None;
    app.close_now(i);
    app.act(UiAction::Quit);
    assert!(app.quit_confirmed);
}

#[test]
fn quitting_with_nothing_unsaved_closes_at_once() {
    let mut app = PixelApp::with_new_sprite();
    app.act(UiAction::Quit);
    assert!(app.ui.dialog.is_none() && app.quit_confirmed);
}

#[test]
fn closing_home_leaves_no_tabs_and_view_home_reopens_it() {
    let mut app = PixelApp::default();
    app.act(UiAction::CloseTab);
    assert!(!app.ui.home_open && !app.ui.home_active);
    app.act(UiAction::Home);
    assert!(app.ui.home_open && app.ui.home_active);
}
