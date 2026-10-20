# Asset attribution

Every non-code asset in this repository (images, icons, palettes, example files) is listed here with its author, source and licence. An asset file without a row is a bug.

**Policy (mandatory):** PixelCraft contains **no Aseprite (Igara Studio) or other vendor iconography, images, skins, themes, fonts, palettes or sample sprites**. Every asset is original work by PixelCraft contributors or third-party material under an open licence (OSI open source, public domain / CC0, or Creative Commons that allows redistribution). Screenshots of Aseprite are never committed. Font files are not added here: shared fonts live in storytold/craft-fonts, an optional build input (`CRAFT_FONTS_DIR`). The one exception is the ArtCraft brand files in `docs/brand/`: ArtCraft trademarks, not open source, used under `docs/brand/LICENSE-brand.txt`.

Generated-in-code art is original and has no file to list: the theme (`crates/ui-egui/src/theme.rs`), the pixel font (`crates/ui-egui/src/pixel_font/`), the pixel icon set (`crates/ui-egui/src/icons.rs`), the pixel cursor (`crates/ui-egui/src/cursor.rs`) and the default palette (`crates/color/src/lib.rs`).

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `apps/pixelcraft-web/index.html` | PixelCraft contributors | original | MIT OR Apache-2.0 | trunk entry page |
| `docs/images/interface.png` | PixelCraft contributors | rendered by PixelCraft itself (`--example snapshot`) | MIT OR Apache-2.0 | README and UI design screenshot |
| `assets/app-icon/pixelcraft.svg` | the PixelCraft project owner | original (drawn in ArtCraft, vectorised) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | App icon master (lizard) |
| `assets/app-icon/pixelcraft-1024.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | macOS render |
| `assets/app-icon/pixelcraft.icns` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | macOS bundle icon |
| `assets/app-icon/pixelcraft.ico` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Windows executable icon |
| `assets/app-icon/hicolor/16x16/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/24x24/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/32x32/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/48x48/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/64x64/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/128x128/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/256x256/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/512x512/apps/ai.storyteller.pixelcraft.png` | the PixelCraft project owner | rendered from `pixelcraft.svg` by `packaging/icons.sh` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux hicolor render |
| `assets/app-icon/hicolor/scalable/apps/ai.storyteller.pixelcraft.svg` | the PixelCraft project owner | copy of `pixelcraft.svg` | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Linux scalable icon |
| `assets/app-icon/README.md`, `assets/app-icon/LICENSE.txt` | PixelCraft contributors | — | — | Icon notes and licence text |
