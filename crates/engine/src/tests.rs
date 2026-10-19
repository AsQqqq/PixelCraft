use serde_json::json;

use super::*;

fn session_with_sprite() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({ "width": 8, "height": 8 })).unwrap();
    s
}

#[test]
fn every_command_survives_empty_params() {
    for spec in command_specs() {
        let mut empty = Session::new();
        let _ = empty.execute(spec.id, &json!({}));
        let mut s = session_with_sprite();
        let _ = s.execute(spec.id, &json!({}));
        let _ = s.execute(spec.id, &json!({ "width": -1, "x0": "a", "layer": 1e30, "color": "#zz" }));
    }
}

#[test]
fn command_ids_are_unique() {
    let mut ids: Vec<_> = command_specs().iter().map(|c| c.id).collect();
    ids.sort_unstable();
    let n = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), n);
}

#[test]
fn stroke_then_undo_and_redo() {
    let mut s = session_with_sprite();
    s.execute("pixel.stroke", &json!({ "x0": 0, "y0": 0, "x1": 7, "y1": 7, "color": "#ff0000" })).unwrap();
    let red = |s: &Session| {
        let d = s.active().unwrap();
        d.sprite.cel(0, 0).unwrap().image.rgba(geom::Point::new(3, 3), &d.sprite.palette)
    };
    assert_eq!(red(&s), color::Rgba8::opaque(255, 0, 0));
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(red(&s), color::Rgba8::TRANSPARENT);
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(red(&s), color::Rgba8::opaque(255, 0, 0));
}

#[test]
fn failed_command_keeps_the_sprite() {
    let mut s = session_with_sprite();
    assert!(s.execute("layer.toggleVisible", &json!({ "layer": 9 })).is_err());
    assert!(s.active().unwrap().undo.is_empty());
}

#[test]
fn new_layer_and_frame() {
    let mut s = session_with_sprite();
    s.execute("layer.new", &json!({})).unwrap();
    s.execute("frame.new", &json!({})).unwrap();
    let d = s.active().unwrap();
    assert_eq!((d.sprite.layers.len(), d.sprite.frames.len()), (2, 2));
    assert!(d.sprite.cel(0, 1).is_some());
}

#[test]
fn frame_navigation_wraps_and_close_moves_the_active_document() {
    let mut s = session_with_sprite();
    s.execute("frame.new", &json!({})).unwrap();
    s.execute("frame.new", &json!({})).unwrap();
    s.execute("frame.goToFirst", &json!({})).unwrap();
    s.execute("frame.goToPrevious", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().frame, 2);
    s.execute("frame.goToNext", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().frame, 0);
    assert!(s.execute("timeline.setCel", &json!({ "frame": 7 })).is_err());
    s.execute("file.new", &json!({})).unwrap();
    s.execute("file.close", &json!({})).unwrap();
    assert_eq!((s.docs.len(), s.active), (1, Some(0)));
    s.execute("file.close", &json!({})).unwrap();
    assert_eq!(s.active, None);
}

#[test]
fn undo_back_to_the_start_is_clean_and_branching_loses_the_saved_point() {
    let mut s = session_with_sprite();
    let dirty = |s: &Session| s.active().unwrap().is_dirty();
    assert!(!dirty(&s));
    s.execute("pixel.stroke", &json!({ "x0": 1, "y0": 1 })).unwrap();
    assert!(dirty(&s));
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(!dirty(&s));
    s.execute("edit.redo", &json!({})).unwrap();
    s.active_mut().unwrap().mark_saved();
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(dirty(&s));
    s.execute("pixel.stroke", &json!({ "x0": 2, "y0": 2 })).unwrap();
    assert!(dirty(&s));
    assert_eq!(s.active().unwrap().saved_at, None);
}

#[test]
fn sprite_names_count_up_for_the_session() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("file.close", &json!({})).unwrap();
    let out = s.execute("file.new", &json!({})).unwrap();
    assert_eq!(out["name"], "Sprite-0002");
}

#[test]
fn white_background_makes_an_opaque_background_layer() {
    let mut s = Session::new();
    s.execute("file.new", &json!({ "width": 4, "height": 4, "background": "white" })).unwrap();
    let d = s.active().unwrap();
    assert!(d.sprite.layers[0].background);
    assert_eq!(d.sprite.cel(0, 0).unwrap().image.rgba(geom::Point::new(3, 3), &d.sprite.palette), color::Rgba8::WHITE);
    assert!(s.execute("file.new", &json!({ "background": "plaid" })).is_err());
}
