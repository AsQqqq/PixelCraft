# Gaps: the ranked work list

> **Last updated:** 2026-10-20

| Rank | Gap | Area | Notes |
|---|---|---|---|
| 1 | Pixel look, second pass: bevels on every egui widget (menus, text fields, sliders, checkboxes are flat-outlined), pixel scroll bars, Cyrillic and more symbols in the pixel font, a light theme | UI | `theme.rs`, `pixel_font/glyphs.rs` |
| 2 | No Save, and no `.aseprite` open: images (PNG, JPEG, GIF…) open as new sprites, but `.aseprite` reads the header only and can't be written; Save is disabled in the unsaved-changes Warning; Recent files/folders stay empty; Recover Files is disabled; no import of an image into an open sprite | files | `crates/ase`, `crates/io`, File menu, Home |
| 3 | Tools not implemented: Slice, Gradient, Curve, Blur, Jumble, Text; eraser's right button (replace foreground with background); Shift/Ctrl constraints on shapes and lines; magic wand tolerance in the UI | tools | tool bar shows them; clicking says so |
| 4 | Ink types, opacity, brush shapes other than round, dynamics | tools | context bar |
| 5 | Timeline: no drag, multi-select, rename, lock toggle, tags, onion skin | UI | `panels/timeline.rs` |
| 6 | Scroll bars, pixel grid, tiled mode, symmetry | UI | |
| 7 | Wheel modifiers: Ctrl = brush size, Ctrl+Shift = frame, Alt = palette index | UI | `panels/canvas.rs` |
| 8 | No PNG/GIF export | files | |
| 9 | No control channel or MCP transport | agents | |
| 10 | Compositor ignores blend modes other than Normal | compose | |
| 11 | Selection changes are not undoable (the target app undoes them) | engine | |
| 13 | Preferences and UI state are not saved between runs | UI | needs a settings file |
| 14 | Web build: no custom pixel cursors (eframe's web runner ignores cursor images) | UI | |
| 12 | Cmd+Q from the macOS app menu: not verified that it reaches the unsaved-changes flow (File ▸ Exit and the window's close button do) | UI | |
