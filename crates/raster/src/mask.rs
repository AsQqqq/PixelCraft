//! Selection masks: which pixels of the canvas are selected.
//!
//! A [`Mask`] is canvas-sized, one bool per pixel. Shapes (rectangle, ellipse, polygon, magic wand)
//! build masks; [`Mask::combine`] applies the marquee modes (replace, add, subtract, intersect);
//! [`Mask::edges`] lists the boundary segments the UI draws as marching ants.

use pixelcraft_geom::{Point, Rect, Size};
use serde::{Deserialize, Serialize};

use crate::{Image, RasterError};

/// How a new selection shape combines with the current selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectMode {
    #[default]
    Replace,
    Add,
    Subtract,
    Intersect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mask {
    size: Size,
    bits: Vec<bool>,
}

/// A unit-length piece of the selection's outline, between two pixel corners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: Point,
    pub to: Point,
}

impl Mask {
    /// An empty mask the size of the canvas.
    pub fn new(size: Size) -> Result<Self, RasterError> {
        if size.is_empty() || size.w > crate::MAX_SIDE || size.h > crate::MAX_SIDE {
            return Err(RasterError::BadSize(size.w, size.h));
        }
        let len = size.area().ok_or(RasterError::BadSize(size.w, size.h))?;
        Ok(Self { size, bits: vec![false; len] })
    }

    pub fn size(&self) -> Size {
        self.size
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        if x < 0 || y < 0 || x >= i64::from(self.size.w) || y >= i64::from(self.size.h) {
            return None;
        }
        usize::try_from(y * i64::from(self.size.w) + x).ok()
    }

    pub fn get(&self, p: Point) -> bool {
        self.index(i64::from(p.x), i64::from(p.y)).and_then(|i| self.bits.get(i)).copied().unwrap_or(false)
    }

    pub fn set(&mut self, p: Point, on: bool) {
        if let Some(b) = self.index(i64::from(p.x), i64::from(p.y)).and_then(|i| self.bits.get_mut(i)) {
            *b = on;
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.bits.iter().any(|&b| b)
    }

    /// Every pixel selected.
    pub fn all(size: Size) -> Result<Self, RasterError> {
        let mut m = Self::new(size)?;
        m.bits.fill(true);
        Ok(m)
    }

    pub fn invert(&mut self) {
        for b in &mut self.bits {
            *b = !*b;
        }
    }

    /// The smallest rectangle holding every selected pixel (empty when nothing is selected).
    pub fn bounds(&self) -> Rect {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..self.size.h {
            for x in 0..self.size.w {
                if self.get(Point::new(x as i32, y as i32)) {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        if x1 == 0 { Rect::default() } else { Rect::new(x0 as i32, y0 as i32, x1 - x0, y1 - y0) }
    }

    /// Selects the rectangle `r` (clipped to the canvas).
    pub fn rect(size: Size, r: Rect) -> Result<Self, RasterError> {
        let mut m = Self::new(size)?;
        let r = r.intersect(Rect::from_size(size));
        for y in r.y..r.y.saturating_add(r.h as i32) {
            for x in r.x..r.x.saturating_add(r.w as i32) {
                m.set(Point::new(x, y), true);
            }
        }
        Ok(m)
    }

    /// Selects the ellipse inscribed in `r`.
    pub fn ellipse(size: Size, r: Rect) -> Result<Self, RasterError> {
        let mut m = Self::new(size)?;
        for p in ellipse_pixels(r, true) {
            m.set(p, true);
        }
        Ok(m)
    }

    /// Selects the inside of the closed polygon through `points` (pixel centres, even-odd rule),
    /// plus its outline.
    pub fn polygon(size: Size, points: &[Point]) -> Result<Self, RasterError> {
        let mut m = Self::new(size)?;
        for p in polygon_pixels(points, Rect::from_size(size)) {
            m.set(p, true);
        }
        Ok(m)
    }

    /// Combines `shape` into `self` with `mode`.
    pub fn combine(&mut self, shape: &Mask, mode: SelectMode) {
        if shape.size != self.size {
            return;
        }
        for (a, &b) in self.bits.iter_mut().zip(&shape.bits) {
            *a = match mode {
                SelectMode::Replace => b,
                SelectMode::Add => *a || b,
                SelectMode::Subtract => *a && !b,
                SelectMode::Intersect => *a && b,
            };
        }
    }

    /// The selection's outline as unit edges between pixel corners (for marching ants).
    pub fn edges(&self) -> Vec<Edge> {
        let mut out = Vec::new();
        let (w, h) = (self.size.w as i32, self.size.h as i32);
        for y in 0..=h {
            for x in 0..=w {
                let here = self.get(Point::new(x, y));
                // Horizontal edge above (x, y): between (x, y-1) and (x, y).
                if x < w && here != self.get(Point::new(x, y - 1)) {
                    out.push(Edge { from: Point::new(x, y), to: Point::new(x + 1, y) });
                }
                // Vertical edge left of (x, y): between (x-1, y) and (x, y).
                if y < h && here != self.get(Point::new(x - 1, y)) {
                    out.push(Edge { from: Point::new(x, y), to: Point::new(x, y + 1) });
                }
            }
        }
        out
    }
}

/// The pixels of the ellipse inscribed in `r`: filled, or only the outline.
pub fn ellipse_pixels(r: Rect, filled: bool) -> Vec<Point> {
    let mut out = Vec::new();
    if r.is_empty() {
        return out;
    }
    let (w, h) = (f64::from(r.w), f64::from(r.h));
    let (cx, cy) = (f64::from(r.x) + w / 2.0, f64::from(r.y) + h / 2.0);
    let (rx, ry) = (w / 2.0, h / 2.0);
    let inside = |x: i64, y: i64| {
        let dx = (x as f64 + 0.5 - cx) / rx;
        let dy = (y as f64 + 0.5 - cy) / ry;
        dx * dx + dy * dy <= 1.0 + 1e-9
    };
    let (x0, y0) = (i64::from(r.x), i64::from(r.y));
    let (x1, y1) = (r.right(), r.bottom());
    // Bounded by the rectangle, which callers clip to sane sizes.
    if (x1 - x0).saturating_mul(y1 - y0) > 1 << 24 {
        return out;
    }
    for y in y0..y1 {
        for x in x0..x1 {
            if !inside(x, y) {
                continue;
            }
            let edge = !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
            if filled || edge {
                out.push(Point::new(x as i32, y as i32));
            }
        }
    }
    // A 1-pixel-wide or -tall ellipse is the whole rectangle.
    if out.is_empty() {
        for y in y0..y1 {
            for x in x0..x1 {
                out.push(Point::new(x as i32, y as i32));
            }
        }
    }
    out
}

/// Pixels inside the closed polygon through `points` (even-odd, sampled at pixel centres) and on
/// its outline, clipped to `clip`.
pub fn polygon_pixels(points: &[Point], clip: Rect) -> Vec<Point> {
    let mut out = Vec::new();
    if points.is_empty() {
        return out;
    }
    let bounds = {
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for p in points {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        Rect::new(x0, y0, (i64::from(x1) - i64::from(x0) + 1) as u32, (i64::from(y1) - i64::from(y0) + 1) as u32).intersect(clip)
    };
    for y in bounds.y..bounds.y.saturating_add(bounds.h as i32) {
        let py = f64::from(y) + 0.5;
        let mut xs: Vec<f64> = Vec::new();
        for (i, a) in points.iter().enumerate() {
            let b = points.get((i + 1) % points.len()).unwrap_or(a);
            let (ay, by) = (f64::from(a.y) + 0.5, f64::from(b.y) + 0.5);
            if (ay <= py && by > py) || (by <= py && ay > py) {
                let t = (py - ay) / (by - ay);
                xs.push(f64::from(a.x) + 0.5 + t * f64::from(b.x - a.x));
            }
        }
        xs.sort_by(|a, b| a.total_cmp(b));
        for [a, b] in xs.as_chunks::<2>().0 {
            let (from, to) = ((a - 0.5).ceil() as i64, (b - 0.5).floor() as i64);
            for x in from.max(i64::from(bounds.x))..=to.min(bounds.right() - 1) {
                out.push(Point::new(x as i32, y));
            }
        }
    }
    // The outline, so thin shapes still select something.
    for (i, a) in points.iter().enumerate() {
        let b = points.get((i + 1) % points.len()).unwrap_or(a);
        for p in line_points(*a, *b) {
            if clip.contains(p) {
                out.push(p);
            }
        }
    }
    out
}

/// A 1-pixel Bresenham line, both ends included (capped at 2²⁰ points).
pub fn line_points(a: Point, b: Point) -> Vec<Point> {
    let (x0, y0, x1, y1) = (i64::from(a.x), i64::from(a.y), i64::from(b.x), i64::from(b.y));
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut err) = (x0, y0, dx + dy);
    let mut out = Vec::new();
    while out.len() < 1 << 20 {
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

/// The magic wand: pixels whose colour is within `tolerance` (0–255 per channel) of the pixel at
/// `seed`; only those connected to it (4-way) when `contiguous`.
pub fn magic_wand(img: &Image, palette: &pixelcraft_color::Palette, seed: Point, tolerance: u8, contiguous: bool) -> Result<Mask, RasterError> {
    let size = img.size();
    let mut m = Mask::new(size)?;
    if !Rect::from_size(size).contains(seed) {
        return Ok(m);
    }
    let target = img.rgba(seed, palette);
    let close = |p: Point| {
        let c = img.rgba(p, palette);
        let d = |a: u8, b: u8| a.abs_diff(b) <= tolerance;
        d(c.r, target.r) && d(c.g, target.g) && d(c.b, target.b) && d(c.a, target.a)
    };
    if !contiguous {
        for y in 0..size.h as i32 {
            for x in 0..size.w as i32 {
                let p = Point::new(x, y);
                if close(p) {
                    m.set(p, true);
                }
            }
        }
        return Ok(m);
    }
    let mut stack = vec![seed];
    m.set(seed, true);
    while let Some(p) = stack.pop() {
        for n in [p.offset(1, 0), p.offset(-1, 0), p.offset(0, 1), p.offset(0, -1)] {
            if Rect::from_size(size).contains(n) && !m.get(n) && close(n) {
                m.set(n, true);
                stack.push(n);
            }
        }
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_combine() {
        let size = Size::new(8, 8);
        let mut m = Mask::rect(size, Rect::new(0, 0, 4, 4)).unwrap();
        m.combine(&Mask::rect(size, Rect::new(2, 2, 4, 4)).unwrap(), SelectMode::Add);
        assert_eq!(m.bounds(), Rect::new(0, 0, 6, 6));
        m.combine(&Mask::rect(size, Rect::new(0, 0, 6, 3)).unwrap(), SelectMode::Subtract);
        assert!(!m.get(Point::new(1, 1)) && m.get(Point::new(3, 3)));
        m.combine(&Mask::rect(size, Rect::new(3, 3, 1, 1)).unwrap(), SelectMode::Intersect);
        assert_eq!(m.bounds(), Rect::new(3, 3, 1, 1));
    }

    #[test]
    fn a_single_pixel_has_four_edges() {
        let m = Mask::rect(Size::new(4, 4), Rect::new(1, 1, 1, 1)).unwrap();
        assert_eq!(m.edges().len(), 4);
        assert!(Mask::new(Size::new(4, 4)).unwrap().edges().is_empty());
    }

    #[test]
    fn shapes_stay_inside_the_canvas() {
        let size = Size::new(10, 10);
        let e = Mask::ellipse(size, Rect::new(-5, -5, 30, 30)).unwrap();
        assert!(!e.is_empty());
        let p = Mask::polygon(size, &[Point::new(0, 0), Point::new(9, 0), Point::new(0, 9)]).unwrap();
        assert!(p.get(Point::new(1, 1)) && !p.get(Point::new(9, 9)));
        assert!(Mask::polygon(size, &[]).unwrap().is_empty());
    }

    #[test]
    fn wand_selects_the_connected_colour() {
        let mut img = Image::new(pixelcraft_color::ColorMode::Rgba, Size::new(4, 1)).unwrap();
        img.put_pixel(Point::new(2, 0), &[255, 0, 0, 255]);
        let pal = pixelcraft_color::Palette::default_16();
        let m = magic_wand(&img, &pal, Point::new(0, 0), 0, true).unwrap();
        assert!(m.get(Point::new(1, 0)) && !m.get(Point::new(2, 0)) && !m.get(Point::new(3, 0)));
        let m = magic_wand(&img, &pal, Point::new(0, 0), 0, false).unwrap();
        assert!(m.get(Point::new(3, 0)));
    }
}
