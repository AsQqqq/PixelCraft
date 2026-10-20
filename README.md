<p align="center">
  <a href="https://getartcraft.com/">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="docs/brand/artcraft-logo-white.svg">
      <img alt="ArtCraft" src="docs/brand/artcraft-logo.svg" width="200">
    </picture>
  </a>
</p>

<p align="center">
  <img alt="" src="assets/app-icon/hicolor/128x128/apps/ai.storyteller.pixelcraft.png" width="96" height="96">
</p>

<h1 align="center">PixelCraft</h1>

<p align="center">
  <b>Pixel art and sprite animation; an open-source, clean-room take on the Aseprite workflow, rebuilt in pure Rust.</b>
</p>

<p align="center">
  A fast, open-source pixel-art editor for sprites, tiles and frame-by-frame animation. It runs
  natively on macOS, Windows, Linux and BSD, and in the browser via WebAssembly.<br>
  <i>By the ArtCraft team.</i>
</p>

<p align="center">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-147a40?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on macOS, Windows, Linux, BSD and the web" src="https://img.shields.io/badge/runs%20on-macOS%20%C2%B7%20Windows%20%C2%B7%20Linux%20%C2%B7%20BSD%20%C2%B7%20Web-1f9d55?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-147a40?style=flat-square">
  <img alt="Agent-drivable over MCP" src="https://img.shields.io/badge/agents-MCP-1f9d55?style=flat-square">
</p>

<p align="center">
  <a href="https://discord.gg/artcraft"><img alt="Join the ArtCraft community on Discord" src="https://img.shields.io/badge/Join%20us%20on%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40"></a>
</p>

<p align="center">
  <a href="https://getartcraft.com/apps/pixelcraft"><b>PixelCraft on getartcraft.com</b></a> ·
  <a href="https://getartcraft.com/">ArtCraft</a> ·
  <a href="https://getartcraft.com/apps">All Crafting Apps</a>
</p>

> [!NOTE]
> **ArtCraft is a community of artists from all walks of life.** Painters, photographers,
> filmmakers, illustrators, designers, animators, hobbyists, and people who picked up a pencil
> last week. If you make things, you're one of us. **[Come say hi on Discord](https://discord.gg/artcraft).**

<p align="center">
  <a href="#why-pixelcraft">Why PixelCraft</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#for-agents-mcp-cli-and-the-control-channel">For agents</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#roadmap">Roadmap</a> ·
  <a href="#license-and-credits">License and credits</a>
</p>

<p align="center">
  <img src="docs/images/interface.png" alt="PixelCraft showing a 32×32 sprite with two crossed strokes at 1200 % zoom, the palette and colour picker on the left, the tool bar on the right and a timeline with two layers and three frames" width="100%">
  <br><sub>The first interface: built entirely with PixelCraft commands (<code>cargo run -p pixelcraft-ui-egui --example snapshot</code>).</sub>
</p>

## Why PixelCraft

- **Familiar.** Aseprite's layout and habits: the menu bar and context bar, the colour bar and
  palette, the tool bar, the timeline of layers × frames, onion skin, tags and its keyboard
  shortcuts. If you know Aseprite, you already know PixelCraft.
- **Compatible.** `.aseprite`/`.ase` is the native format, read and written from the public file
  specification. PNG, GIF and sprite sheets with JSON data follow.
- **Pixel-exact.** RGBA, grayscale and indexed sprites; every tool decides pixel by pixel, with no
  anti-aliasing unless you ask for it.
- **Agent-native.** Every menu item, tool stroke and keystroke is a command that agents can run
  over a JSON control channel, an **MCP server**, or the CLI.
- **Everywhere.** One Rust codebase for desktop and the web. No subscription, no licence server,
  no telemetry.

## Quick start

```sh
cargo run --release -p pixelcraft                                  # desktop app
cargo run --release -p pixelcraft-cli -- commands                  # every command and its params
cargo xtask ci                                                     # fmt, clippy, tests, layers, wasm
```

### Web

```sh
cd apps/pixelcraft-web && trunk serve --release     # http://127.0.0.1:8080
```

You need [trunk](https://trunkrs.dev) and the `wasm32-unknown-unknown` target.

## For agents: MCP, CLI and the control channel

```sh
pixelcraft-cli commands --search layer                             # find commands
pixelcraft-cli run --cmd 'pixel.stroke={"x0":0,"y0":0,"x1":7,"y1":7,"color":"#d24b3c"}' \
                   --cmd layer.new --cmd frame.new --cmd sprite.properties
```

The MCP server (`crates/mcp`) offers `list_commands` and `execute_command`, which reaches every
one of PixelCraft's commands. The stdio transport and the desktop control channel are next; see
[`docs/mcp.md`](docs/mcp.md).

## Architecture

PixelCraft is an engine-first Cargo workspace with enforced layering (`cargo xtask layers`). The
egui frontend is a separate crate, so the UI can be swapped without touching the engine.

| Layer | Crates |
|---|---|
| L0 | `geom` (pixel-grid geometry) · `color` (colour modes, RGBA, palettes) · `raster` (images) · `ase` (standalone `.aseprite` reader/writer) |
| L1 | `doc` (sprite, layers, frames, cels, tags: pure data) |
| L2 | `paint` (pixel-exact tool algorithms: line, fill, shapes) |
| L3 | `compose` (CPU compositor, the reference renderer) |
| L4 | `io` (sprites ↔ `.aseprite`, PNG, GIF, sprite sheets) |
| L5 | `engine` (session, commands, history) |
| L6 | `ui-egui` (Aseprite-style UI) · `mcp` (MCP server) |
| L7 | `apps/pixelcraft`, `apps/pixelcraft-cli`, `apps/pixelcraft-web` |

- **Contributor and agent rules** (clean-room, asset policy, never-crash, quality gates): [`AGENTS.md`](AGENTS.md)
- **How it is built:** [`docs/architecture.md`](docs/architecture.md)
- **Bundled assets:** each one is listed with its licence in [`ATTRIBUTION.md`](ATTRIBUTION.md)

## Roadmap

PixelCraft is in **pre-alpha**: the Aseprite-style layout, zoom ladder, navigation and shortcuts
are in, and the pencil, eraser, eyedropper, zoom and hand tools work. Next up: one undo step per
stroke, the remaining tools, then `.aseprite` read/write and PNG/GIF export. Stage and numbers are in
[ROADMAP.md](ROADMAP.md); the ranked work list is [`docs/gaps.md`](docs/gaps.md).

## The Crafting Apps

PixelCraft is one of the ArtCraft crafting apps, alongside PhotoCraft, VectorCraft, FilmCraft,
LightCraft, DesignCraft, GridCraft and their siblings: [getartcraft.com/apps](https://getartcraft.com/apps).

## License and credits

PixelCraft is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Copyright (c) 2026 ArtCraft Team and the PixelCraft contributors. Required notices are in [NOTICE](NOTICE).

Bundled icons, images and other assets keep their own open licenses; each one is listed with its
author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md). All UI icons are drawn in code and
are original. PixelCraft bundles no fonts: it uses the fonts installed on your system.

The ArtCraft name, wordmark and logos in [`docs/brand/`](docs/brand/) are trademarks of the
ArtCraft Team and are not covered by this license. They may be used only unmodified, and only as
part of this repository and PixelCraft, under `docs/brand/LICENSE-brand.txt`.
Forks and modified versions must remove them.

<sub>Aseprite is a trademark of Igara Studio S.A. PixelCraft is an independent, open-source project and is not affiliated with, sponsored by or endorsed by Igara Studio; the name is used only to describe the workflows and file format it is compatible with.</sub>

<br>

<p align="center">
  <a href="https://getartcraft.com/"><img alt="ArtCraft" src="docs/brand/artcraft-mark.svg" width="28"></a><br>
  <sub>Made by the <a href="https://getartcraft.com/">ArtCraft</a> team and community.</sub>
</p>
