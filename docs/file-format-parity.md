# File format parity

> **Last updated:** 2026-10-20

| Format | Read | Write | Notes |
|---|---|---|---|
| `.aseprite` / `.ase` | header (size, colour mode) | — | from the public specification; `crates/ase` |

| GIF | — | — | |
| Sprite sheet + JSON | — | — | |

## Opening images

File ▸ Open... (Cmd+O, the Home tab's link) shows the OS's dialog (rfd, from the app crate; the
browser's file picker on the web); dropping files on the window opens them too. Each file opens
as a new tab named after the file, fitted to the editor when it is bigger. A file that is already
open becomes the active tab. Decoding is capped (`raster::MAX_SIDE` per side,
`io::MAX_DECODE_BYTES` in total), and images bigger than the GPU's texture limit are drawn in
tiles (`ui-egui/src/tiles.rs`). Adding an image to an existing sprite (as a layer) comes later.
