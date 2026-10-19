# Gaps: the ranked work list

> **Last updated:** 2026-10-19

| Rank | Gap | Area | Notes |
|---|---|---|---|
| 1 | Each pencil segment is its own undo step | engine | needs a stroke transaction (begin/commit) so one drag = one undo |
| 2 | No Save / Open: `.aseprite` reads the header only and can't be written; Save is disabled in the unsaved-changes Warning; Recent files/folders stay empty; Recover Files is disabled | files | `crates/ase`, `crates/io`, File menu, Home |
| 3 | Tools not implemented: Move, Bucket, Line, Rectangle, Ellipse, selection | tools | tool bar shows them; clicking says so |
| 4 | Brush size, ink types, pixel-perfect have no effect | tools | context bar shows them disabled or inert |
| 5 | Timeline: no drag, multi-select, rename, lock toggle, tags, onion skin | UI | `panels/timeline.rs` |
| 6 | Preview window, scroll bars, pixel grid, tiled mode | UI | |
| 7 | Wheel modifiers: Ctrl = brush size, Ctrl+Shift = frame, Alt = palette index | UI | `panels/canvas.rs` |
| 8 | No PNG/GIF export | files | |
| 9 | No control channel or MCP transport | agents | |
| 10 | Compositor ignores blend modes other than Normal | compose | |
| 11 | No floating Preview window (the target app opens one with every sprite) | UI | |
| 12 | Cmd+Q from the macOS app menu: not verified that it reaches the unsaved-changes flow (File ▸ Exit and the window's close button do) | UI | |
