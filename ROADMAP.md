# PixelCraft roadmap

**Stage: pre-alpha** · next: alpha (the Aseprite-style interface and `.aseprite` round-trip)

> **Last reviewed:** 2026-10-19 · **Last updated:** 2026-10-20 · **Change:** minor (0.1.4-beta: copy, cut and paste with a transform box) · **Target:** Aseprite

PixelCraft aims at full Aseprite parity, and at being better in places: open (MIT/Apache),
fully scriptable by agents (MCP, control channel, CLI), and on the web. This page is the
summary. The detail is in:

- [`docs/gaps.md`](docs/gaps.md): every known shortfall, ranked; the work list
- [`docs/roadmap.md`](docs/roadmap.md): milestones, current focus, what's next
- [`docs/architecture.md`](docs/architecture.md): how it is built
- [`docs/ui-design.md`](docs/ui-design.md): layout, tokens and the look
- [`docs/file-format-parity.md`](docs/file-format-parity.md): `.aseprite`, PNG, GIF, sprite sheets
- [`docs/releasing.md`](docs/releasing.md): versions, packaging, the release workflow

## Headline numbers

| Number | Value | Kind |
|---|---|---|
| Commands | 31 | measured: `pixelcraft-cli commands` |
| Tests | 118 | measured: `cargo test --workspace` |
| `.aseprite` support | header only | measured: `crates/ase` |

## What's next

1. **M1 — Interface** (in progress): layout, zoom ladder and shortcuts are in
   ([`docs/ui-design.md`](docs/ui-design.md)); next, one undo step per stroke.
2. **M2 — Tools.** Pencil (one undo step per stroke), eraser, eyedropper, bucket, line,
   rectangle, ellipse, selection, move; brush size and pixel-perfect.
3. **M3 — Files.** `.aseprite` read and write (layers, cels, frames, tags, palette), PNG and GIF export.
4. **M4 — Animation.** Timeline editing, onion skin, playback, tags and directions.

## Progress log

- **2026-10-20 (0.1.4-beta):** Edit ▸ Copy / Cut / Paste (Cmd/Ctrl+C, X, V; `edit.copy`, `edit.cut`, `edit.paste`): the paste floats in a transform box — move, stretch by eight handles (Shift: proportions), rotate outside the corners (Shift: 15°) — and applies with Enter or a click to the current layer and frame; nearest-neighbour scaling and rotation in `raster::transform`; works in every colour mode; Cmd+V works whatever the system clipboard holds.
- **2026-10-20 (0.1.3-beta):** Redo also on Cmd/Ctrl+Shift+Z and Cmd/Ctrl+R (as well as Cmd/Ctrl+Y).
- **2026-10-20 (0.1.3-beta):** clicking a frame number in the timeline selects the frame; Delete/Backspace then delete the frame itself (before, they only cleared its pixels and left an empty frame in the animation).
- **2026-10-20 (0.1.3-beta):** opening images: File ▸ Open... (Cmd+O, Home link; the OS dialog, the browser's picker on the web) and drag-and-drop open PNG, JPEG, GIF (animated: every frame), BMP, WebP, TGA, ICO, TIFF, PNM and QOI as new tabs (`file.open`); big images open fitted and draw in tiles past the GPU texture limit (fixes a crash risk).
- **2026-10-20 (0.1.3-beta):** Frame ▸ Delete Frame (`frame.remove`, Alt+C, timeline right-click menu): removes the frame and its cels, shrinks tags, one undo step; the only frame stays.
- **2026-10-20 (0.1.3-beta):** Edit ▸ Delete (`edit.clear`, Delete or Backspace) clears the selected pixels, in every colour mode; the whole cel without a selection; the background colour on a background layer.
- **2026-10-20 (0.1.2-beta):** releases: `cargo xtask version [set]`, `--version` on both binaries, packaging scripts for macOS (universal DMG), Windows (x64/ARM64 portable zip), Linux (AppImage, deb, rpm, tar.gz) and the web (static zip), and a release workflow that builds a draft "PixelCraft v<version>" with SHA256SUMS ([`docs/releasing.md`](docs/releasing.md)).
- **2026-10-20 (0.1.1):** the app icon (the owner's lizard): master SVG, macOS/Windows/Linux/web renders from `packaging/icons.sh` (pure-Rust renderer in xtask), set on the window, the Windows executable, the web favicon and the Home tab.
- **2026-10-20 (0.1.1):** drawing on a new layer works (cels are created on first paint); pixel cursors are real OS cursors (no lag, no leftovers, every window and dialog); Edit ▸ Preferences with native/pixel cursor, cursor scale and wheel direction.
- **2026-10-20:** fixes: the pixel cursor only takes over in a focused window (no more two cursors); double-click on the tab strip's free space for New Sprite; Preview window plays the animation on its own (▶/■, Shift+Enter), zooms with the wheel, pans, fits.
- **2026-10-20:** pixel-art look: PixelCraft's own pixel font built into a TrueType font in code, hand-drawn pixel icons, bevelled controls, no anti-aliasing, a pixel cursor drawn over the UI, the "Dusk" palette.
- **2026-10-20:** the target app's tool bar: twelve groups with pop-out strips and tooltips, 20 of 26 tools working (marquees, lasso, magic wand, pencil, spray, eraser, eyedropper, zoom, hand, move, bucket, line, rectangles, ellipses, contour, polygon), brush size and pixel-perfect, one undo step per stroke or shape, hover brush preview with negative outline and crosshair, marching-ants selections, Preview window, double-click on an empty workspace for New Sprite.
- **2026-10-19:** application lifecycle as observed in the target app: Home tab at launch, New Sprite dialog (remembered values, background), session-wide sprite numbering, ● for unsaved changes (clean again after undo), Warning on close and quit, window title; wheel zoom direction setting.
- **2026-10-19:** first interface: measured layout, tabs, colour picker, tool icons, timeline, 25-level zoom around the pointer, panning, layout-independent shortcuts, playback.
- **2026-10-19:** workspace, layering, engine command model, first window, CLI and MCP tool list.
