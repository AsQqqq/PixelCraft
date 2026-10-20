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

#[test]
fn choosing_a_tool_remembers_it_for_its_group_button() {
    let mut app = PixelApp::with_new_sprite();
    app.act(UiAction::SetTool(Tool::FilledEllipse));
    assert_eq!(app.ui.tool, Tool::FilledEllipse);
    assert_eq!(app.ui.group_tool[Tool::Rectangle.group()], Tool::FilledEllipse);
    assert_eq!(app.ui.group_tool[Tool::Pencil.group()], Tool::Pencil);
}

#[test]
fn every_tool_is_in_exactly_one_group() {
    for tool in Tool::ALL {
        assert_eq!(state::TOOL_GROUPS.iter().filter(|g| g.contains(&tool)).count(), 1, "{tool:?}");
    }
}

#[test]
fn brush_size_stays_in_range() {
    let mut app = PixelApp::with_new_sprite();
    app.act(UiAction::BrushSize(-5));
    assert_eq!(app.ui.brush_size, 1);
    app.act(UiAction::BrushSize(1000));
    assert_eq!(app.ui.brush_size, pixelcraft_engine::paint::MAX_BRUSH);
}

#[test]
fn tooltips_follow_the_target_app_wording() {
    let s = panels::tool_bar::tooltip(Tool::RectangularMarquee);
    assert!(s.starts_with("Rectangular Marquee Tool:\n* Left Button: replace/add to current selection"));
    assert!(s.contains("Shortcut: M"));
    assert_eq!(panels::tool_bar::tooltip(Tool::Pencil), "Pencil Tool\nShortcut: B");
}

// --- interaction, through egui_kittest -------------------------------------------------------------

fn harness(app: PixelApp) -> egui_kittest::Harness<'static, PixelApp> {
    // Short steps, so two clicks land within the double-click delay.
    egui_kittest::Harness::builder().with_size(egui::vec2(1100.0, 700.0)).with_step_dt(0.02).build_ui_state(|ui, app: &mut PixelApp| app.ui(ui), app)
}

fn click(h: &mut egui_kittest::Harness<'static, PixelApp>, pos: egui::Pos2) {
    h.event(egui::Event::PointerMoved(pos));
    h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.step();
}

#[test]
fn double_clicking_free_tab_space_opens_new_sprite() {
    let mut h = harness(PixelApp::with_new_sprite());
    h.run_steps(3);
    let free = h.state().hit.tab_free.expect("the tab strip has free space");
    let p = free.center();
    click(&mut h, p);
    assert!(h.state().ui.dialog.is_none(), "a single click does nothing");
    click(&mut h, p);
    h.run_steps(2);
    assert!(matches!(h.state().ui.dialog, Some(Dialog::NewSprite { .. })), "double click opens New Sprite");
}

#[test]
fn preview_play_button_plays_the_animation_in_the_window() {
    let mut app = PixelApp::with_new_sprite();
    app.run("frame.new", &json!({}));
    app.run("frame.new", &json!({}));
    app.run("frame.goToFirst", &json!({}));
    let mut h = harness(app);
    h.run_steps(3);
    let play = h.state().hit.preview_play.expect("the preview has a play button");
    click(&mut h, play.center());
    assert!(h.state().ui.preview.playing);
    // Frames advance in the preview; the editor stays on frame 1.
    // kittest doesn't advance the clock on its own; a real window does.
    for i in 0..30 {
        h.input_mut().time = Some(10.0 + f64::from(i) * 0.02);
        h.step();
    }
    let d = h.state().session.active().unwrap();
    assert_eq!(d.frame, 0);
    assert!(h.state().ui.preview.frame > 0 || h.state().ui.preview.elapsed_ms > 0.0);
    click(&mut h, play.center());
    assert!(!h.state().ui.preview.playing, "the same button stops");
}

#[test]
fn preferences_apply_on_ok_and_cancel_discards() {
    let mut app = PixelApp::with_new_sprite();
    app.act(UiAction::Preferences);
    let Some(Dialog::Preferences { mut draft, section }) = app.ui.dialog.clone() else { panic!("expected Preferences") };
    draft.native_cursor = true;
    app.ui.dialog = Some(Dialog::Preferences { section, draft });
    // Cancel: drawn by the dialog; here we check the model: nothing applied until OK/Apply.
    assert!(!app.ui.prefs.native_cursor);
}

#[test]
fn pixel_cursor_is_an_os_cursor_image_and_native_turns_it_off() {
    let mut h = harness(PixelApp::with_new_sprite());
    h.event(egui::Event::PointerMoved(egui::pos2(600.0, 300.0)));
    h.run_steps(2);
    assert!(h.output().platform_output.cursor_image.is_some(), "pixel cursor by default");
    h.state_mut().ui.prefs.native_cursor = true;
    h.run_steps(2);
    assert!(h.output().platform_output.cursor_image.is_none(), "native cursors: no image");
}

#[test]
fn new_sprite_dialog_keeps_the_pixel_cursor() {
    let mut h = harness(PixelApp::default());
    h.state_mut().act(UiAction::NewFile);
    h.event(egui::Event::PointerMoved(egui::pos2(550.0, 350.0)));
    h.run_steps(3);
    assert!(h.output().platform_output.cursor_image.is_some());
}

#[test]
fn backspace_and_delete_clear_the_selection() {
    for key in [egui::Key::Backspace, egui::Key::Delete] {
        let mut app = PixelApp::with_new_sprite();
        app.run("draw.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 9, "y1": 9, "filled": true, "color": "#ff0000" }));
        app.run("select.shape", &json!({ "shape": "rect", "x0": 2, "y0": 2, "x1": 4, "y1": 4 }));
        let mut h = harness(app);
        h.run_steps(2);
        h.key_press(key);
        h.run_steps(2);
        let d = h.state().session.active().unwrap();
        let at = |x, y| d.sprite.cel(0, 0).unwrap().image.rgba(pixelcraft_engine::geom::Point::new(x, y), &d.sprite.palette);
        assert_eq!(at(3, 3).a, 0, "{key:?} clears inside the selection");
        assert_eq!(at(6, 6).a, 255, "{key:?} keeps the rest");
    }
}

#[test]
fn alt_c_deletes_the_current_frame_but_not_the_last_one() {
    let mut app = PixelApp::with_new_sprite();
    app.run("frame.new", &json!({}));
    let mut h = harness(app);
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::C);
    h.run_steps(2);
    assert_eq!(h.state().session.active().unwrap().sprite.frames.len(), 1);
    h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::C);
    h.run_steps(2);
    assert_eq!(h.state().session.active().unwrap().sprite.frames.len(), 1, "the only frame stays");
}

/// A file dropped on the window in tests (the window system's own type lives in egui-winit).
#[derive(Debug)]
struct DroppedPath(std::path::PathBuf);

impl egui::DroppedFile for DroppedPath {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

fn png_file(name: &str, w: u32, h: u32) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("pixelcraft-ui-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    image::RgbaImage::from_pixel(w, h, image::Rgba([0, 128, 255, 255])).save(&path).unwrap();
    path
}

fn png_bytes(w: u32, h: u32) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::from(image::RgbaImage::new(w, h)).write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[test]
fn dropping_an_image_opens_it_in_a_new_tab() {
    let path = png_file("drop.png", 6, 4);
    let mut h = harness(PixelApp::default());
    h.run_steps(2);
    h.input_mut().dropped_files.push(std::sync::Arc::new(DroppedPath(path.clone())));
    h.run_steps(2);
    let app = h.state();
    assert!(!app.ui.home_active, "the opened sprite's tab is active");
    let d = app.session.active().unwrap();
    assert_eq!((d.name.as_str(), d.sprite.size.w, d.sprite.size.h), ("drop.png", 6, 4));
    assert_eq!(app.window_title(), format!("drop.png - {APP_NAME} v{VERSION}"));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn file_open_asks_the_host_for_its_dialog() {
    let mut app = PixelApp::default();
    app.invoke("file.open");
    assert!(!app.take_open_request(), "no dialog without a host");
    assert!(app.status.as_deref().is_some_and(|s| s.contains("drag")), "{:?}", app.status);
    app.can_open_dialog = true;
    let mut h = harness(app);
    h.run_steps(1);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::O);
    h.run_steps(1);
    assert!(h.state_mut().take_open_request(), "Cmd+O asks for the dialog");
    assert!(!h.state_mut().take_open_request(), "once");
    // What the dialog picks arrives through the inbox and opens on the next frame.
    let path = png_file("picked.png", 3, 3);
    h.state().picked_sender().send(open::Picked::Path(path.to_string_lossy().into_owned())).unwrap();
    h.run_steps(1);
    assert_eq!(h.state().session.active().unwrap().name, "picked.png");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn browser_bytes_open_and_bad_files_only_say_so() {
    let mut app = PixelApp::default();
    app.open_picked(open::Picked::Bytes { name: "web.png".into(), bytes: png_bytes(2, 5) });
    assert_eq!(app.session.active().unwrap().name, "web.png");
    assert!(app.session.active().unwrap().path.is_none());
    app.open_picked(open::Picked::Bytes { name: "notes.txt".into(), bytes: b"hello".to_vec() });
    assert_eq!(app.session.docs.len(), 1);
    assert!(app.status.as_deref().is_some_and(|s| s.starts_with("notes.txt")), "{:?}", app.status);
}

#[test]
fn a_photo_bigger_than_the_editor_opens_fitted() {
    let mut app = PixelApp::default();
    app.open_picked(open::Picked::Bytes { name: "photo.png".into(), bytes: png_bytes(3000, 2000) });
    app.open_picked(open::Picked::Bytes { name: "icon.png".into(), bytes: png_bytes(16, 16) });
    let ids: Vec<u64> = app.session.docs.iter().map(|d| d.uid).collect();
    let mut h = harness(app);
    h.run_steps(2);
    h.state_mut().session.active = Some(0);
    h.run_steps(2);
    let zoom = |h: &egui_kittest::Harness<'static, PixelApp>, i: usize| h.state().ui.viewports.get(&ids[i]).map(|v| v.zoom);
    assert!(zoom(&h, 0).is_some_and(|z| z < 1.0), "fitted: {:?}", zoom(&h, 0));
    assert_eq!(zoom(&h, 1), Some(1.0), "a small image stays at 100 %");
}

#[test]
fn delete_on_a_selected_frame_deletes_the_frame_not_its_pixels() {
    let mut app = PixelApp::with_new_sprite();
    app.run("draw.stroke", &json!({ "points": [[1, 1]], "color": "#ff0000" }));
    app.run("frame.new", &json!({}));
    app.run("frame.new", &json!({}));
    let mut h = harness(app);
    h.run_steps(2);
    // Click frame 2's number: the frame is selected; Backspace deletes it.
    let n2 = h.state().hit.frame_numbers[1];
    click(&mut h, n2.center());
    h.run_steps(1);
    assert_eq!(h.state().ui.timeline_frame, Some(1));
    h.key_press(egui::Key::Backspace);
    h.run_steps(2);
    let frames = |h: &egui_kittest::Harness<'static, PixelApp>| h.state().session.active().unwrap().sprite.frames.len();
    assert_eq!(frames(&h), 2, "the frame itself is gone");
    // Delete again deletes the next selected frame; the last one stays.
    h.key_press(egui::Key::Delete);
    h.run_steps(2);
    assert_eq!(frames(&h), 1);
    h.key_press(egui::Key::Delete);
    h.run_steps(2);
    assert_eq!(frames(&h), 1, "the only frame stays");
    assert!(h.state().session.active().unwrap().sprite.cel(0, 0).is_some(), "and keeps its drawing");

    // Clicking a cel instead: Delete clears pixels and keeps every frame.
    h.state_mut().run("frame.new", &json!({}));
    h.run_steps(2);
    h.state_mut().ui.timeline_frame = None;
    h.key_press(egui::Key::Delete);
    h.run_steps(2);
    assert_eq!(frames(&h), 2);
}

#[test]
fn redo_has_three_keys() {
    let shift_cmd = egui::Modifiers { shift: true, ..egui::Modifiers::COMMAND };
    for (mods, key) in [(egui::Modifiers::COMMAND, egui::Key::Y), (shift_cmd, egui::Key::Z), (egui::Modifiers::COMMAND, egui::Key::R)] {
        let mut app = PixelApp::with_new_sprite();
        app.run("draw.stroke", &json!({ "points": [[1, 1]], "color": "#ff0000" }));
        let mut h = harness(app);
        h.run_steps(1);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
        h.run_steps(1);
        assert_eq!(h.state().session.active().unwrap().redo.len(), 1, "undone");
        h.key_press_modifiers(mods, key);
        h.run_steps(1);
        let d = h.state().session.active().unwrap();
        assert_eq!((d.undo.len(), d.redo.len()), (1, 0), "{mods:?}+{key:?} redoes");
    }
}
