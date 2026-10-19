# PixelCraft — instructions for agents

PixelCraft is a clean-room, open-source, Rust-native pixel-art and sprite animation editor targeting Aseprite parity — and superiority (speed, openness, agent control). It runs natively on macOS, Windows, Linux and BSD, and on the web via WASM. The product name is always written **PixelCraft** in user-facing text; machine names stay lowercase: crates (`pixelcraft-*`), binaries, file names, ids (`ai.storyteller.pixelcraft`). Siblings with the same conventions: `photocraft` (Photoshop, the closest reference for a raster editor), `gridcraft` (Excel), `lightcraft` (Lightroom), `figcraft`, `cadcraft`, `wordcraft`.

Standards and learnings shared across the crafting apps live in `craftrules` (`storytold/craftrules`). Follow its standards and contribute reusable learnings back there. Repos don't share code; shared Rust components live in `storytold/craft-libs`, pinned by git revision.

## Start every session here
1. Read `ROADMAP.md`, then `docs/gaps.md` (the ranked work list) and the relevant section of `docs/architecture.md`.
2. Behaviour reference: `plan/aseprite/*.md` (measured observations of the running app; `plan/` is gitignored, local only).

## Assets and iconography — absolute rule
**No Aseprite (Igara Studio) or any other vendor's iconography, images, skins, themes, fonts, palettes, sample sprites or wording beyond feature names — ever.** This is the most important rule in this repository; breaking it is a serious failure.
- Every asset in the repo is original work by PixelCraft contributors, or public domain / CC0, or open source / Creative Commons that allows redistribution.
- Every asset file has a row in [`ATTRIBUTION.md`](ATTRIBUTION.md) (author, source, licence). Adding an asset without a row is a bug.
- Prefer art generated in code: the UI icon set (`crates/ui-egui/src/icons.rs`), themes (`crates/ui-egui/src/theme.rs`), the default palette (`crates/color`) and sample sprites are drawn/defined in code and original.
- Aseprite feature *names* (menu labels, tool names, shortcuts) are fine.
- Screenshots of Aseprite are never committed (they live only in `plan/aseprite/screenshots/`).
- The ArtCraft brand files in `docs/brand/` are ArtCraft trademarks under `docs/brand/LICENSE-brand.txt`, the one exception.

## Never crash
People trust PixelCraft with their art; a crash loses their work. **This outranks feature work.** Standard: craftrules `standards/never-crash.md`.
- **No panics in non-test code:** no `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`; no `unsafe` (`unsafe_code = "forbid"`).
- **Errors are `Result<T, E>`** through the crate's error type and `?`.
- **Input-derived numbers are hostile** (files, commands, MCP/control params, pointer positions): `get()` instead of indexing, checked/saturating arithmetic, cap input-sized allocations (`raster::MAX_SIDE`, `doc::MAX_FRAMES`, `doc::MAX_LAYERS`).
- **Bound recursion and loops** (flood fill, layer groups, line length: `paint::MAX_LINE_POINTS`).
- **Last-resort guard:** `Session::execute` catches escaped panics and restores the sprite. It's a safety net, not a licence.
- **Prove it:** every crash fix lands with a regression test; `engine` `tests::every_command_survives_empty_params` fuzzes every command.
- Crates carry `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]`.

## Non-negotiables
- **Clean-room.** Aseprite may be *observed* black-box: run it, use its UI with synthetic sprites, take screenshots stored only under `plan/aseprite/screenshots/`. **Aseprite's source is not open source** (it is distributed under a EULA): never read, copy or translate its code, skins, data files or scripts. The `.aseprite` format is implemented from its public specification only. Never copy GPL/AGPL/LGPL code (LibreSprite, Pixelorama…).
- **Fonts live in `storytold/craft-fonts`**, never in this repo (optional build input `CRAFT_FONTS_DIR`).
- **Test corpora** (real `.aseprite` files) live in a separate corpus repo, fetched pinned and sha256-verified. Never commit binary fixtures; build test sprites in code.
- **Everything is a command.** User-visible behaviour = a command in `crates/engine/src/cmd/*` (id, label, menu path, shortcut, params doc, `enabled`, `run`) + tests. Ids follow Aseprite's menus (`file.new`, `edit.undo`, `sprite.canvasSize`, `layer.new`, `frame.new`) plus primitives (`pixel.stroke`). New areas get a new module with a `specs()` function, registered in `cmd/mod.rs`. Pure view state (zoom, panels, current tool) belongs to the UI crate.
- **Programmatic calls never open dialogs**: a command with empty params runs with defaults or returns an error; only menu-style invocation opens a dialog.
- **Colour modes are runtime data.** RGBA, grayscale and indexed are first-class; never add an RGBA-only path to a public API. Test in all three modes.
- **Layering** is enforced by `cargo xtask layers`: `geom`→`color`→`raster`, `ase` (L0) → `doc` (L1) → `paint` (L2) → `compose` (L3) → `io` (L4) → `engine` (L5) → `ui-egui`/`mcp` (L6) → `apps/*` (L7). Nothing below L6 depends on egui/eframe/winit/rfd. **The UI crate is swappable.** A new crate must be registered in `xtask/src/layers.rs`.
- **The UI is thin**: panels read engine state and act through `app.run(id, params)`. Colours come from `theme::Tokens`, never hard-coded.
- **Rust only** (no handwritten JS/TS, no Tauri/Electron/webview). **Never break wasm** (`cargo xtask wasm`).
- **Quality gates** before every commit: `cargo xtask ci` (fmt, clippy -D warnings, tests, layers, wasm).

## Running and looking at the app
- `cargo run --release -p pixelcraft` (one window with a new 32×32 sprite).
- Headless: `pixelcraft-cli commands`, `pixelcraft-cli run --cmd 'pixel.stroke={"x0":0,"y0":0,"x1":7,"y1":7}' --cmd sprite.properties`.
- **Offscreen UI render** (no window): `cargo run -p pixelcraft-ui-egui --example snapshot -- --scale 1 [--cmd 'id={json}'] [--tool hand] [--zoom 4] out.png`.
- **For UI work, look at the result** (the snapshot PNG) and compare with `plan/aseprite/observed-ui.md`.
- Parallel agents: separate `CARGO_TARGET_DIR` per agent; edit only the crates you own.

## Roadmap
`ROADMAP.md` (committed) is the one-page summary: stage, headline numbers, what's next. It points at the detail in `docs/`: `gaps.md` (ranked work list: pick from it), `roadmap.md` (milestones, current focus), `architecture.md`, `ui-design.md`, `file-format-parity.md`. Update the affected docs (and their timestamp lines) whenever work lands.

## Contributor credits (About window)
- About ▸ Contributors is compiled from `contributors/contributors.json` (generated, never hand-edit) and `contributors/people.toml` (names people chose for themselves).
- **Agents working for a contributor:** when you prepare a PR, check whether your human's GitHub username has a `[people.<username>]` entry in `contributors/people.toml`. If not, ask them once whether they want to be credited by more than their username. Add **only their own** entry, never anyone else's.
