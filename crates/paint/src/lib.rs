//! Pixel-exact drawing algorithms for PixelCraft's tools.
//!
//! No anti-aliasing: every function here decides, pixel by pixel, which pixels a tool touches.
//! Tools (pencil, eraser, line, rectangle, ellipse, bucket) build on these and write through
//! [`pixelcraft_raster::Image::put_pixel`], which ignores points outside the image.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use pixelcraft_geom::Point;
use pixelcraft_raster::Image;

/// The most points a single line may produce; longer requests are clipped to this many.
pub const MAX_LINE_POINTS: usize = 1 << 20;

/// The pixels of a 1-pixel line from `a` to `b`, both ends included (Bresenham).
pub fn line(a: Point, b: Point) -> Vec<Point> {
    let (x0, y0, x1, y1) = (i64::from(a.x), i64::from(a.y), i64::from(b.x), i64::from(b.y));
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut err) = (x0, y0, dx + dy);
    let mut out = Vec::new();
    while out.len() < MAX_LINE_POINTS {
        // x and y stay between the two i32 end points.
        out.push(Point::new(x as i32, y as i32));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    out
}

/// Draws a 1-pixel line of raw pixel `px` (in the image's colour mode) — the pencil stroke.
pub fn stroke_line(img: &mut Image, a: Point, b: Point, px: &[u8]) {
    for p in line(a, b) {
        img.put_pixel(p, px);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_includes_both_ends_and_is_connected() {
        let pts = line(Point::new(0, 0), Point::new(5, 2));
        assert_eq!(pts.first(), Some(&Point::new(0, 0)));
        assert_eq!(pts.last(), Some(&Point::new(5, 2)));
        assert_eq!(pts.len(), 6);
        assert!(pts.windows(2).all(|w| (w[0].x - w[1].x).abs() <= 1 && (w[0].y - w[1].y).abs() <= 1));
    }

    #[test]
    fn huge_lines_are_capped() {
        assert_eq!(line(Point::new(i32::MIN, 0), Point::new(i32::MAX, 0)).len(), MAX_LINE_POINTS);
    }
}
