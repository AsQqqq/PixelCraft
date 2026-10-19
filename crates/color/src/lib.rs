//! PixelCraft colour: the three colour modes of a sprite (RGBA, grayscale, indexed), RGBA pixels
//! and palettes.
//!
//! The colour mode is runtime data. Never add an RGBA-only path to a public API: indexed sprites
//! are first-class in pixel art.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// How a sprite stores its pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorMode {
    /// 4 bytes per pixel: red, green, blue, alpha (straight, not premultiplied).
    #[default]
    Rgba,
    /// 2 bytes per pixel: value, alpha.
    Grayscale,
    /// 1 byte per pixel: an index into the sprite's palette.
    Indexed,
}

impl ColorMode {
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            ColorMode::Rgba => 4,
            ColorMode::Grayscale => 2,
            ColorMode::Indexed => 1,
        }
    }
}

/// A straight-alpha 8-bit RGBA colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Rgba8 = Rgba8::new(0, 0, 0, 0);
    pub const BLACK: Rgba8 = Rgba8::new(0, 0, 0, 255);
    pub const WHITE: Rgba8 = Rgba8::new(255, 255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }
    /// `#rrggbb` or `#rrggbbaa` (the `#` is optional).
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        let byte = |i: usize| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok());
        match s.len() {
            6 => Some(Self::opaque(byte(0)?, byte(2)?, byte(4)?)),
            8 => Some(Self::new(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => None,
        }
    }
    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

/// An ordered list of colours. Indexed sprites refer to entries by position (at most 256).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
    pub colors: Vec<Rgba8>,
}

impl Palette {
    pub const MAX_INDEXED: usize = 256;

    pub fn get(&self, index: u8) -> Rgba8 {
        self.colors.get(usize::from(index)).copied().unwrap_or(Rgba8::TRANSPARENT)
    }

    /// The default palette of a new sprite: a small, original 16-colour ramp set.
    /// (Defined here in code, never copied from another program.)
    pub fn default_16() -> Self {
        const HEX: [&str; 16] = [
            "#000000", "#1d1a2b", "#3b3b58", "#6b6b8a", "#a8a8c0", "#ffffff", "#7a2e2e", "#d24b3c", "#f29a4a", "#f7d56a", "#2f6b3a", "#5fb04f",
            "#2a4b8d", "#4f8fe0", "#8a4fb0", "#e07ab8",
        ];
        Self { colors: HEX.iter().filter_map(|h| Rgba8::from_hex(h)).collect() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip_and_garbage() {
        assert_eq!(Rgba8::from_hex("#d24b3c").map(Rgba8::to_hex).as_deref(), Some("#d24b3c"));
        assert_eq!(Rgba8::from_hex("11223344"), Some(Rgba8::new(0x11, 0x22, 0x33, 0x44)));
        assert_eq!(Rgba8::from_hex("#zz0000"), None);
        assert_eq!(Rgba8::from_hex("#ééé"), None);
    }

    #[test]
    fn default_palette_is_complete_and_out_of_range_reads_transparent() {
        let p = Palette::default_16();
        assert_eq!(p.colors.len(), 16);
        assert_eq!(p.get(200), Rgba8::TRANSPARENT);
    }
}
