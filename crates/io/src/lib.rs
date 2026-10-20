//! Import and export between PixelCraft sprites and files.
//!
//! `.aseprite`/`.ase` is the native format (through the standalone `pixelcraft-ase` crate).
//! Images (PNG, JPEG, GIF, BMP, WebP, TGA, ICO, TIFF, PNM, QOI) open as new RGBA sprites; an
//! animated GIF keeps its frames and their durations. Export follows. Everything read from a
//! file is hostile input: sizes are capped before anything is decoded.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::io::Cursor;
use std::sync::Arc;

use image::{AnimationDecoder, ImageDecoder, ImageFormat, ImageReader, Limits};
use pixelcraft_ase::{AseError, Depth, HEADER_MAGIC, Header};
use pixelcraft_color::ColorMode;
use pixelcraft_doc::{Cel, Frame, MAX_FRAMES, Sprite};
use pixelcraft_geom::{Point, Size};
use pixelcraft_raster::{Image, MAX_SIDE, RasterError};

/// The most memory one opened image may decode into (a 16384×8192 RGBA image).
pub const MAX_DECODE_BYTES: u64 = 512 * 1024 * 1024;

/// The file extensions [`read_image`] understands (for Open dialogs and drop filters).
pub const IMAGE_EXTENSIONS: &[&str] =
    &["png", "jpg", "jpeg", "jfif", "gif", "bmp", "webp", "tga", "ico", "tif", "tiff", "pbm", "pgm", "ppm", "pnm", "qoi"];

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error(transparent)]
    Ase(#[from] AseError),
    #[error(transparent)]
    Raster(#[from] RasterError),
    #[error("{0} is not supported yet")]
    Unsupported(&'static str),
    #[error("can't read the image: {0}")]
    Image(String),
}

impl From<image::ImageError> for IoError {
    fn from(e: image::ImageError) -> Self {
        IoError::Image(e.to_string())
    }
}

fn limits() -> Limits {
    let mut l = Limits::default();
    l.max_image_width = Some(MAX_SIDE);
    l.max_image_height = Some(MAX_SIDE);
    l.max_alloc = Some(MAX_DECODE_BYTES);
    l
}

/// Opens an image file's bytes as a new sprite: one layer, RGBA. An image without an alpha
/// channel (a photo) becomes an opaque `Background` layer; an animated GIF keeps every frame
/// (up to [`MAX_FRAMES`]) with its duration. `.aseprite` files are recognised and refused until
/// the full reader lands, rather than opening as an empty canvas.
pub fn read_image(bytes: &[u8]) -> Result<Sprite, IoError> {
    if bytes.len() >= 6 && bytes.get(4..6) == Some(&HEADER_MAGIC.to_le_bytes()[..]) {
        return Err(IoError::Unsupported("opening .aseprite files"));
    }
    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|e| IoError::Image(e.to_string()))?;
    match reader.format() {
        None => Err(IoError::Image("not an image PixelCraft can open".into())),
        Some(ImageFormat::Gif) => read_gif(bytes),
        Some(_) => {
            let mut reader = reader;
            reader.limits(limits());
            let img = reader.decode()?;
            let opaque = !img.color().has_alpha();
            let rgba = img.into_rgba8();
            let size = Size::new(rgba.width(), rgba.height());
            sprite_from_frames(size, vec![(rgba.into_raw(), Frame::default().duration_ms)], opaque)
        }
    }
}

fn read_gif(bytes: &[u8]) -> Result<Sprite, IoError> {
    let mut decoder = image::codecs::gif::GifDecoder::new(Cursor::new(bytes))?;
    decoder.set_limits(limits())?;
    let (w, h) = decoder.dimensions();
    let size = Size::new(w, h);
    let mut frames = Vec::new();
    // Bound the decoded total, not only each frame: a long animation of a big canvas adds up.
    let frame_bytes = u64::from(w).saturating_mul(u64::from(h)).saturating_mul(4).max(1);
    let max_frames = usize::try_from(MAX_DECODE_BYTES / frame_bytes).unwrap_or(MAX_FRAMES).clamp(1, MAX_FRAMES);
    for frame in decoder.into_frames().take(max_frames) {
        let frame = frame?;
        let (num, den) = frame.delay().numer_denom_ms();
        // GIFs often say 0 (or 10 ms) and browsers show 100 ms; so does PixelCraft.
        let ms = num.checked_div(den).unwrap_or(0);
        let ms = if ms < 20 { 100 } else { ms };
        frames.push((frame.into_buffer().into_raw(), ms));
    }
    sprite_from_frames(size, frames, false)
}

/// A one-layer RGBA sprite of `size` with one cel per frame.
fn sprite_from_frames(size: Size, frames: Vec<(Vec<u8>, u32)>, opaque: bool) -> Result<Sprite, IoError> {
    if frames.is_empty() {
        return Err(IoError::Image("the image has no frames".into()));
    }
    let mut sprite = Sprite::new(size, ColorMode::Rgba)?;
    sprite.frames.clear();
    sprite.cels.clear();
    for (i, (data, ms)) in frames.into_iter().enumerate() {
        let image = Image::from_bytes(ColorMode::Rgba, size, data)?;
        sprite.frames.push(Frame { duration_ms: ms });
        sprite.cels.push(Cel { layer: 0, frame: i, position: Point::default(), opacity: 255, image: Arc::new(image) });
    }
    if opaque && let Some(layer) = sprite.layers.first_mut() {
        layer.name = "Background".into();
        layer.background = true;
    }
    Ok(sprite)
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
        for bad in [&[][..], &[0xff; 300], b"\x89PNG\r\n\x1a\n broken", b"GIF89a", b"\xff\xd8\xff\xe0 not a jpeg"] {
            assert!(read_image(bad).is_err(), "{bad:?}");
        }
    }

    fn encode(img: &image::DynamicImage, format: ImageFormat) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, format).unwrap();
        out.into_inner()
    }

    #[test]
    fn a_png_with_alpha_opens_as_a_transparent_layer() {
        let mut img = image::RgbaImage::new(3, 2);
        img.put_pixel(2, 1, image::Rgba([10, 20, 30, 255]));
        let sprite = read_image(&encode(&img.into(), ImageFormat::Png)).unwrap();
        assert_eq!(sprite.size, Size::new(3, 2));
        assert_eq!(sprite.frames.len(), 1);
        assert!(!sprite.layers[0].background);
        let cel = sprite.cel(0, 0).unwrap();
        assert_eq!(cel.image.pixel(Point::new(2, 1)), Some(&[10, 20, 30, 255][..]));
        assert_eq!(cel.image.pixel(Point::new(0, 0)), Some(&[0, 0, 0, 0][..]));
    }

    #[test]
    fn a_photo_without_alpha_opens_as_a_background_layer() {
        let img = image::RgbImage::from_pixel(4, 4, image::Rgb([200, 100, 50]));
        for format in [ImageFormat::Jpeg, ImageFormat::Bmp, ImageFormat::Png] {
            let sprite = read_image(&encode(&img.clone().into(), format)).unwrap();
            assert!(sprite.layers[0].background, "{format:?}");
            assert_eq!(sprite.layers[0].name, "Background");
            let px = sprite.cel(0, 0).unwrap().image.pixel(Point::new(1, 1)).unwrap().to_vec();
            assert_eq!(px[3], 255, "{format:?}");
        }
    }

    #[test]
    fn an_animated_gif_keeps_its_frames_and_durations() {
        use image::codecs::gif::{GifEncoder, Repeat};
        let mut out = Vec::new();
        {
            let mut enc = GifEncoder::new(&mut out);
            enc.set_repeat(Repeat::Infinite).unwrap();
            for (i, ms) in [(0u8, 200u32), (1, 50), (2, 0)] {
                let buf = image::RgbaImage::from_pixel(2, 2, image::Rgba([i * 100, 0, 0, 255]));
                enc.encode_frame(image::Frame::from_parts(buf, 0, 0, image::Delay::from_numer_denom_ms(ms, 1))).unwrap();
            }
        }
        let sprite = read_image(&out).unwrap();
        assert_eq!(sprite.frames.len(), 3);
        assert_eq!(sprite.frames.iter().map(|f| f.duration_ms).collect::<Vec<_>>(), vec![200, 50, 100]);
        assert_eq!(sprite.cel(0, 2).unwrap().image.pixel(Point::new(0, 0)).map(|p| p[0]), Some(200));
    }

    #[test]
    fn an_aseprite_file_is_refused_not_opened_empty() {
        let mut b = vec![0u8; 128];
        b[4..6].copy_from_slice(&HEADER_MAGIC.to_le_bytes());
        assert!(matches!(read_image(&b), Err(IoError::Unsupported(_))));
    }
}
