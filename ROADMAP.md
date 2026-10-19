# PixelCraft roadmap

**Stage: pre-alpha** · next: alpha (the Aseprite-style interface and `.aseprite` round-trip)

> **Last reviewed:** 2026-10-19 · **Last updated:** 2026-10-19 · **Change:** major (application lifecycle: Home, New Sprite, unsaved-changes warnings) · **Target:** Aseprite

PixelCraft aims at full Aseprite parity, and at being better in places: open (MIT/Apache),
fully scriptable by agents (MCP, control channel, CLI), and on the web. This page is the
summary. The detail is in:

- [`docs/gaps.md`](docs/gaps.md): every known shortfall, ranked; the work list
- [`docs/roadmap.md`](docs/roadmap.md): milestones, current focus, what's next
- [`docs/architecture.md`](docs/architecture.md): how it is built
- [`docs/ui-design.md`](docs/ui-design.md): layout, tokens and the look
- [`docs/file-format-parity.md`](docs/file-format-parity.md): `.aseprite`, PNG, GIF, sprite sheets

## Headline numbers

| Number | Value | Kind |
|---|---|---|
| Commands | 16 | measured: `pixelcraft-cli commands` |
| Tests | 42 | measured: `cargo test --workspace` |
| `.aseprite` support | header only | measured: `crates/ase` |

## What's next

1. **M1 — Interface** (in progress): layout, zoom ladder and shortcuts are in
   ([`docs/ui-design.md`](docs/ui-design.md)); next, one undo step per stroke.
2. **M2 — Tools.** Pencil (one undo step per stroke), eraser, eyedropper, bucket, line,
   rectangle, ellipse, selection, move; brush size and pixel-perfect.
3. **M3 — Files.** `.aseprite` read and write (layers, cels, frames, tags, palette), PNG and GIF export.
4. **M4 — Animation.** Timeline editing, onion skin, playback, tags and directions.

## Progress log

- **2026-10-19:** application lifecycle as observed in the target app: Home tab at launch, New Sprite dialog (remembered values, background), session-wide sprite numbering, ● for unsaved changes (clean again after undo), Warning on close and quit, window title; wheel zoom direction setting.
- **2026-10-19:** first interface: measured layout, tabs, colour picker, tool icons, timeline, 25-level zoom around the pointer, panning, layout-independent shortcuts, playback.
- **2026-10-19:** workspace, layering, engine command model, first window, CLI and MCP tool list.
