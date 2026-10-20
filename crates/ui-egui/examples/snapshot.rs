//! Renders the whole PixelCraft window offscreen (wgpu, no window) to a PNG.
//!
//! `cargo run -p pixelcraft-ui-egui --example snapshot -- [--size 1280x800] [--scale 2] [--home]
//!  [--cmd 'id={json}']... [--action new|close|quit|closehome]... [--tool zoom] [--zoom 8]
//!  [--pointer x,y] out.png`
//!
//! `--home` starts like the app (Home tab, no sprite); `--action` replays lifecycle steps.
//!
//! Commands run on a new 32×32 sprite before the render, e.g.
//! `--cmd 'pixel.stroke={"x0":2,"y0":2,"x1":29,"y1":29,"color":"#d24b3c"}'`.

use pixelcraft_ui_egui::{PixelApp, Tool, UiAction};
use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut size = (1280.0f32, 800.0f32);
    let mut scale = 2.0f32;
    let mut cmds: Vec<String> = vec![];
    let mut tool = None;
    let mut zoom_digit = None;
    let mut out = "snapshot.png".to_string();
    let mut home = false;
    let mut actions: Vec<String> = vec![];
    let mut pointer: Option<(f32, f32)> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--size" => {
                if let Some((w, h)) = it.next().and_then(|s| s.split_once('x')) {
                    size = (w.parse().unwrap_or(1280.0), h.parse().unwrap_or(800.0));
                }
            }
            "--scale" => scale = it.next().and_then(|s| s.parse().ok()).unwrap_or(2.0),
            "--cmd" => cmds.extend(it.next().cloned()),
            "--tool" => {
                tool = it.next().and_then(|s| Tool::ALL.into_iter().find(|t| t.label().replace(' ', "").eq_ignore_ascii_case(&s.replace(' ', ""))))
            }
            "--home" => home = true,
            "--pointer" => pointer = it.next().and_then(|s| s.split_once(',')).and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?))),
            "--action" => actions.extend(it.next().cloned()),
            "--zoom" => zoom_digit = it.next().and_then(|s| s.parse::<u8>().ok()),
            other => out = other.to_string(),
        }
    }

    let mut app = if home { PixelApp::default() } else { PixelApp::with_new_sprite() };
    for c in &cmds {
        let (id, p) = c.split_once('=').map(|(a, b)| (a, serde_json::from_str(b).unwrap_or(json!({})))).unwrap_or((c.as_str(), json!({})));
        app.run(id, &p);
    }
    for a in &actions {
        match a.as_str() {
            "new" => app.act(UiAction::NewFile),
            "close" => app.act(UiAction::CloseTab),
            "quit" => app.act(UiAction::Quit),
            "closehome" => {
                app.act(UiAction::Home);
                app.act(UiAction::CloseTab);
            }
            other => eprintln!("unknown action `{other}`"),
        }
    }
    if let Some(t) = tool {
        app.act(UiAction::SetTool(t));
    }

    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(size.0, size.1))
        .with_pixels_per_point(scale)
        .with_max_steps(12)
        // kittest's default "predictable" filtering is always bilinear; the app samples in
        // hardware, so do the same here or zoomed-in pixels come out blurred.
        .with_render_options(egui_wgpu::RendererOptions { predictable_texture_filtering: false, ..egui_wgpu::RendererOptions::PREDICTABLE })
        .wgpu()
        .build_ui_state(|ui, app: &mut PixelApp| app.ui(ui), app);
    harness.run_steps(4);
    if let Some(d) = zoom_digit {
        harness.state_mut().act(UiAction::ZoomPreset(d));
        harness.run_steps(2);
    }
    if let Some((x, y)) = pointer {
        harness.input_mut().events.push(egui::Event::PointerMoved(egui::pos2(x, y)));
        harness.run_steps(2);
    }
    let img = harness.render().expect("render");
    img.save(&out).expect("save png");
    eprintln!("wrote {out} ({}x{})", img.width(), img.height());
}
