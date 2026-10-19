//! A clean-room reader and writer for the `.ase`/`.aseprite` file format.
//!
//! Implemented from the published format specification only (little-endian header, frames,
//! chunks), never from any program's source code. Standalone: no workspace dependencies, so it can
//! be published on its own. Files are hostile input: every length and count is checked before it
//! is used, and a malformed file is an [`AseError`], never a panic.
//!
//! Status: the 128-byte file header. Frames and chunks follow (see `docs/file-format-parity.md`).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

pub const HEADER_LEN: usize = 128;
pub const HEADER_MAGIC: u16 = 0xA5E0;
pub const FRAME_MAGIC: u16 = 0xF1FA;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AseError {
    #[error("the file is too short to be an .aseprite file ({0} bytes)")]
    TooShort(usize),
    #[error("not an .aseprite file (header magic {0:#06x})")]
    BadMagic(u16),
    #[error("unsupported colour depth {0} bits per pixel")]
    BadDepth(u16),
    #[error("sprite size {0}×{1} is out of range")]
    BadSize(u16, u16),
}

/// Bits per pixel in the header: 32 = RGBA, 16 = grayscale, 8 = indexed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    Rgba,
    Grayscale,
    Indexed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub file_size: u32,
    pub frames: u16,
    pub width: u16,
    pub height: u16,
    pub depth: Depth,
    pub flags: u32,
    /// Palette entry that is transparent (indexed sprites only).
    pub transparent_index: u8,
    /// Number of colours (0 means 256 in old files).
    pub num_colors: u16,
    pub pixel_ratio: (u8, u8),
    pub grid: (i16, i16, u16, u16),
}

fn u16_at(b: &[u8], o: usize) -> u16 {
    b.get(o..o + 2).map_or(0, |s| u16::from_le_bytes([s[0], s[1]]))
}
fn i16_at(b: &[u8], o: usize) -> i16 {
    u16_at(b, o) as i16
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4).map_or(0, |s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

impl Header {
    pub fn parse(bytes: &[u8]) -> Result<Self, AseError> {
        let b = bytes.get(..HEADER_LEN).ok_or(AseError::TooShort(bytes.len()))?;
        let magic = u16_at(b, 4);
        if magic != HEADER_MAGIC {
            return Err(AseError::BadMagic(magic));
        }
        let depth = match u16_at(b, 12) {
            32 => Depth::Rgba,
            16 => Depth::Grayscale,
            8 => Depth::Indexed,
            d => return Err(AseError::BadDepth(d)),
        };
        let (width, height) = (u16_at(b, 8), u16_at(b, 10));
        if width == 0 || height == 0 {
            return Err(AseError::BadSize(width, height));
        }
        Ok(Header {
            file_size: u32_at(b, 0),
            frames: u16_at(b, 6),
            width,
            height,
            depth,
            flags: u32_at(b, 14),
            transparent_index: b.get(28).copied().unwrap_or(0),
            num_colors: u16_at(b, 32),
            pixel_ratio: (b.get(34).copied().unwrap_or(1), b.get(35).copied().unwrap_or(1)),
            grid: (i16_at(b, 36), i16_at(b, 38), u16_at(b, 40), u16_at(b, 42)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(w: u16, h: u16, depth: u16) -> Vec<u8> {
        let mut b = vec![0u8; HEADER_LEN];
        b[4..6].copy_from_slice(&HEADER_MAGIC.to_le_bytes());
        b[6..8].copy_from_slice(&1u16.to_le_bytes());
        b[8..10].copy_from_slice(&w.to_le_bytes());
        b[10..12].copy_from_slice(&h.to_le_bytes());
        b[12..14].copy_from_slice(&depth.to_le_bytes());
        b
    }

    #[test]
    fn parses_a_synthetic_header() {
        let h = Header::parse(&header(32, 16, 8)).unwrap();
        assert_eq!((h.width, h.height, h.depth, h.frames), (32, 16, Depth::Indexed, 1));
    }

    #[test]
    fn rejects_malformed_headers() {
        assert_eq!(Header::parse(&[0; 10]), Err(AseError::TooShort(10)));
        assert_eq!(Header::parse(&[0; HEADER_LEN]), Err(AseError::BadMagic(0)));
        assert_eq!(Header::parse(&header(32, 32, 24)), Err(AseError::BadDepth(24)));
        assert_eq!(Header::parse(&header(0, 32, 32)), Err(AseError::BadSize(0, 32)));
    }
}
