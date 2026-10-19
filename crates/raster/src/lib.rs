//! PixelCraft raster images: a width × height block of pixels in one [`ColorMode`].
//!
//! Sprites are small, so an [`Image`] is a plain row-major byte buffer. Cels share images through
//! `Arc` in the document model, which keeps undo snapshots cheap; mutation goes through
//! `Arc::make_mut` and copies only the image being edited.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use pixelcraft_color::{ColorMode, Palette, Rgba8};
use pixelcraft_geom::{Point, Size};
use serde::{Deserialize, Serialize};

/// The largest side of an image: anything bigger is refused rather than allocated.
pub const MAX_SIDE: u32 = 65_535;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RasterError {
    #[error("image size {0}×{1} is out of range (1–{MAX_SIDE} pixels per side)")]
    BadSize(u32, u32),
    #[error("pixel data has {got} bytes, expected {want}")]
    BadLength { got: usize, want: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    mode: ColorMode,
    size: Size,
    data: Vec<u8>,
}

impl Image {
    /// A cleared image: transparent for RGBA and grayscale, index 0 for indexed.
    pub fn new(mode: ColorMode, size: Size) -> Result<Self, RasterError> {
        let len = Self::byte_len(mode, size)?;
        Ok(Self { mode, size, data: vec![0; len] })
    }

    pub fn from_bytes(mode: ColorMode, size: Size, data: Vec<u8>) -> Result<Self, RasterError> {
        let want = Self::byte_len(mode, size)?;
        if data.len() != want {
            return Err(RasterError::BadLength { got: data.len(), want });
        }
        Ok(Self { mode, size, data })
    }

    fn byte_len(mode: ColorMode, size: Size) -> Result<usize, RasterError> {
        if size.is_empty() || size.w > MAX_SIDE || size.h > MAX_SIDE {
            return Err(RasterError::BadSize(size.w, size.h));
        }
        size.area().and_then(|a| a.checked_mul(mode.bytes_per_pixel())).ok_or(RasterError::BadSize(size.w, size.h))
    }

    pub fn mode(&self) -> ColorMode {
        self.mode
    }
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    fn offset(&self, p: Point) -> Option<usize> {
        let x = usize::try_from(p.x).ok().filter(|&x| x < self.size.w as usize)?;
        let y = usize::try_from(p.y).ok().filter(|&y| y < self.size.h as usize)?;
        y.checked_mul(self.size.w as usize)?.checked_add(x)?.checked_mul(self.mode.bytes_per_pixel())
    }

    /// The raw pixel bytes at `p`, or `None` outside the image.
    pub fn pixel(&self, p: Point) -> Option<&[u8]> {
        let o = self.offset(p)?;
        self.data.get(o..o + self.mode.bytes_per_pixel())
    }

    /// Writes raw pixel bytes at `p`. Points outside the image and wrong-length pixels are ignored.
    pub fn put_pixel(&mut self, p: Point, px: &[u8]) {
        let bpp = self.mode.bytes_per_pixel();
        if px.len() != bpp {
            return;
        }
        if let Some(dst) = self.offset(p).and_then(|o| self.data.get_mut(o..o + bpp)) {
            dst.copy_from_slice(px);
        }
    }

    /// Sets every pixel to raw pixel `px`; a wrong-length pixel is ignored.
    pub fn fill(&mut self, px: &[u8]) {
        if px.len() != self.mode.bytes_per_pixel() {
            return;
        }
        for dst in self.data.chunks_exact_mut(px.len()) {
            dst.copy_from_slice(px);
        }
    }

    /// The colour at `p` as RGBA (indexed pixels are looked up in `palette`).
    pub fn rgba(&self, p: Point, palette: &Palette) -> Rgba8 {
        match (self.mode, self.pixel(p)) {
            (ColorMode::Rgba, Some(&[r, g, b, a])) => Rgba8::new(r, g, b, a),
            (ColorMode::Grayscale, Some(&[v, a])) => Rgba8::new(v, v, v, a),
            (ColorMode::Indexed, Some(&[i])) => palette.get(i),
            _ => Rgba8::TRANSPARENT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_absurd_sizes() {
        assert_eq!(Image::new(ColorMode::Rgba, Size::new(0, 4)), Err(RasterError::BadSize(0, 4)));
        assert!(Image::new(ColorMode::Rgba, Size::new(MAX_SIDE + 1, 1)).is_err());
        assert!(Image::from_bytes(ColorMode::Indexed, Size::new(2, 2), vec![0; 3]).is_err());
    }

    #[test]
    fn put_and_read_in_every_mode() {
        let pal = Palette::default_16();
        for mode in [ColorMode::Rgba, ColorMode::Grayscale, ColorMode::Indexed] {
            let mut img = Image::new(mode, Size::new(4, 4)).unwrap();
            let px: &[u8] = match mode {
                ColorMode::Rgba => &[255, 255, 255, 255],
                ColorMode::Grayscale => &[255, 255],
                ColorMode::Indexed => &[5],
            };
            img.put_pixel(Point::new(1, 2), px);
            img.put_pixel(Point::new(-1, 99), px); // ignored, no panic
            assert_eq!(img.rgba(Point::new(1, 2), &pal), Rgba8::WHITE, "{mode:?}");
            assert_eq!(img.rgba(Point::new(9, 9), &pal), Rgba8::TRANSPARENT);
        }
    }
}
