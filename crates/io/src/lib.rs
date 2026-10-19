//! Import and export between PixelCraft sprites and files.
//!
//! `.aseprite`/`.ase` is the native format (through the standalone `pixelcraft-ase` crate). PNG,
//! GIF and sprite-sheet export follow. Everything read from a file is hostile input.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use pixelcraft_ase::{AseError, Depth, Header};
use pixelcraft_color::ColorMode;
use pixelcraft_doc::Sprite;
use pixelcraft_geom::Size;
use pixelcraft_raster::RasterError;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error(transparent)]
    Ase(#[from] AseError),
    #[error(transparent)]
    Raster(#[from] RasterError),
    #[error("{0} is not supported yet")]
    Unsupported(&'static str),
}

/// Reads an `.aseprite` file. For now: the canvas and colour mode from its header, with one
/// empty layer and frame (frames and chunks land next).
pub fn read_aseprite(bytes: &[u8]) -> Result<Sprite, IoError> {
    let h = Header::parse(bytes)?;
    let mode = match h.depth {
        Depth::Rgba => ColorMode::Rgba,
        Depth::Grayscale => ColorMode::Grayscale,
        Depth::Indexed => ColorMode::Indexed,
    };
    Ok(Sprite::new(Size::new(u32::from(h.width), u32::from(h.height)), mode)?)
}

/// Writes an `.aseprite` file.
pub fn write_aseprite(_sprite: &Sprite) -> Result<Vec<u8>, IoError> {
    Err(IoError::Unsupported("saving .aseprite files"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage_is_an_error_not_a_panic() {
        assert!(read_aseprite(&[]).is_err());
        assert!(read_aseprite(&[0xff; 300]).is_err());
    }
}
