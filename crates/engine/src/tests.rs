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

fn px(s: &Session, x: i32, y: i32) -> color::Rgba8 {
    let d = s.active().unwrap();
    d.sprite.cel(d.layer, d.frame).unwrap().image.rgba(geom::Point::new(x, y), &d.sprite.palette)
}

#[test]
fn a_whole_stroke_is_one_undo_step() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[0, 0], [7, 0], [7, 7]], "color": "#ff0000" })).unwrap();
    assert_eq!(s.active().unwrap().undo.len(), 1);
    assert_eq!(px(&s, 7, 4), color::Rgba8::opaque(255, 0, 0));
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(px(&s, 7, 4), color::Rgba8::TRANSPARENT);
}

#[test]
fn painting_stays_inside_the_selection() {
    let mut s = session_with_sprite();
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 3, "y1": 7 })).unwrap();
    s.execute("draw.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 7, "y1": 7, "filled": true, "color": "#00ff00" })).unwrap();
    assert_eq!(px(&s, 3, 3), color::Rgba8::opaque(0, 255, 0));
    assert_eq!(px(&s, 4, 3), color::Rgba8::TRANSPARENT);
    s.execute("select.deselect", &json!({})).unwrap();
    assert!(s.active().unwrap().selection.is_none());
    s.execute("select.reselect", &json!({})).unwrap();
    assert!(s.active().unwrap().selection.is_some());
}

#[test]
fn selection_modes_and_inverse() {
    let mut s = session_with_sprite();
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 1, "y1": 1 })).unwrap();
    let out = s.execute("select.shape", &json!({ "shape": "rect", "x0": 4, "y0": 4, "x1": 5, "y1": 5, "mode": "add" })).unwrap();
    assert_eq!((out["width"].as_u64(), out["height"].as_u64()), (Some(6), Some(6)));
    s.execute("select.all", &json!({})).unwrap();
    s.execute("select.inverse", &json!({})).unwrap();
    assert!(s.active().unwrap().selection.is_none(), "inverting everything selects nothing");
    assert!(s.execute("select.shape", &json!({ "shape": "blob" })).is_err());
}

#[test]
fn bucket_fills_the_connected_area() {
    let mut s = session_with_sprite();
    s.execute("draw.shape", &json!({ "shape": "line", "x0": 4, "y0": 0, "x1": 4, "y1": 7 })).unwrap();
    s.execute("draw.fill", &json!({ "x": 0, "y": 0, "color": "#0000ff" })).unwrap();
    assert_eq!(px(&s, 3, 5), color::Rgba8::opaque(0, 0, 255));
    assert_eq!(px(&s, 5, 5), color::Rgba8::TRANSPARENT);
}

#[test]
fn move_shifts_the_selected_pixels_and_the_selection() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[1, 1]], "color": "#ff0000" })).unwrap();
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 2, "y1": 2 })).unwrap();
    s.execute("cel.moveContent", &json!({ "dx": 3, "dy": 2 })).unwrap();
    assert_eq!(px(&s, 4, 3), color::Rgba8::opaque(255, 0, 0));
    assert_eq!(px(&s, 1, 1), color::Rgba8::TRANSPARENT);
    assert_eq!(s.active().unwrap().selection.as_ref().unwrap().bounds(), geom::Rect::new(3, 2, 3, 3));
}

#[test]
fn drawing_on_a_new_layer_keeps_the_drawing() {
    let mut s = session_with_sprite();
    s.execute("layer.new", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().layer, 1);
    s.execute("draw.stroke", &json!({ "points": [[2, 2], [5, 2]], "color": "#00ff00" })).unwrap();
    s.execute("draw.shape", &json!({ "shape": "rect", "x0": 0, "y0": 5, "x1": 3, "y1": 7, "filled": true, "color": "#0000ff" })).unwrap();
    let d = s.active().unwrap();
    let cel = d.sprite.cel(1, 0).expect("the new layer has a cel once painted");
    assert_eq!(cel.image.rgba(geom::Point::new(4, 2), &d.sprite.palette), color::Rgba8::opaque(0, 255, 0));
    // Layer 1 stays untouched; the composite shows both.
    assert_eq!(d.sprite.cel(0, 0).unwrap().image.rgba(geom::Point::new(4, 2), &d.sprite.palette), color::Rgba8::TRANSPARENT);
    let flat = compose::render_frame(&d.sprite, 0).unwrap();
    assert_eq!(flat.rgba(geom::Point::new(1, 6), &d.sprite.palette), color::Rgba8::opaque(0, 0, 255));
    // Undo removes the painting, and the bucket works on an empty layer too.
    s.execute("layer.new", &json!({})).unwrap();
    s.execute("draw.fill", &json!({ "x": 0, "y": 0, "color": "#ff0000" })).unwrap();
    let d = s.active().unwrap();
    assert_eq!(d.sprite.cel(2, 0).unwrap().image.rgba(geom::Point::new(7, 7), &d.sprite.palette), color::Rgba8::opaque(255, 0, 0));
}

#[test]
fn delete_clears_the_selected_pixels_in_every_colour_mode() {
    for mode in ["rgba", "grayscale", "indexed"] {
        let mut s = Session::new();
        s.execute("file.new", &json!({ "width": 8, "height": 8, "colorMode": mode })).unwrap();
        s.execute("draw.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 7, "y1": 7, "filled": true, "color": "#ffffff", "index": 2 }))
            .unwrap();
        s.execute("select.shape", &json!({ "shape": "rect", "x0": 2, "y0": 2, "x1": 4, "y1": 4 })).unwrap();
        let out = s.execute("edit.clear", &json!({})).unwrap();
        assert_eq!(out["cleared"], 9, "{mode}");
        let raw = |s: &Session, x, y| {
            let d = s.active().unwrap();
            d.sprite.cel(0, 0).unwrap().image.pixel(geom::Point::new(x, y)).unwrap().to_vec()
        };
        assert!(raw(&s, 3, 3).iter().all(|&b| b == 0), "{mode}: inside is transparent");
        assert!(raw(&s, 5, 5).iter().any(|&b| b != 0), "{mode}: outside is kept");
        assert!(s.active().unwrap().selection.is_some(), "{mode}: the selection stays");
        // One undo step brings the pixels back.
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(raw(&s, 3, 3).iter().any(|&b| b != 0), "{mode}: undo restores");
    }
}

#[test]
fn delete_without_a_selection_removes_the_cel_and_a_background_takes_the_colour() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[0, 0], [7, 0]], "color": "#ff0000" })).unwrap();
    s.execute("edit.clear", &json!({})).unwrap();
    assert!(s.active().unwrap().sprite.cel(0, 0).is_none());
    // Nothing left to clear: no extra undo step.
    let steps = s.active().unwrap().undo.len();
    s.execute("edit.clear", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().undo.len(), steps);

    let mut s = Session::new();
    s.execute("file.new", &json!({ "width": 4, "height": 4, "background": "white" })).unwrap();
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 1, "y1": 1 })).unwrap();
    s.execute("edit.clear", &json!({ "color": "#00ff00" })).unwrap();
    assert_eq!(px(&s, 1, 1), color::Rgba8::opaque(0, 255, 0));
    assert_eq!(px(&s, 3, 3), color::Rgba8::WHITE);
}

#[test]
fn delete_refuses_a_locked_layer() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[0, 0]], "color": "#ff0000" })).unwrap();
    Arc::make_mut(&mut s.active_mut().unwrap().sprite).layers[0].locked = true;
    assert!(s.execute("edit.clear", &json!({})).is_err());
    assert_eq!(px(&s, 0, 0), color::Rgba8::opaque(255, 0, 0));
}

#[test]
fn delete_frame_removes_its_cels_and_keeps_the_rest() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[0, 0]], "color": "#ff0000" })).unwrap();
    s.execute("frame.new", &json!({})).unwrap();
    s.execute("draw.stroke", &json!({ "points": [[1, 1]], "color": "#00ff00" })).unwrap();
    s.execute("frame.new", &json!({})).unwrap();
    s.execute("draw.stroke", &json!({ "points": [[2, 2]], "color": "#0000ff" })).unwrap();
    // Frames: 0 red, 1 red+green, 2 red+green+blue; delete the middle one.
    s.execute("timeline.setCel", &json!({ "frame": 1 })).unwrap();
    let out = s.execute("frame.remove", &json!({})).unwrap();
    assert_eq!(out["frames"], 2);
    let d = s.active().unwrap();
    assert_eq!(d.frame, 1, "the next frame becomes current");
    assert_eq!(d.sprite.cels.len(), 2);
    assert_eq!(px(&s, 2, 2), color::Rgba8::opaque(0, 0, 255), "frame 3 moved to frame 2");
    // The last frame: the current frame steps back.
    s.execute("frame.remove", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().frame, 0);
    assert_eq!(px(&s, 0, 0), color::Rgba8::opaque(255, 0, 0));
    // The only frame can't go; undo brings the others back.
    assert!(s.execute("frame.remove", &json!({})).is_err());
    assert!(!command_specs().iter().find(|c| c.id == "frame.remove").unwrap().info(&s).enabled);
    s.execute("edit.undo", &json!({})).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().sprite.frames.len(), 3);
    assert!(s.execute("frame.remove", &json!({ "frame": 9 })).is_err());
}

#[test]
fn delete_frame_shrinks_tags() {
    let mut s = session_with_sprite();
    for _ in 0..4 {
        s.execute("frame.new", &json!({})).unwrap();
    }
    let sp = Arc::make_mut(&mut s.active_mut().unwrap().sprite);
    sp.tags = vec![
        doc::Tag { name: "walk".into(), from: 1, to: 3, direction: Default::default() },
        doc::Tag { name: "blink".into(), from: 2, to: 2, direction: Default::default() },
    ];
    s.execute("frame.remove", &json!({ "frame": 2 })).unwrap();
    let tags = &s.active().unwrap().sprite.tags;
    assert_eq!(tags.len(), 1, "a tag on the removed frame alone goes");
    assert_eq!((tags[0].from, tags[0].to), (1, 2));
}

#[test]
fn file_open_reads_an_image_into_a_new_tab_and_reuses_an_open_one() {
    let dir = std::env::temp_dir().join(format!("pixelcraft-open-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cat.png");
    let mut img = image::RgbaImage::new(5, 3);
    img.put_pixel(4, 2, image::Rgba([255, 0, 0, 255]));
    img.save(&path).unwrap();
    let path = path.to_string_lossy().into_owned();

    let mut s = session_with_sprite();
    let out = s.execute("file.open", &json!({ "path": path })).unwrap();
    assert_eq!(out["name"], "cat.png");
    assert_eq!((out["width"].as_u64(), out["height"].as_u64()), (Some(5), Some(3)));
    let d = s.active().unwrap();
    assert_eq!(d.name, "cat.png");
    assert_eq!(d.path.as_deref(), Some(path.as_str()));
    assert!(!d.is_dirty(), "a file that was just opened has no unsaved changes");
    assert_eq!(px(&s, 4, 2), color::Rgba8::opaque(255, 0, 0));
    assert_eq!(s.docs.len(), 2);
    // Opening it again switches to its tab instead of a second copy.
    s.active = Some(0);
    let again = s.execute("file.open", &json!({ "path": path })).unwrap();
    assert_eq!(again["alreadyOpen"], true);
    assert_eq!((s.docs.len(), s.active), (2, Some(1)));
    // Errors, not panics: no path, a missing file, a file that isn't an image.
    assert!(s.execute("file.open", &json!({})).is_err());
    assert!(s.execute("file.open", &json!({ "path": dir.join("nope.png").to_string_lossy() })).is_err());
    let text = dir.join("notes.png");
    std::fs::write(&text, "hello").unwrap();
    assert!(s.execute("file.open", &json!({ "path": text.to_string_lossy() })).is_err());
    assert_eq!(s.docs.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

fn raw_px(s: &Session, layer: usize, frame: usize, x: i32, y: i32) -> Option<Vec<u8>> {
    let d = s.active().unwrap();
    d.sprite.cel(layer, frame).and_then(|c| c.image.pixel(geom::Point::new(x, y)).map(<[u8]>::to_vec))
}

#[test]
fn copy_and_paste_in_every_colour_mode() {
    for mode in ["rgba", "grayscale", "indexed"] {
        let mut s = Session::new();
        s.execute("file.new", &json!({ "width": 8, "height": 8, "colorMode": mode })).unwrap();
        s.execute("draw.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 1, "y1": 1, "filled": true, "color": "#ffffff", "index": 5 }))
            .unwrap();
        s.execute("select.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 1, "y1": 1 })).unwrap();
        let out = s.execute("edit.copy", &json!({})).unwrap();
        assert_eq!((out["width"].as_u64(), out["height"].as_u64()), (Some(2), Some(2)), "{mode}");
        let before = raw_px(&s, 0, 0, 0, 0);
        let pasted = s.execute("edit.paste", &json!({ "x": 5, "y": 5 })).unwrap();
        assert_eq!((pasted["x"].as_i64(), pasted["y"].as_i64()), (Some(5), Some(5)), "{mode}");
        assert_eq!(raw_px(&s, 0, 0, 6, 6), before, "{mode}: the copy lands at 5,5");
        let b = s.active().unwrap().selection.as_ref().unwrap().bounds();
        assert_eq!((b.x, b.y, b.w, b.h), (5, 5, 2, 2), "{mode}: the pasted pixels are selected");
        s.execute("edit.undo", &json!({})).unwrap();
        assert_ne!(raw_px(&s, 0, 0, 6, 6), before, "{mode}: one undo step");
    }
}

#[test]
fn cut_copies_then_clears_and_paste_goes_to_the_current_layer_and_frame() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[2, 2], [3, 2]], "color": "#ff0000" })).unwrap();
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 2, "y0": 2, "x1": 3, "y1": 2 })).unwrap();
    s.execute("edit.cut", &json!({})).unwrap();
    assert_eq!(px(&s, 2, 2), color::Rgba8::TRANSPARENT, "cut clears");
    assert!(s.clipboard.is_some());
    // Another layer and another frame: the paste lands there, at the copied spot.
    s.execute("layer.new", &json!({})).unwrap();
    s.execute("frame.new", &json!({})).unwrap();
    s.execute("edit.paste", &json!({})).unwrap();
    let d = s.active().unwrap();
    assert_eq!((d.layer, d.frame), (1, 1));
    assert_eq!(raw_px(&s, 1, 1, 3, 2), Some(vec![255, 0, 0, 255]));
    assert_eq!(raw_px(&s, 0, 1, 3, 2), Some(vec![0, 0, 0, 0]), "layer 1 untouched");
}

#[test]
fn paste_scales_rotates_and_crosses_sprites() {
    let mut s = session_with_sprite();
    s.execute("draw.stroke", &json!({ "points": [[0, 0], [1, 0]], "color": "#00ff00" })).unwrap();
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 0, "y0": 0, "x1": 1, "y1": 0 })).unwrap();
    s.execute("edit.copy", &json!({})).unwrap();
    // 2×1 stretched to 4×2.
    let out = s.execute("edit.paste", &json!({ "x": 0, "y": 4, "width": 4, "height": 2 })).unwrap();
    assert_eq!((out["width"].as_u64(), out["height"].as_u64()), (Some(4), Some(2)));
    assert_eq!(px(&s, 3, 5), color::Rgba8::opaque(0, 255, 0));
    // A quarter turn: 2×1 becomes 1×2.
    let out = s.execute("edit.paste", &json!({ "x": 6, "y": 0, "angle": 90 })).unwrap();
    assert_eq!((out["width"].as_u64(), out["height"].as_u64()), (Some(1), Some(2)));
    // Into a smaller sprite where the copied spot is off the canvas: centred.
    s.execute("select.shape", &json!({ "shape": "rect", "x0": 6, "y0": 6, "x1": 7, "y1": 7 })).unwrap();
    s.execute("edit.copy", &json!({})).unwrap();
    s.execute("file.new", &json!({ "width": 4, "height": 4 })).unwrap();
    let out = s.execute("edit.paste", &json!({})).unwrap();
    assert_eq!((out["x"].as_i64(), out["y"].as_i64()), (Some(1), Some(1)));
}

#[test]
fn paste_refuses_what_it_cannot_do() {
    let mut s = session_with_sprite();
    assert!(s.execute("edit.paste", &json!({})).is_err(), "nothing copied");
    s.execute("edit.copy", &json!({})).unwrap();
    for bad in [json!({ "width": 0 }), json!({ "width": 60000, "height": 60000 }), json!({ "angle": "left" }), json!({ "x": "a" })] {
        assert!(s.execute("edit.paste", &bad).is_err(), "{bad}");
    }
    assert!(s.execute("edit.paste", &json!({ "x": -100000, "y": 2147483000 })).is_ok(), "far away is just off the canvas");
    Arc::make_mut(&mut s.active_mut().unwrap().sprite).layers[0].locked = true;
    assert!(s.execute("edit.paste", &json!({})).is_err());
    assert!(s.execute("edit.cut", &json!({})).is_err());
}
