# Asset attribution

Every non-code asset in this repository (images, icons, palettes, example files) is listed here with its author, source and licence. An asset file without a row is a bug.

**Policy (mandatory):** PixelCraft contains **no Aseprite (Igara Studio) or other vendor iconography, images, skins, themes, fonts, palettes or sample sprites**. Every asset is original work by PixelCraft contributors or third-party material under an open licence (OSI open source, public domain / CC0, or Creative Commons that allows redistribution). Screenshots of Aseprite are never committed. Font files are not added here: shared fonts live in storytold/craft-fonts, an optional build input (`CRAFT_FONTS_DIR`). The one exception is the ArtCraft brand files in `docs/brand/`: ArtCraft trademarks, not open source, used under `docs/brand/LICENSE-brand.txt`.

Generated-in-code art is original and has no file to list: the theme (`crates/ui-egui/src/theme.rs`), the pixel font (`crates/ui-egui/src/pixel_font/`), the pixel icon set (`crates/ui-egui/src/icons.rs`), the pixel cursor (`crates/ui-egui/src/cursor.rs`) and the default palette (`crates/color/src/lib.rs`).

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `assets/app-icon/hicolor/scalable/apps/ai.storyteller.pixelcraft.svg` | PixelCraft contributors | original (hand-written SVG) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | PLACEHOLDER app icon (pixel heart) |
| `assets/app-icon/LICENSE.txt` | (licence text) | — | — | |
| `apps/pixelcraft-web/index.html` | PixelCraft contributors | original | MIT OR Apache-2.0 | trunk entry page |
| `docs/images/interface.png` | PixelCraft contributors | rendered by PixelCraft itself (`--example snapshot`) | MIT OR Apache-2.0 | README and UI design screenshot |
