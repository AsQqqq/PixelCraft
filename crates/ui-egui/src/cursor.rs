//! Mouse cursors. With the pixel style (the default) every cursor is a **real OS cursor** made
//! from PixelCraft's own pixel-art sprites (original art, drawn in code): egui hands the RGBA
//! image to the window system (`Context::set_cursor_image` → winit custom cursor), so the OS
//! draws it everywhere — in the window, in dialogs, popups and menus — with no lag, and nothing is
//! painted into the frame that could be left behind when the pointer leaves. With the native
//! style the OS's own cursors are used (Preferences ▸ Cursors ▸ "Use native mouse cursors").
//!
//! Platforms without custom cursor support (the web build today) fall back to the matching
//! standard cursor.
//!
//! In the art, `#` is the outline (black), `o` the fill (white); the hotspot is the pixel that
//! points.

use std::collections::HashMap;
use std::sync::Arc;

use egui::{CursorIcon, CustomCursorImage};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sprite {
    Arrow,
    Text,
    Hand,
    ResizeH,
    ResizeV,
    Crosshair,
}

struct Art {
    rows: &'static [&'static str],
    hotspot: (u16, u16),
}

const ARROW: Art = Art {
    rows: &[
        "#.........",
        "##........",
        "#o#.......",
        "#oo#......",
        "#ooo#.....",
        "#oooo#....",
        "#ooooo#...",
        "#oooooo#..",
        "#ooooooo#.",
        "#oooo####.",
        "#oo#o#....",
        "#o#.#o#...",
        "##..#o#...",
        ".....#o#..",
        ".....###..",
    ],
    hotspot: (0, 0),
};

const TEXT: Art = Art {
    rows: &[
        "###.###", "#oo#oo#", "###o###", "..#o#..", "..#o#..", "..#o#..", "..#o#..", "..#o#..", "..#o#..", "..#o#..", "###o###", "#oo#oo#", "###.###",
    ],
    hotspot: (3, 6),
};

const HAND: Art = Art {
    rows: &[
        "...##.##.##.",
        "..#oo#oo#oo#",
        "..#oo#oo#oo#",
        "..#oooooooo#",
        ".##oooooooo#",
        "#oo#ooooooo#",
        "#ooooooooo#.",
        ".#oooooooo#.",
        "..#ooooooo#.",
        "...#ooooo#..",
        "....#####...",
    ],
    hotspot: (6, 5),
};

const RESIZE_H: Art =
    Art { rows: &["..#.....#..", ".#o#...#o#.", "#oo#####oo#", "#ooooooooo#", "#oo#####oo#", ".#o#...#o#.", "..#.....#.."], hotspot: (5, 3) };

const RESIZE_V: Art = Art {
    rows: &["..###..", ".#ooo#.", "#ooooo#", "###o###", "..#o#..", "..#o#..", "..#o#..", "###o###", "#ooooo#", ".#ooo#.", "..###.."],
    hotspot: (3, 5),
};

/// The canvas cursor: a crosshair with an open centre, so the pixel under it stays visible.
const CROSSHAIR: Art = Art {
    rows: &[
        "....###....",
        "....#o#....",
        "....#o#....",
        "....#o#....",
        "####...####",
        "#ooo...ooo#",
        "####...####",
        "....#o#....",
        "....#o#....",
        "....#o#....",
        "....###....",
    ],
    hotspot: (5, 5),
};

impl Sprite {
    pub const ALL: [Sprite; 6] = [Sprite::Arrow, Sprite::Text, Sprite::Hand, Sprite::ResizeH, Sprite::ResizeV, Sprite::Crosshair];

    /// The sprite for what the widgets asked for (`None` keeps the cursor hidden).
    pub fn for_icon(icon: CursorIcon) -> Option<Self> {
        use CursorIcon::*;
        Some(match icon {
            None => return Option::None,
            Crosshair | Cell => Sprite::Crosshair,
            Text | VerticalText => Sprite::Text,
            Grab | Grabbing | Move | AllScroll => Sprite::Hand,
            ResizeHorizontal | ResizeColumn | ResizeEast | ResizeWest => Sprite::ResizeH,
            ResizeVertical | ResizeRow | ResizeNorth | ResizeSouth => Sprite::ResizeV,
            _ => Sprite::Arrow,
        })
    }

    fn art(self) -> &'static Art {
        match self {
            Sprite::Arrow => &ARROW,
            Sprite::Text => &TEXT,
            Sprite::Hand => &HAND,
            Sprite::ResizeH => &RESIZE_H,
            Sprite::ResizeV => &RESIZE_V,
            Sprite::Crosshair => &CROSSHAIR,
        }
    }

    /// The sprite as an RGBA image, each art pixel `scale`×`scale` image pixels.
    pub fn image(self, scale: u16) -> CustomCursorImage {
        let art = self.art();
        let s = scale.clamp(1, 8);
        let (w, h) = (art.rows.first().map_or(0, |r| r.len()) as u16, art.rows.len() as u16);
        let (iw, ih) = (w * s, h * s);
        let mut rgba = vec![0u8; usize::from(iw) * usize::from(ih) * 4];
        for (y, row) in art.rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let v = match ch {
                    '#' => 0,
                    'o' => 255,
                    _ => continue,
                };
                for dy in 0..usize::from(s) {
                    for dx in 0..usize::from(s) {
                        let i = ((y * usize::from(s) + dy) * usize::from(iw) + x * usize::from(s) + dx) * 4;
                        if let Some(px) = rgba.get_mut(i..i + 4) {
                            px.copy_from_slice(&[v, v, v, 255]);
                        }
                    }
                }
            }
        }
        CustomCursorImage { rgba: Arc::from(rgba), size: [iw, ih], hotspot: [art.hotspot.0 * s, art.hotspot.1 * s] }
    }
}

/// The cursor images, made once per scale and reused (the integration dedupes by `Arc`).
#[derive(Default)]
pub struct Cursors {
    images: HashMap<(Sprite, u16), CustomCursorImage>,
}

impl Cursors {
    fn image(&mut self, sprite: Sprite, scale: u16) -> CustomCursorImage {
        self.images.entry((sprite, scale)).or_insert_with(|| sprite.image(scale)).clone()
    }

    /// End of frame: with the pixel style, turn the cursor egui's widgets asked for into the
    /// matching pixel-art OS cursor. `scale` is image pixels per art pixel.
    pub fn apply(&mut self, ctx: &egui::Context, pixel: bool, scale: u16) {
        if !pixel {
            return;
        }
        let wanted = ctx.output(|o| o.cursor_icon);
        if let Some(sprite) = Sprite::for_icon(wanted) {
            let img = self.image(sprite, scale);
            ctx.set_cursor_image(Some(img));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprites_are_rectangular_and_hotspots_inside() {
        for s in Sprite::ALL {
            let art = s.art();
            let w = art.rows.first().map_or(0, |r| r.len());
            assert!(art.rows.iter().all(|r| r.len() == w), "{s:?}");
            assert!(usize::from(art.hotspot.0) < w && usize::from(art.hotspot.1) < art.rows.len(), "{s:?}");
        }
    }

    #[test]
    fn images_scale_and_keep_the_hotspot() {
        let img = Sprite::Crosshair.image(3);
        assert_eq!(img.size, [33, 33]);
        assert_eq!(img.hotspot, [15, 15]);
        assert_eq!(img.rgba.len(), 33 * 33 * 4);
        // The open centre stays transparent; the outline is opaque black.
        let at = |x: usize, y: usize| img.rgba.get((y * 33 + x) * 4 + 3).copied();
        assert_eq!(at(16, 16), Some(0));
        assert_eq!(at(13, 1), Some(255));
    }

    #[test]
    fn images_are_made_once_per_scale() {
        let mut c = Cursors::default();
        let a = c.image(Sprite::Arrow, 2);
        let b = c.image(Sprite::Arrow, 2);
        assert!(Arc::ptr_eq(&a.rgba, &b.rgba));
    }

    #[test]
    fn widget_cursors_map_to_sprites() {
        assert_eq!(Sprite::for_icon(CursorIcon::Crosshair), Some(Sprite::Crosshair));
        assert_eq!(Sprite::for_icon(CursorIcon::Default), Some(Sprite::Arrow));
        assert_eq!(Sprite::for_icon(CursorIcon::Text), Some(Sprite::Text));
        assert_eq!(Sprite::for_icon(CursorIcon::Grab), Some(Sprite::Hand));
        assert_eq!(Sprite::for_icon(CursorIcon::None), None);
    }

    #[test]
    fn the_pixel_style_sets_an_os_cursor_image() {
        let ctx = egui::Context::default();
        let mut c = Cursors::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            ui.ctx().set_cursor_icon(CursorIcon::Text);
            c.apply(ui.ctx(), true, 2);
        });
        out.textures_delta.clear();
        assert_eq!(out.platform_output.cursor_icon, CursorIcon::Text, "the standard icon stays as the fallback");
        assert_eq!(out.platform_output.cursor_image.map(|i| i.size), Some([14, 26]));
        let mut out = ctx.run_ui(Default::default(), |ui| c.apply(ui.ctx(), false, 2));
        out.textures_delta.clear();
        assert!(out.platform_output.cursor_image.is_none(), "native style: no image");
    }
}
