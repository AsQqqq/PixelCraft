//! Scaling and rotating an image, for pasting with the transform box: nearest-neighbour, so
//! pixel art stays crisp (every destination pixel takes exactly one source pixel).

use pixelcraft_geom::{Point, Size};

use crate::{Image, Mask, RasterError};

/// The most pixels a transformed image may have (a 8192×8192 RGBA image is 256 MiB).
pub const MAX_PIXELS: u64 = 8192 * 8192;

/// `src` stretched to `w`×`h` and rotated by `angle_deg` (clockwise on screen) about the box's
/// centre. Returns the image (same colour mode, untouched pixels cleared), the pixels the box
/// covers, and where the result's top-left corner sits relative to the unrotated box's.
pub fn scale_rotate(src: &Image, w: u32, h: u32, angle_deg: f32) -> Result<(Image, Mask, Point), RasterError> {
    if w == 0 || h == 0 || w > crate::MAX_SIDE || h > crate::MAX_SIDE {
        return Err(RasterError::BadSize(w, h));
    }
    let a = if angle_deg.is_finite() { angle_deg.to_radians() } else { 0.0 };
    let (sin, cos) = a.sin_cos();
    let (bw, bh) = (f64::from(w), f64::from(h));
    let (sin, cos) = (f64::from(sin), f64::from(cos));
    // The rotated box's bounds, around the box's centre.
    let (cx, cy) = (bw / 2.0, bh / 2.0);
    let corners = [(-cx, -cy), (cx, -cy), (cx, cy), (-cx, cy)].map(|(x, y)| (x * cos - y * sin, x * sin + y * cos));
    let min_x = corners.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let max_x = corners.iter().map(|c| c.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = corners.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    let max_y = corners.iter().map(|c| c.1).fold(f64::NEG_INFINITY, f64::max);
    // The rotated box starts at a fractional point; its pixels are sampled at whole steps from
    // there, so a quarter turn maps pixel centres onto pixel centres (lossless), and the result
    // lands on the canvas at the nearest whole pixel.
    let (ox, oy) = (cx + min_x, cy + min_y);
    let (dw, dh) = (((max_x - min_x) - 1e-6).ceil().max(1.0) as u32, ((max_y - min_y) - 1e-6).ceil().max(1.0) as u32);
    if u64::from(dw) * u64::from(dh) > MAX_PIXELS {
        return Err(RasterError::BadSize(dw, dh));
    }
    let size = Size::new(dw, dh);
    let mut out = Image::new(src.mode(), size)?;
    let mut mask = Mask::new(size)?;
    let (sw, sh) = (f64::from(src.size().w), f64::from(src.size().h));
    for dy in 0..dh {
        for dx in 0..dw {
            // This pixel's centre, relative to the box's centre, rotated back.
            let px = ox + f64::from(dx) + 0.5 - cx;
            let py = oy + f64::from(dy) + 0.5 - cy;
            let u = px * cos + py * sin + cx;
            let v = -px * sin + py * cos + cy;
            // A hair of tolerance: centres that land exactly on the box's edge are inside.
            let (u, v) = (u.clamp(-1e-9, bw) + 1e-9, v.clamp(-1e-9, bh) + 1e-9);
            if u <= 0.0 || v <= 0.0 || u >= bw || v >= bh {
                continue;
            }
            let sx = ((u * sw / bw).floor() as i64).clamp(0, sw as i64 - 1) as i32;
            let sy = ((v * sh / bh).floor() as i64).clamp(0, sh as i64 - 1) as i32;
            let d = Point::new(dx as i32, dy as i32);
            if let Some(p) = src.pixel(Point::new(sx, sy)) {
                out.put_pixel(d, p);
            }
            mask.set(d, true);
        }
    }
    Ok((out, mask, Point::new((ox + 0.5 + 1e-6).floor() as i32, (oy + 0.5 + 1e-6).floor() as i32)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelcraft_color::ColorMode;

    fn numbered(w: u32, h: u32) -> Image {
        let mut img = Image::new(ColorMode::Grayscale, Size::new(w, h)).unwrap();
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                img.put_pixel(Point::new(x, y), &[(y * w as i32 + x) as u8, 255]);
            }
        }
        img
    }

    fn at(img: &Image, x: i32, y: i32) -> u8 {
        img.pixel(Point::new(x, y)).unwrap()[0]
    }

    #[test]
    fn identity_is_exact() {
        let src = numbered(3, 2);
        let (out, mask, off) = scale_rotate(&src, 3, 2, 0.0).unwrap();
        assert_eq!(out, src);
        assert_eq!(off, Point::new(0, 0));
        assert_eq!(mask.bounds().w, 3);
    }

    #[test]
    fn scaling_repeats_pixels() {
        let (out, _, _) = scale_rotate(&numbered(2, 1), 4, 2, 0.0).unwrap();
        assert_eq!(out.size(), Size::new(4, 2));
        assert_eq!([at(&out, 0, 0), at(&out, 1, 1), at(&out, 2, 0), at(&out, 3, 1)], [0, 0, 1, 1]);
    }

    #[test]
    fn quarter_turns_are_lossless() {
        // 0 1 2      3 0
        // 3 4 5  →   4 1   (clockwise)
        //            5 2
        let (out, mask, off) = scale_rotate(&numbered(3, 2), 3, 2, 90.0).unwrap();
        assert_eq!(out.size(), Size::new(2, 3));
        assert_eq!(off, Point::new(1, 0), "half a pixel off the odd side, rounded");
        assert_eq!([at(&out, 0, 0), at(&out, 1, 0), at(&out, 0, 2), at(&out, 1, 2)], [3, 0, 5, 2]);
        assert_eq!(mask.bounds().w * mask.bounds().h, 6);
        let (half, _, _) = scale_rotate(&numbered(3, 2), 3, 2, 180.0).unwrap();
        assert_eq!([at(&half, 0, 0), at(&half, 2, 1)], [5, 0]);
    }

    #[test]
    fn hostile_sizes_are_refused() {
        let src = numbered(2, 2);
        assert!(scale_rotate(&src, 0, 5, 0.0).is_err());
        assert!(scale_rotate(&src, 60_000, 60_000, 0.0).is_err());
        assert!(scale_rotate(&src, 4, 4, f32::NAN).is_ok());
    }
}
