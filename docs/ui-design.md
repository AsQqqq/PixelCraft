# UI design

> **Last updated:** 2026-10-19

![PixelCraft's window: menu bar, tabs, context bar, colour bar with palette and picker, a 32×32 sprite at 1200 %, tool bar, timeline with two layers and three frames, status bar](images/interface.png)

## Layout

The target app's arrangement and proportions (measured: `plan/aseprite/observed-ui.md`), with our
own look. One module per panel in `crates/ui-egui/src/panels/`; sizes in `theme::size`.

```text
┌──────────────────────────────────────────────────────────────┐
│ menu bar: File Edit Sprite Layer Frame Select View Help      │
├──────────────────────────────────────────────────────────────┤
│ tabs: one per sprite, × closes, + new                 24 pt  │
├──────────────────────────────────────────────────────────────┤
│ context bar: options of the current tool              34 pt  │
├────────┬─────────────────────────────────────────────┬───────┤
│ colour │                                             │ tool  │
│ bar    │      sprite editor (checkerboard, sprite)   │ bar   │
│ 144 pt │                                             │ 36 pt │
│        ├─────────────────────────────────────────────┤       │
│palette │ timeline: transport · layers × frames 140pt │       │
│picker  │                                             │       │
├────────┴─────────────────────────────────────────────┴───────┤
│ status bar: pointer pixel, size · frame, zoom          24 pt │
└──────────────────────────────────────────────────────────────┘
```

| Panel | Module | Contents |
|---|---|---|
| Menu bar | `menu_bar.rs` | engine commands by `menu` path, plus the UI's view actions (`UiAction::MENU`) |
| Tabs | `tabs.rs` | open sprites (• = unsaved changes), × closes (`file.close`), + new (`file.new`) |
| Context bar | `context_bar.rs` | per tool: pencil/eraser size, ink, pixel-perfect; zoom/hand: 100 %, Center, Fit Screen |
| Colour bar | `color_bar.rs`, `color_picker.rs` | palette in 5 columns (click = foreground, right click = background, corner mark on the foreground), fg/bg swatches (click or X swaps), saturation/value square, hue and alpha strips, palette index and hex |
| Tool bar | `tool_bar.rs`, `icons.rs` | one icon button per tool, tooltips with the key |
| Sprite editor | `canvas.rs` | checkerboard, sprite texture (nearest above 100 %, linear below), 1 pt border |
| Timeline | `timeline.rs` | first/previous/play/next/last; frame numbers; per layer eye, lock, name and one cell per frame (dot = cel); active row, frame and cel highlighted |
| Status bar | `status_bar.rs` | `+ x y` under the pointer, `□ w h`, `Frame: n/total`, zoom `100.0%`; errors replace the left side |

## Lifecycle

As observed in the target app (`plan/aseprite/lifecycle.md`), tested in `crates/ui-egui/src/tests.rs`:

| Step | PixelCraft |
|---|---|
| Launch | Home tab only: New File..., Open File..., Recover Files..., Recent files and folders (no News pane, by choice); status bar shows the version; title `PixelCraft v0.1.0` |
| File ▸ New... (Cmd+N, Home link) | **New Sprite** dialog with the last used values: width/height (width focused and selected), colour mode, background, Advanced Options; Enter = OK, Escape = Cancel |
| OK | tab `Sprite-0001` (numbers count up for the session, never reused), 100 % zoom centred, title `Sprite-0001 - PixelCraft v0.1.0`; white/black background makes an opaque `Background` layer |
| Edit the sprite | the tab's × becomes ● (unsaved changes); the title doesn't change |
| Undo back to the saved point | ● goes back to ×; closing no longer asks |
| Close (Cmd+W, tab ×/●) untouched | closes at once; with no sprite left, back to Home |
| Close modified | **Warning**: "Saving changes to the sprite "…" before closing?" Save / Don't Save / Cancel |
| Quit (File ▸ Exit, Cmd+Q, window close) | one Warning per modified sprite ("before quitting?"); Cancel stops the quit; Don't Save on the last one quits |
| Close Home | empty workspace with a small Home button; View ▸ Home reopens it |

Save stays disabled in the Warning until `.aseprite` writing lands (`docs/gaps.md`).

## Zoom and navigation

`crates/ui-egui/src/zoom.rs`: 25 levels, `1/n` and `n` for n ∈ {1, 2, 3, 4, 5, 6, 8, 12, 16, 24,
32, 48, 64} (1.6 % to 6400 %), as measured in the target app.

| Input | Action |
|---|---|
| Wheel | one level per notch around the pointer (wheel up / swipe down = in; View ▸ Invert Wheel Zoom flips it) |
| Shift + wheel | scroll sideways |
| Cmd/Ctrl + `=` / `-` | zoom in / out (around the pointer) |
| `` ` `` `1` `2` `3` `4` `5` `6` | 50 %, 100 %, 200 %, 400 %, 800 %, 1600 %, 3200 % |
| Cmd/Ctrl + `0` | fit on screen |
| Shift + `Z` | scroll to the centre |
| Space (hold), Hand tool `H`, middle button | pan |
| Zoom tool `Z` | click in, right click out |

A new sprite opens at 100 %, centred, like the target app (Cmd+0 fits it). Zoom and pan are kept
per document. The checkerboard squares are 16×16 sprite pixels, doubled while under 8 points.

## Keys

`crates/ui-egui/src/keys.rs` matches the **physical** key with exact modifiers, so shortcuts work
under any keyboard layout. Engine commands bind through their `shortcut` field; UI-only keys:

| Key | Action |
|---|---|
| B E I Z H V G L U, Shift+U | Pencil, Eraser, Eyedropper, Zoom, Hand, Move, Bucket, Line, Rectangle, Ellipse |
| Alt (hold) | eyedropper |
| X | swap foreground and background |
| Tab | show/hide the timeline |
| Enter | play/stop |
| Left / Right, Home / End | previous/next, first/last frame (engine) |
| Up / Down | next/previous layer (engine) |

## Tokens

Every colour comes from `theme::Tokens` (`crates/ui-egui/src/theme.rs`): an original dark studio
palette with a warm orange accent. No skin, colour or icon is taken from the target app; the
icons are drawn in code (`icons.rs`).

## Looking at the UI

```sh
cargo run -p pixelcraft-ui-egui --example snapshot -- --scale 1 \
  --cmd 'pixel.stroke={"x0":2,"y0":2,"x1":29,"y1":29,"color":"#d24b3c"}' --tool hand --zoom 4 out.png
```

Renders the whole window offscreen (no window, no focus stealing). kittest wraps the UI in an
8 pt margin that the real window doesn't have.
