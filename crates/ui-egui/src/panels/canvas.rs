//! The sprite editor: the current frame, composited by the engine, drawn over a transparency
//! checkerboard at the viewport's zoom and pan, with the tools' hover and drag feedback on top.
//!
//! Navigation follows the target app: the wheel zooms one ladder step per notch around the
//! pointer (Shift+wheel scrolls sideways), Space or the Hand tool or the middle button pans.
//! Every tool commits **one** engine command when the mouse is released (one undo step), and
//! shows a live preview while dragging. Hover feedback (`plan/aseprite/tools.md`): painting tools
//! show the brush filled with the colour they would paint, outlined in negative black/white; the
//! crosshair is the cursor itself; the selection shows marching ants.

use std::collections::HashSet;
use std::sync::Arc;

use egui::{Color32, Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, Rect, Stroke, Vec2, pos2, vec2};
use pixelcraft_engine::color::Rgba8;
use pixelcraft_engine::compose::render_frame;
use pixelcraft_engine::doc::Sprite;
use pixelcraft_engine::geom::{self, Point};
use pixelcraft_engine::paint;
use pixelcraft_engine::raster::{Edge, Image, Mask, SelectMode};
use serde_json::{Value as Json, json};

use crate::theme::{Tokens, size};
use crate::zoom::{self, Viewport};
use crate::{PixelApp, Tool};

/// Points of trackpad scrolling that make one zoom step.
const POINTS_PER_STEP: f32 = 50.0;
/// More preview pixels than this are not drawn while dragging (the stroke still commits).
const MAX_PREVIEW_PIXELS: usize = 200_000;

/// A drag in progress with a tool, in canvas pixels.
#[derive(Clone, Debug)]
struct Drag {
    tool: Tool,
    secondary: bool,
    mods: Modifiers,
    start: Point,
    last: Point,
    /// Freehand path (pencil, eraser, contour, lasso) or spray dots.
    path: Vec<Point>,
}

/// What the canvas remembers between frames.
#[derive(Default)]
pub struct CanvasCache {
    /// The sprite and frame the texture shows; holding the `Arc` keeps its address from being reused.
    key: Option<(Arc<Sprite>, usize)>,
    texture: Option<egui::TextureHandle>,
    /// The composited frame, for the eyedropper and the negative outline colour.
    image: Option<Image>,
    /// The editor area and the pointer in it, for keyboard zoom.
    view: Option<Rect>,
    pointer: Option<Pos2>,
    wheel: f32,
    drag: Option<Drag>,
    /// Vertices of a polygon or polygonal lasso being clicked in.
    poly: Vec<Point>,
    poly_tool: Option<Tool>,
    /// The selection the marching ants were computed for, and its outline.
    ants: Option<(Mask, Vec<Edge>)>,
    seed: u64,
}

fn active_size(app: &PixelApp) -> Option<(u64, Vec2)> {
    let d = app.active_doc().and_then(|i| app.session.docs.get(i))?;
    Some((d.uid, vec2(d.sprite.size.w as f32, d.sprite.size.h as f32)))
}

pub fn viewport_mut(app: &mut PixelApp) -> Option<&mut Viewport> {
    let (uid, _) = active_size(app)?;
    app.ui.viewports.get_mut(&uid)
}

/// The current frame's texture and the sprite size, for the Preview window.
pub fn texture(app: &PixelApp) -> Option<(egui::TextureId, Vec2)> {
    let (_, size) = active_size(app)?;
    Some((app.canvas.texture.as_ref()?.id(), size))
}

/// The point keyboard zoom keeps fixed: the pointer when it is over the editor, else the centre.
fn anchor(app: &PixelApp, view: Rect) -> Pos2 {
    app.canvas.pointer.filter(|p| view.contains(*p)).unwrap_or(view.center())
}

pub fn zoom_step(app: &mut PixelApp, inward: bool) {
    let Some(vp) = viewport_mut(app) else { return };
    let z = if inward { zoom::step_in(vp.zoom) } else { zoom::step_out(vp.zoom) };
    zoom_set(app, z);
}

pub fn zoom_set(app: &mut PixelApp, z: f32) {
    let (Some((uid, sprite)), Some(view)) = (active_size(app), app.canvas.view) else { return };
    let a = anchor(app, view);
    if let Some(vp) = app.ui.viewports.get_mut(&uid) {
        vp.zoom_to(z, view, sprite, a);
    }
}

pub fn fit(app: &mut PixelApp) {
    let (Some((uid, sprite)), Some(view)) = (active_size(app), app.canvas.view) else { return };
    if let Some(vp) = app.ui.viewports.get_mut(&uid) {
        vp.fit(view, sprite);
    }
}

/// Wheel events this frame: unit, delta (the direction content moves) and modifiers.
fn wheel_events(ui: &egui::Ui) -> Vec<(MouseWheelUnit, Vec2, Modifiers)> {
    ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                Event::MouseWheel { unit, delta, modifiers, .. } => Some((*unit, *delta, *modifiers)),
                _ => None,
            })
            .collect()
    })
}

fn c32(c: Rgba8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

fn pts(points: &[Point]) -> Json {
    Json::Array(points.iter().map(|p| json!([p.x, p.y])).collect())
}

/// How a selection tool combines: modifiers first (Shift add, Alt+Shift subtract, Cmd+Shift
/// intersect), right button subtracts, else the context bar's mode.
fn select_mode(app: &PixelApp, secondary: bool, m: Modifiers) -> SelectMode {
    if secondary || (m.alt && m.shift) {
        SelectMode::Subtract
    } else if m.command && m.shift {
        SelectMode::Intersect
    } else if m.shift {
        SelectMode::Add
    } else {
        app.ui.select_mode
    }
}

fn mode_name(m: SelectMode) -> &'static str {
    match m {
        SelectMode::Replace => "replace",
        SelectMode::Add => "add",
        SelectMode::Subtract => "subtract",
        SelectMode::Intersect => "intersect",
    }
}

/// A tiny xorshift for the spray (no dependency, deterministic per session).
fn next_rand(seed: &mut u64) -> u64 {
    if *seed == 0 {
        *seed = 0x9E37_79B9_7F4A_7C15;
    }
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

/// Colour param for the engine: the colour the button paints with.
fn paint_color(app: &PixelApp, secondary: bool) -> String {
    if secondary { app.ui.bg.to_hex() } else { app.ui.fg.to_hex() }
}

/// Commits a finished drag as one engine command.
fn commit_drag(app: &mut PixelApp, d: Drag) {
    let (a, b) = (d.start, d.last);
    let color = paint_color(app, d.secondary);
    let size = app.ui.brush_size;
    match d.tool {
        Tool::Pencil => {
            app.run("draw.stroke", &json!({ "points": pts(&d.path), "size": size, "pixelPerfect": app.ui.pixel_perfect, "color": color }));
        }
        Tool::Spray => {
            app.run("draw.stroke", &json!({ "points": pts(&d.path), "size": 1, "connect": false, "color": color }));
        }
        Tool::Eraser => {
            app.run("draw.stroke", &json!({ "points": pts(&d.path), "size": size, "erase": true }));
        }
        Tool::Line => {
            app.run("draw.shape", &json!({ "shape": "line", "x0": a.x, "y0": a.y, "x1": b.x, "y1": b.y, "size": size, "color": color }));
        }
        Tool::Rectangle | Tool::FilledRectangle | Tool::Ellipse | Tool::FilledEllipse => {
            let shape = if matches!(d.tool, Tool::Rectangle | Tool::FilledRectangle) { "rect" } else { "ellipse" };
            let filled = matches!(d.tool, Tool::FilledRectangle | Tool::FilledEllipse);
            app.run("draw.shape", &json!({ "shape": shape, "x0": a.x, "y0": a.y, "x1": b.x, "y1": b.y, "filled": filled, "color": color }));
        }
        Tool::Contour => {
            if d.path.len() > 1 {
                app.run("draw.shape", &json!({ "shape": "polygon", "points": pts(&d.path), "filled": true, "color": color }));
            }
        }
        Tool::RectangularMarquee | Tool::EllipticalMarquee | Tool::Lasso => {
            let mode = select_mode(app, d.secondary, d.mods);
            // A click without dragging deselects, like the target app.
            if a == b && d.path.len() <= 1 && mode == SelectMode::Replace {
                app.run("select.deselect", &json!({}));
                return;
            }
            let params = match d.tool {
                Tool::Lasso => json!({ "shape": "polygon", "points": pts(&d.path), "mode": mode_name(mode) }),
                Tool::EllipticalMarquee => json!({ "shape": "ellipse", "x0": a.x, "y0": a.y, "x1": b.x, "y1": b.y, "mode": mode_name(mode) }),
                _ => json!({ "shape": "rect", "x0": a.x, "y0": a.y, "x1": b.x, "y1": b.y, "mode": mode_name(mode) }),
            };
            app.run("select.shape", &params);
        }
        Tool::Move => {
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            if dx != 0 || dy != 0 {
                app.run("cel.moveContent", &json!({ "dx": dx, "dy": dy }));
            }
        }
        _ => {}
    }
}

/// Closes and commits the polygon or polygonal lasso being clicked in.
fn commit_poly(app: &mut PixelApp, mods: Modifiers) {
    let poly = std::mem::take(&mut app.canvas.poly);
    let tool = app.canvas.poly_tool.take();
    if poly.len() < 2 {
        return;
    }
    match tool {
        Some(Tool::Polygon) => {
            let color = paint_color(app, false);
            app.run("draw.shape", &json!({ "shape": "polygon", "points": pts(&poly), "filled": true, "color": color }));
        }
        Some(Tool::PolygonalLasso) => {
            let mode = select_mode(app, false, mods);
            app.run("select.shape", &json!({ "shape": "polygon", "points": pts(&poly), "mode": mode_name(mode) }));
        }
        _ => {}
    }
}

pub fn show(app: &mut PixelApp, ui: &mut egui::Ui, t: &Tokens) {
    let view = ui.max_rect();
    app.canvas.view = Some(view);
    let Some(d) = app.active_doc().and_then(|i| app.session.docs.get(i)) else {
        app.ui.hover_pixel = None;
        return;
    };
    let (uid, sprite, frame) = (d.uid, Arc::clone(&d.sprite), d.frame);
    let ssize = vec2(sprite.size.w as f32, sprite.size.h as f32);

    // A new document opens at 100 %, centred, like the target app (Cmd+0 fits it).
    let mut vp = *app.ui.viewports.entry(uid).or_default();

    // Re-render only when the sprite or the frame changed.
    let fresh = app.canvas.key.as_ref().is_some_and(|(s, f)| Arc::ptr_eq(s, &sprite) && *f == frame);
    if !fresh && let Ok(img) = render_frame(&sprite, frame) {
        let px = [img.size().w as usize, img.size().h as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(px, img.bytes());
        // Sharp pixels when zoomed in, filtered when zoomed out.
        let options = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Linear,
            ..egui::TextureOptions::NEAREST
        };
        app.canvas.texture = Some(ui.ctx().load_texture("canvas", color, options));
        app.canvas.image = Some(img);
        app.canvas.key = Some((Arc::clone(&sprite), frame));
    }

    let resp = ui.allocate_rect(view, egui::Sense::click_and_drag());
    let pointer = resp.hover_pos().or(resp.interact_pointer_pos());
    app.canvas.pointer = pointer;
    let inside = |(x, y): (i32, i32)| x >= 0 && y >= 0 && (x as f32) < ssize.x && (y as f32) < ssize.y;
    let at = pointer.map(|p| {
        let (x, y) = vp.pixel_at(view, ssize, p);
        Point::new(x, y)
    });
    app.ui.hover_pixel = resp.hover_pos().map(|p| vp.pixel_at(view, ssize, p)).filter(|&px| inside(px));

    // Quick tools while held: Space = hand; Alt = eyedropper (except where Alt is a selection modifier).
    let (space, mods) = ui.input(|i| (i.key_down(egui::Key::Space), i.modifiers));
    let tool = if space {
        Tool::Hand
    } else if mods.alt && !app.ui.tool.selects() && app.canvas.drag.is_none() {
        Tool::Eyedropper
    } else {
        app.ui.tool
    };
    let panning = resp.dragged_by(PointerButton::Middle) || (tool == Tool::Hand && resp.dragged_by(PointerButton::Primary));
    if panning {
        vp.pan += resp.drag_delta();
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if tool == Tool::Hand && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    } else if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }

    // Wheel: zoom around the pointer; Shift scrolls sideways.
    if let Some(p) = resp.hover_pos() {
        for (unit, delta, m) in wheel_events(ui) {
            if m.shift {
                let scale = if unit == MouseWheelUnit::Point { 1.0 } else { 40.0 };
                vp.pan.x += (delta.x + delta.y) * scale;
                continue;
            }
            if m.command || m.alt {
                continue; // brush size and palette index: not yet
            }
            let steps = match unit {
                MouseWheelUnit::Point => {
                    app.canvas.wheel += delta.y;
                    let n = (app.canvas.wheel / POINTS_PER_STEP).trunc();
                    app.canvas.wheel -= n * POINTS_PER_STEP;
                    n as i32
                }
                _ => delta.y.signum() as i32,
            };
            // By default content moving down (wheel up / swipe down) zooms in; the
            // "Invert Wheel Zoom" setting flips it.
            let inward = (steps > 0) != app.ui.prefs.invert_wheel_zoom;
            for _ in 0..steps.unsigned_abs() {
                let z = if inward { zoom::step_in(vp.zoom) } else { zoom::step_out(vp.zoom) };
                vp.zoom_to(z, view, ssize, p);
            }
        }
    }

    // Mouse buttons on the canvas.
    let (pressed_l, pressed_r, released) = ui.input(|i| (i.pointer.primary_pressed(), i.pointer.secondary_pressed(), i.pointer.any_released()));
    let press = (pressed_l || pressed_r) && resp.hovered() && !panning && !space;
    let secondary = pressed_r && !pressed_l;
    if let (true, Some(p)) = (press, at) {
        match tool {
            Tool::Eyedropper => {
                if let (true, Some(img)) = (inside((p.x, p.y)), &app.canvas.image) {
                    let c = img.rgba(p, &sprite.palette);
                    if secondary {
                        app.ui.bg = c;
                    } else {
                        app.ui.fg = c;
                    }
                }
            }
            Tool::Zoom => {
                if let Some(sp) = pointer {
                    let z = if secondary { zoom::step_out(vp.zoom) } else { zoom::step_in(vp.zoom) };
                    vp.zoom_to(z, view, ssize, sp);
                }
            }
            Tool::Bucket => {
                let color = paint_color(app, secondary);
                app.run("draw.fill", &json!({ "x": p.x, "y": p.y, "color": color }));
            }
            Tool::MagicWand => {
                let mode = select_mode(app, secondary, mods);
                app.run("select.shape", &json!({ "shape": "wand", "x0": p.x, "y0": p.y, "mode": mode_name(mode) }));
            }
            Tool::Polygon | Tool::PolygonalLasso => {
                if app.canvas.poly_tool != Some(tool) {
                    app.canvas.poly.clear();
                    app.canvas.poly_tool = Some(tool);
                }
                let closes_loop = app.canvas.poly.first() == Some(&p) && app.canvas.poly.len() > 2;
                if resp.double_clicked() || closes_loop {
                    commit_poly(app, mods);
                } else if app.canvas.poly.last() != Some(&p) {
                    app.canvas.poly.push(p);
                }
            }
            other if !other.works() => {
                app.status = Some(format!("The {} tool is not implemented yet", other.label()));
            }
            Tool::Hand => {}
            other => {
                app.canvas.drag = Some(Drag { tool: other, secondary, mods, start: p, last: p, path: vec![p] });
            }
        }
    }
    // Drag updates: freehand tools follow the pointer pixel by pixel; the spray scatters dots.
    if let (Some(drag), Some(p)) = (app.canvas.drag.as_mut(), at) {
        if drag.tool == Tool::Spray {
            let radius = (i64::from(app.ui.brush_size) * 2 + 2).max(3);
            let span = (2 * radius + 1) as u64;
            for _ in 0..6 {
                let dx = (next_rand(&mut app.canvas.seed) % span) as i64 - radius;
                let dy = (next_rand(&mut app.canvas.seed) % span) as i64 - radius;
                if dx * dx + dy * dy <= radius * radius {
                    drag.path.push(p.offset(dx as i32, dy as i32));
                }
            }
        } else if drag.last != p {
            drag.path.push(p);
        }
        drag.last = p;
    }
    if released && let Some(drag) = app.canvas.drag.take() {
        commit_drag(app, drag);
    }
    // Enter closes a polygon, Escape drops it.
    let (enter, escape) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
    if !app.canvas.poly.is_empty() {
        if enter {
            commit_poly(app, mods);
        } else if escape || !matches!(app.ui.tool, Tool::Polygon | Tool::PolygonalLasso) {
            app.canvas.poly.clear();
            app.canvas.poly_tool = None;
        }
    }
    app.ui.viewports.insert(uid, vp);

    // Marching ants follow the document's selection (after this frame's commands).
    let doc_sel = app.active_doc().and_then(|i| app.session.docs.get(i)).and_then(|d| d.selection.clone());
    let ants = match (&doc_sel, &app.canvas.ants) {
        (Some(m), Some((cached, edges))) if cached == m => Some(edges.clone()),
        (Some(m), _) => {
            let edges = m.edges();
            app.canvas.ants = Some((m.clone(), edges.clone()));
            Some(edges)
        }
        (None, _) => {
            app.canvas.ants = None;
            None
        }
    };
    let animating = ants.is_some() || app.canvas.drag.as_ref().is_some_and(|d| d.tool.selects()) || !app.canvas.poly.is_empty();
    if animating {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    }

    let painter = ui.painter_at(view);
    paint_canvas(&painter, view, vp, ssize, app.canvas.texture.as_ref(), t);
    let overlay =
        Overlay { painter: &painter, view, vp, ssize, image: app.canvas.image.as_ref(), palette: &sprite.palette, t, time: ui.input(|i| i.time) };
    let has_selection = doc_sel.is_some();
    if let Some(edges) = &ants {
        let offset = app.canvas.drag.as_ref().filter(|d| d.tool == Tool::Move).map_or((0, 0), |d| (d.last.x - d.start.x, d.last.y - d.start.y));
        overlay.ants(edges, offset);
    }
    if let Some(drag) = &app.canvas.drag {
        overlay.drag_preview(app, drag, &sprite, has_selection);
    } else if !app.canvas.poly.is_empty() {
        overlay.poly_preview(app, &app.canvas.poly, at, app.canvas.poly_tool == Some(Tool::PolygonalLasso));
    }
    let hovering = resp.hovered() && !panning && tool != Tool::Hand && app.canvas.drag.is_none();
    if let (true, Some(p)) = (hovering, at) {
        overlay.hover(app, tool, p);
    }
}

fn paint_canvas(painter: &egui::Painter, view: Rect, vp: Viewport, ssize: Vec2, texture: Option<&egui::TextureHandle>, t: &Tokens) {
    painter.rect_filled(view, 0.0, t.canvas_bg);
    let r = vp.sprite_rect(view, ssize);
    let visible = r.intersect(view);
    if visible.is_positive() {
        // Checkerboard squares of 16×16 sprite pixels, so they scale with the zoom; doubled while
        // smaller than CHECKER_MIN points. Only the visible ones are drawn.
        let mut s = size::CHECKER_SPRITE_PX * vp.zoom;
        while s < size::CHECKER_MIN {
            s *= 2.0;
        }
        let first = ((visible.min - r.min) / s).floor();
        let last = ((visible.max - r.min) / s).ceil();
        for y in first.y as i64..last.y as i64 {
            for x in first.x as i64..last.x as i64 {
                let c = if (x + y) % 2 == 0 { t.checker_a } else { t.checker_b };
                let sq = Rect::from_min_size(r.min + vec2(x as f32 * s, y as f32 * s), vec2(s, s)).intersect(r);
                painter.rect_filled(sq, 0.0, c);
            }
        }
        if let Some(tex) = texture {
            painter.image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
    }
    painter.rect_stroke(r, 0.0, Stroke::new(1.0, t.canvas_border), egui::StrokeKind::Outside);
}

/// Everything drawn over the sprite: hover, previews, marching ants.
struct Overlay<'a> {
    painter: &'a egui::Painter,
    view: Rect,
    vp: Viewport,
    ssize: Vec2,
    image: Option<&'a Image>,
    palette: &'a pixelcraft_engine::color::Palette,
    t: &'a Tokens,
    time: f64,
}

impl Overlay<'_> {
    fn origin(&self) -> Pos2 {
        self.vp.sprite_rect(self.view, self.ssize).min
    }

    /// The screen point of a pixel corner.
    fn corner(&self, x: i32, y: i32) -> Pos2 {
        self.origin() + vec2(x as f32, y as f32) * self.vp.zoom
    }

    fn pixel_rect(&self, p: Point) -> Rect {
        Rect::from_min_size(self.corner(p.x, p.y), Vec2::splat(self.vp.zoom))
    }

    /// Black over light pixels, white over dark ones ("negative black and white").
    fn negative(&self, p: Point) -> Color32 {
        let inside = p.x >= 0 && p.y >= 0 && (p.x as f32) < self.ssize.x && (p.y as f32) < self.ssize.y;
        let c = match (inside, self.image) {
            (true, Some(img)) => img.rgba(p, self.palette),
            _ => Rgba8::TRANSPARENT,
        };
        let lum = if c.a < 128 {
            // Transparent: the checkerboard (light) or the editor margin (dark) shows through.
            if inside { 180 } else { 60 }
        } else {
            (u32::from(c.r) * 30 + u32::from(c.g) * 59 + u32::from(c.b) * 11) / 100
        };
        if lum > 110 { Color32::BLACK } else { Color32::WHITE }
    }

    /// Fills `pixels` with `color`, skipping the ones off screen.
    fn fill(&self, pixels: &[Point], color: Color32) {
        if pixels.len() > MAX_PREVIEW_PIXELS {
            return;
        }
        for &p in pixels {
            let r = self.pixel_rect(p);
            if r.intersects(self.view) {
                self.painter.rect_filled(r, 0.0, color);
            }
        }
    }

    /// The outline of a pixel set, each edge in the negative colour of the pixel inside it.
    fn outline(&self, pixels: &[Point]) {
        if pixels.len() > MAX_PREVIEW_PIXELS {
            return;
        }
        let set: HashSet<Point> = pixels.iter().copied().collect();
        for &p in pixels {
            let s = Stroke::new(1.0, self.negative(p));
            let (x, y) = (p.x, p.y);
            if !set.contains(&Point::new(x, y - 1)) {
                self.painter.line_segment([self.corner(x, y), self.corner(x + 1, y)], s);
            }
            if !set.contains(&Point::new(x, y + 1)) {
                self.painter.line_segment([self.corner(x, y + 1), self.corner(x + 1, y + 1)], s);
            }
            if !set.contains(&Point::new(x - 1, y)) {
                self.painter.line_segment([self.corner(x, y), self.corner(x, y + 1)], s);
            }
            if !set.contains(&Point::new(x + 1, y)) {
                self.painter.line_segment([self.corner(x + 1, y), self.corner(x + 1, y + 1)], s);
            }
        }
    }

    /// Hover feedback for the tool at pixel `p`.
    fn hover(&self, app: &PixelApp, tool: Tool, p: Point) {
        let shapes = matches!(
            tool,
            Tool::Rectangle | Tool::FilledRectangle | Tool::Ellipse | Tool::FilledEllipse | Tool::Contour | Tool::Polygon | Tool::Bucket
        );
        if tool.paints_with_brush() || shapes {
            let size = if tool.paints_with_brush() { app.ui.brush_size } else { 1 };
            let pixels = paint::stamp(&[p], size);
            if tool != Tool::Eraser {
                self.fill(&pixels, c32(app.ui.fg));
            }
            self.outline(&pixels);
        }
        // The crosshair itself is the OS cursor (pixel or native), set by the canvas.
    }

    /// Live feedback while dragging.
    fn drag_preview(&self, app: &PixelApp, d: &Drag, sprite: &Sprite, has_selection: bool) {
        let color = c32(if d.secondary { app.ui.bg } else { app.ui.fg });
        let (a, b) = (d.start, d.last);
        let clip = geom::Rect::from_size(sprite.size);
        match d.tool {
            Tool::Pencil => self.fill(&paint::stroke(&d.path, app.ui.brush_size, app.ui.pixel_perfect), color),
            Tool::Spray => self.fill(&d.path, color),
            Tool::Eraser => {
                let px = paint::stroke(&d.path, app.ui.brush_size, false);
                self.fill(&px, self.t.checker_a);
                self.outline(&px);
            }
            Tool::Line => self.fill(&paint::stroke(&[a, b], app.ui.brush_size, false), color),
            Tool::Rectangle => self.fill(&paint::rectangle(a, b, false), color),
            Tool::FilledRectangle => self.fill(&paint::rectangle(a, b, true), color),
            Tool::Ellipse => self.fill(&paint::ellipse(a, b, false), color),
            Tool::FilledEllipse => self.fill(&paint::ellipse(a, b, true), color),
            Tool::Contour => self.fill(&paint::polygon(&d.path, true, clip), color),
            Tool::RectangularMarquee => {
                let r = paint::corners(a, b);
                let (x1, y1) = (r.x.saturating_add(r.w as i32), r.y.saturating_add(r.h as i32));
                self.dashed_loop(&[self.corner(r.x, r.y), self.corner(x1, r.y), self.corner(x1, y1), self.corner(r.x, y1)]);
            }
            Tool::EllipticalMarquee => {
                let r = paint::corners(a, b);
                let c = Rect::from_min_max(self.corner(r.x, r.y), self.corner(r.x.saturating_add(r.w as i32), r.y.saturating_add(r.h as i32)));
                let pts: Vec<Pos2> = (0..48)
                    .map(|i| {
                        let t = i as f32 / 48.0 * std::f32::consts::TAU;
                        c.center() + vec2(t.cos() * c.width() / 2.0, t.sin() * c.height() / 2.0)
                    })
                    .collect();
                self.dashed_loop(&pts);
            }
            Tool::Lasso => {
                let pts: Vec<Pos2> = d.path.iter().map(|p| self.pixel_rect(*p).center()).collect();
                self.dashed_loop(&pts);
            }
            Tool::Move if !has_selection => {
                // Without a selection the whole cel moves: show its outline at the new place.
                let r = Rect::from_min_size(self.corner(b.x - a.x, b.y - a.y), self.ssize * self.vp.zoom);
                self.dashed_loop(&[r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()]);
            }
            _ => {}
        }
    }

    /// The polygon being clicked in, closed back to the pointer.
    fn poly_preview(&self, app: &PixelApp, poly: &[Point], at: Option<Point>, lasso: bool) {
        let mut pts: Vec<Point> = poly.to_vec();
        if let Some(p) = at {
            pts.push(p);
        }
        if lasso {
            let screen: Vec<Pos2> = pts.iter().map(|p| self.pixel_rect(*p).center()).collect();
            self.dashed_loop(&screen);
        } else {
            self.fill(&paint::polygon(&pts, false, geom::Rect::new(0, 0, 0, 0)), c32(app.ui.fg));
        }
        for p in poly {
            self.painter.rect_stroke(self.pixel_rect(*p).expand(1.0), 0.0, Stroke::new(1.0, self.negative(*p)), egui::StrokeKind::Outside);
        }
    }

    /// A closed dashed outline, black on white, marching with time.
    fn dashed_loop(&self, pts: &[Pos2]) {
        if pts.len() < 2 {
            return;
        }
        let mut closed = pts.to_vec();
        if let Some(&first) = pts.first() {
            closed.push(first);
        }
        self.painter.add(egui::Shape::line(closed.clone(), Stroke::new(1.0, Color32::WHITE)));
        let mut travelled = ((self.time * 8.0) % 8.0) as f32;
        let mut dashes = Vec::new();
        for w in closed.windows(2) {
            let (p, q) = (w[0], w[1]);
            let len = p.distance(q);
            let dir = if len > 0.0 { (q - p) / len } else { Vec2::ZERO };
            let mut s = 0.0;
            while s < len {
                let step = (4.0 - travelled % 4.0).min(len - s).max(0.01);
                if (travelled % 8.0) < 4.0 {
                    dashes.push(egui::Shape::line_segment([p + dir * s, p + dir * (s + step)], Stroke::new(1.0, Color32::BLACK)));
                }
                s += step;
                travelled += step;
            }
        }
        self.painter.extend(dashes);
    }

    /// Marching ants along the selection's pixel edges (optionally shown moved by `offset`).
    fn ants(&self, edges: &[Edge], offset: (i32, i32)) {
        let phase = ((self.time * 8.0) as i64).rem_euclid(8);
        let dash = 4.0_f32;
        let mut shapes = Vec::new();
        for e in edges {
            let a = self.corner(e.from.x.saturating_add(offset.0), e.from.y.saturating_add(offset.1));
            let b = self.corner(e.to.x.saturating_add(offset.0), e.to.y.saturating_add(offset.1));
            if !Rect::from_two_pos(a, b).expand(1.0).intersects(self.view) {
                continue;
            }
            let len = a.distance(b);
            let dir = if len > 0.0 { (b - a) / len } else { Vec2::ZERO };
            // The dash index comes from the absolute screen position, so the pattern runs on
            // across edges and marches as `phase` advances.
            let base = (a.x + a.y) as i64;
            let mut s = 0.0;
            while s < len {
                let step = dash.min(len - s);
                let k = ((base + s as i64 + phase) / dash as i64).rem_euclid(2);
                let c = if k == 0 { Color32::BLACK } else { Color32::WHITE };
                shapes.push(egui::Shape::line_segment([a + dir * s, a + dir * (s + step)], Stroke::new(1.0, c)));
                s += step;
            }
        }
        self.painter.extend(shapes);
    }
}
