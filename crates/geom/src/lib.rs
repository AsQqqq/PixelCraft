//! PixelCraft geometry on the pixel grid: integer points, sizes and rectangles.
//!
//! Pixel art lives on whole pixels, so everything here is `i32`. Arithmetic saturates: coordinates
//! come from files, params and pointer input, and must never overflow into a panic.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x.saturating_add(dx), self.y.saturating_add(dy))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub const fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    pub fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }
    /// Pixel count, or `None` when it doesn't fit in `usize`.
    pub fn area(self) -> Option<usize> {
        usize::try_from(self.w).ok()?.checked_mul(usize::try_from(self.h).ok()?)
    }
}

/// A rectangle with its origin at the top-left corner; `x + w` is exclusive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
    pub const fn from_size(size: Size) -> Self {
        Self::new(0, 0, size.w, size.h)
    }
    pub fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }
    pub fn right(self) -> i64 {
        i64::from(self.x) + i64::from(self.w)
    }
    pub fn bottom(self) -> i64 {
        i64::from(self.y) + i64::from(self.h)
    }
    pub fn contains(self, p: Point) -> bool {
        i64::from(p.x) >= i64::from(self.x) && i64::from(p.x) < self.right() && i64::from(p.y) >= i64::from(self.y) && i64::from(p.y) < self.bottom()
    }
    /// The overlap of two rectangles; empty when they don't meet.
    pub fn intersect(self, o: Rect) -> Rect {
        let x0 = i64::from(self.x).max(i64::from(o.x));
        let y0 = i64::from(self.y).max(i64::from(o.y));
        let x1 = self.right().min(o.right());
        let y1 = self.bottom().min(o.bottom());
        if x1 <= x0 || y1 <= y0 {
            return Rect::default();
        }
        // x0/y0 come from i32 values and the extents from u32 widths, so these fit.
        Rect::new(x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_overlapping_and_disjoint() {
        let a = Rect::new(0, 0, 10, 10);
        assert_eq!(a.intersect(Rect::new(5, 5, 10, 10)), Rect::new(5, 5, 5, 5));
        assert!(a.intersect(Rect::new(20, 20, 1, 1)).is_empty());
    }

    #[test]
    fn extremes_never_overflow() {
        let r = Rect::new(i32::MAX, i32::MAX, u32::MAX, u32::MAX);
        assert!(r.contains(Point::new(i32::MAX, i32::MAX)));
        assert_eq!(Point::new(i32::MAX, 0).offset(1, 0).x, i32::MAX);
        assert!(Size::new(u32::MAX, u32::MAX).area().is_some() || usize::BITS < 64);
    }
}
