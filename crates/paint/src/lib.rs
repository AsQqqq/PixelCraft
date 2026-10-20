//! Pixel-exact drawing algorithms for PixelCraft's tools.
//!
//! No anti-aliasing: every function here decides, pixel by pixel, which canvas pixels a tool
//! touches. [`apply`] writes them into a cel's image, skipping pixels outside the selection and
//! outside the image.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::collections::HashSet;

use pixelcraft_geom::{Point, Rect};
use pixelcraft_raster::mask::{ellipse_pixels, line_points, polygon_pixels};
use pixelcraft_raster::{Image, Mask};

/// The most points a single line may produce; longer requests are clipped to this many.
pub const MAX_LINE_POINTS: usize = 1 << 20;
/// The largest brush diameter.
pub const MAX_BRUSH: u32 = 64;
/// The most pixels one operation may touch.
pub const MAX_PIXELS: usize = 1 << 24;

/// The pixels of a 1-pixel line from `a` to `b`, both ends included (Bresenham).
pub fn line(a: Point, b: Point) -> Vec<Point> {
    line_points(a, b)
}

/// The offsets a round brush of `size` pixels covers around its centre pixel.
pub fn brush(size: u32) -> Vec<Point> {
    let n = size.clamp(1, MAX_BRUSH);
    if n == 1 {
        return vec![Point::new(0, 0)];
    }
    let half = (n / 2) as i32;
    ellipse_pixels(Rect::new(-half, -half, n, n), true)
}

/// Drops the corner pixel of every L-shaped step, so 1-pixel lines stay one pixel thick
/// (the "pixel-perfect" option).
pub fn pixel_perfect(points: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(points.len());
    for &p in points {
        if out.last() == Some(&p) {
            continue;
        }
        if let [.., a, b] = out[..] {
            let corner = (a.x - p.x).abs() == 1 && (a.y - p.y).abs() == 1 && (a.x == b.x || a.y == b.y);
            if corner {
                out.pop();
            }
        }
        out.push(p);
    }
    out
}

/// Every pixel a freehand stroke through `path` paints with a round brush of `size`.
pub fn stroke(path: &[Point], size: u32, perfect: bool) -> Vec<Point> {
    let mut centres: Vec<Point> = Vec::new();
    for (i, &p) in path.iter().enumerate() {
        let from = if i == 0 { p } else { path.get(i - 1).copied().unwrap_or(p) };
        for q in line(from, p) {
            if centres.last() != Some(&q) {
                centres.push(q);
            }
            if centres.len() >= MAX_LINE_POINTS {
                break;
            }
        }
    }
    if perfect && size <= 1 {
        centres = pixel_perfect(&centres);
    }
    stamp(&centres, size)
}

/// The brush stamped at every centre, without duplicates.
pub fn stamp(centres: &[Point], size: u32) -> Vec<Point> {
    let tip = brush(size);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for c in centres {
        for o in &tip {
            let p = c.offset(o.x, o.y);
            if seen.insert(p) {
                out.push(p);
                if out.len() >= MAX_PIXELS {
                    return out;
                }
            }
        }
    }
    out
}

/// A rectangle's outline (or all of it) between two corners, both included.
pub fn rectangle(a: Point, b: Point, filled: bool) -> Vec<Point> {
    let r = corners(a, b);
    let mut out = Vec::new();
    if (r.w as usize).saturating_mul(r.h as usize) > MAX_PIXELS {
        return out;
    }
    for y in r.y..r.y.saturating_add(r.h as i32) {
        for x in r.x..r.x.saturating_add(r.w as i32) {
            let edge = x == r.x || y == r.y || i64::from(x) == r.right() - 1 || i64::from(y) == r.bottom() - 1;
            if filled || edge {
                out.push(Point::new(x, y));
            }
        }
    }
    out
}

/// The ellipse inside the rectangle between two corners.
pub fn ellipse(a: Point, b: Point, filled: bool) -> Vec<Point> {
    let r = corners(a, b);
    if (r.w as usize).saturating_mul(r.h as usize) > MAX_PIXELS {
        return Vec::new();
    }
    ellipse_pixels(r, filled)
}

/// The rectangle with corners `a` and `b`, both included.
pub fn corners(a: Point, b: Point) -> Rect {
    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
    let w = (i64::from(x1) - i64::from(x0) + 1).clamp(0, i64::from(u32::MAX)) as u32;
    let h = (i64::from(y1) - i64::from(y0) + 1).clamp(0, i64::from(u32::MAX)) as u32;
    Rect::new(x0, y0, w, h)
}

/// A closed shape through `points`: its outline, or filled (contour and polygon tools).
pub fn polygon(points: &[Point], filled: bool, clip: Rect) -> Vec<Point> {
    if filled {
        return polygon_pixels(points, clip);
    }
    let mut closed = points.to_vec();
    if let Some(&first) = points.first() {
        closed.push(first);
    }
    stroke(&closed, 1, false)
}

/// Writes raw pixel `px` at canvas `points` into `img`, whose top-left sits at canvas `origin`.
/// Pixels outside `selection` (when there is one) or outside the image are left alone.
/// Returns how many pixels were written.
pub fn apply(img: &mut Image, origin: Point, points: &[Point], px: &[u8], selection: Option<&Mask>) -> usize {
    let mut n = 0;
    for &p in points {
        if selection.is_some_and(|m| !m.get(p)) {
            continue;
        }
        let local = Point::new(p.x.saturating_sub(origin.x), p.y.saturating_sub(origin.y));
        if img.pixel(local).is_some() {
            img.put_pixel(local, px);
            n += 1;
        }
    }
    n
}

/// Draws a 1-pixel line of raw pixel `px` (in the image's colour mode).
pub fn stroke_line(img: &mut Image, a: Point, b: Point, px: &[u8]) {
    for p in line(a, b) {
        img.put_pixel(p, px);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelcraft_color::ColorMode;
    use pixelcraft_geom::Size;

    #[test]
    fn line_includes_both_ends_and_is_connected() {
        let pts = line(Point::new(0, 0), Point::new(5, 2));
        assert_eq!(pts.first(), Some(&Point::new(0, 0)));
        assert_eq!(pts.last(), Some(&Point::new(5, 2)));
        assert!(pts.windows(2).all(|w| (w[0].x - w[1].x).abs() <= 1 && (w[0].y - w[1].y).abs() <= 1));
    }

    #[test]
    fn huge_lines_are_capped() {
        assert_eq!(line(Point::new(i32::MIN, 0), Point::new(i32::MAX, 0)).len(), MAX_LINE_POINTS);
    }

    #[test]
    fn brushes_grow_with_size() {
        assert_eq!(brush(1), vec![Point::new(0, 0)]);
        assert!(brush(3).len() >= 5 && brush(8).len() > brush(3).len());
        assert_eq!(brush(0).len(), 1);
        assert!(brush(u32::MAX).len() <= (MAX_BRUSH * MAX_BRUSH) as usize);
    }

    #[test]
    fn pixel_perfect_removes_l_corners() {
        let staircase = [Point::new(0, 0), Point::new(1, 0), Point::new(1, 1), Point::new(2, 1)];
        assert_eq!(pixel_perfect(&staircase), vec![Point::new(0, 0), Point::new(1, 1), Point::new(2, 1)]);
    }

    #[test]
    fn shapes_have_the_right_pixels() {
        assert_eq!(rectangle(Point::new(0, 0), Point::new(2, 2), false).len(), 8);
        assert_eq!(rectangle(Point::new(2, 2), Point::new(0, 0), true).len(), 9);
        assert!(!ellipse(Point::new(0, 0), Point::new(6, 4), false).is_empty());
        assert!(rectangle(Point::new(i32::MIN, i32::MIN), Point::new(i32::MAX, i32::MAX), true).is_empty());
    }

    #[test]
    fn apply_respects_the_selection() {
        let size = Size::new(4, 4);
        let mut img = Image::new(ColorMode::Indexed, size).unwrap();
        let sel = Mask::rect(size, Rect::new(0, 0, 2, 4)).unwrap();
        let n = apply(&mut img, Point::default(), &rectangle(Point::new(0, 0), Point::new(3, 3), true), &[7], Some(&sel));
        assert_eq!(n, 8);
        assert_eq!(img.pixel(Point::new(3, 3)), Some(&[0u8][..]));
    }
}
