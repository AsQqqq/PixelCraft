# PixelCraft architecture

> **Last updated:** 2026-10-19

## Crate map

```text
crates/
  geom color raster ase   L0 pixel-grid geometry, colour modes and palettes, images; standalone .aseprite codec
  doc                     L1 document model: sprite, layers, frames, cels, tags (pure data)
  paint                   L2 pixel-exact tool algorithms (line, fill, shapes)
  compose                 L3 CPU compositor (the reference renderer)
  io                      L4 sprites <-> .aseprite, PNG, GIF, sprite sheets
  engine                  L5 Session + command registry (every action is a command)
  ui-egui mcp             L6 egui shell (thin); MCP server
apps/
  pixelcraft              desktop app (eframe/wgpu)
  pixelcraft-cli          headless CLI (commands, run; later convert, info, mcp)
  pixelcraft-web          the same UI in the browser (trunk + wasm-bindgen)
xtask/                    cargo xtask ci | layers | wasm
```

A crate may depend only on lower layers. Inside L0, `geom` → `color` → `raster`; `ase` depends on
nothing in the workspace. Nothing below L6 uses egui, eframe, winit or rfd.

## The document model

A `Sprite` is a grid of **layers × frames**; where a layer meets a frame there may be a `Cel`
(an image at an offset). `layers[0]` is the bottom layer. Images are `Arc`-shared: a new frame
shares its images with the frame it copies, and an undo snapshot is an `Arc<Sprite>` clone.
Mutation goes through `Arc::make_mut`, which copies only what changes.

## The engine/UI seam

Every action is a command (`crates/engine/src/cmd/*`): `id`, `label`, `menu` path, `shortcut`,
`params` doc, `enabled`, `run`, `undoable`. `Session::execute(id, params)`:

1. finds the command and checks `enabled`;
2. snapshots the active sprite;
3. runs it inside a panic guard;
4. on error or panic restores the sprite; on success of an undoable command that changed the
   sprite, pushes the snapshot on the undo stack.

The UI, CLI, MCP and (later) the control channel all go through it. The menu bar is built from
the command catalog, so a new command with a `menu` path shows up there by itself.
