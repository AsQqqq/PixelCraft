# PixelCraft app icon

A **lizard**: an engraving portrait, standing, head turned towards the viewer, tail curling
behind, in the Crafting Apps' framing (full-bleed field, the figure filling the tile).

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Ink | `#0b0b0c` / `#000000` | line work and contours |
| Paper | `#efe9dc` | the figure |
| PixelCraft orange (app colour) | `#f7931e` | the full-bleed field |

## Geometry

A 512-unit tile (`viewBox="0 0 512 512"`), rounded square with `rx=112`, no border. The macOS
renders pad it onto Apple's 824/1024 icon grid; the Windows and Linux renders crop 22 units off
each side so the figure reads at 16–48 px.

## Provenance

The owner's original drawing, made in ArtCraft in the engraving style, vectorised. License: see
`LICENSE.txt`.

## Files

| File | What | Made by |
|---|---|---|
| `pixelcraft.svg` | the master | the owner |
| `pixelcraft-1024.png` | macOS, padded to Apple's grid | `packaging/icons.sh` |
| `pixelcraft.icns` | macOS app bundle icon (16–1024 px) | `packaging/icons.sh` (iconutil) |
| `pixelcraft.ico` | Windows executable icon (16–256 px, PNG entries) | `packaging/icons.sh` |
| `hicolor/<n>x<n>/apps/ai.storyteller.pixelcraft.png` | Linux icon theme, 16–512 px | `packaging/icons.sh` |
| `hicolor/scalable/apps/ai.storyteller.pixelcraft.svg` | Linux scalable icon (copy of the master) | `packaging/icons.sh` |

Where it shows: the window and taskbar (`apps/pixelcraft/src/app_icon.rs`), the Windows
executable (`apps/pixelcraft/build.rs`), the web build's favicon
(`apps/pixelcraft-web/index.html`) and the Home tab (`crates/ui-egui/src/panels/home.rs`).
