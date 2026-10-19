//! The canvas viewport: zoom level on a fixed ladder, and pan.
//!
//! The ladder is the 25 levels measured in the target app (`plan/aseprite/zoom-and-navigation.md`):
//! `1/n` and `n` for n ∈ {1, 2, 3, 4, 5, 6, 8, 12, 16, 24, 32, 48, 64}. Zoom In/Out, the wheel and
//! the Zoom tool all step along it; digit keys jump to fixed levels. Zooming keeps the sprite pixel
//! under the pointer where it is.

use egui::{Pos2, Rect, Vec2};

/// Every zoom level, smallest first (screen pixels per sprite pixel).
pub const LADDER: [f32; 25] = [
    1.0 / 64.0,
    1.0 / 48.0,
    1.0 / 32.0,
    1.0 / 24.0,
    1.0 / 16.0,
    1.0 / 12.0,
    1.0 / 8.0,
    1.0 / 6.0,
    1.0 / 5.0,
    1.0 / 4.0,
    1.0 / 3.0,
    1.0 / 2.0,
    1.0,
    2.0,
    3.0,
    4.0,
    5.0,
    6.0,
    8.0,
    12.0,
    16.0,
    24.0,
    32.0,
    48.0,
    64.0,
];

pub const MIN: f32 = LADDER[0];
pub const MAX: f32 = LADDER[LADDER.len() - 1];

/// Levels the digit keys jump to: `` ` `` 50 %, `1` 100 %, `2` 200 %, `3` 400 %, `4` 800 %, `5` 1600 %, `6` 3200 %.
pub fn preset_for_digit(digit: u8) -> Option<f32> {
    match digit {
        0 => Some(0.5),
        1 => Some(1.0),
        2 => Some(2.0),
        3 => Some(4.0),
        4 => Some(8.0),
        5 => Some(16.0),
        6 => Some(32.0),
        _ => None,
    }
}

const EPS: f32 = 1e-4;

/// The next level up the ladder (or the top).
pub fn step_in(z: f32) -> f32 {
    LADDER.iter().copied().find(|&l| l > z * (1.0 + EPS)).unwrap_or(MAX)
}

/// The next level down the ladder (or the bottom).
pub fn step_out(z: f32) -> f32 {
    LADDER.iter().rev().copied().find(|&l| l < z * (1.0 - EPS)).unwrap_or(MIN)
}

/// The largest level at which a sprite of `sprite` pixels fits in `view` with `margin` on each side.
pub fn fit(sprite: Vec2, view: Vec2, margin: f32) -> f32 {
    let room = (view - Vec2::splat(2.0 * margin)).max(Vec2::splat(1.0));
    LADDER.iter().rev().copied().find(|&l| sprite.x * l <= room.x && sprite.y * l <= room.y).unwrap_or(MIN)
}

/// The status bar text: one decimal, like `100.0%` or `16.7%`.
pub fn label(z: f32) -> String {
    format!("{:.1}%", z * 100.0)
}

/// Where the sprite sits in the editor: zoom plus a pan offset of the sprite's centre from the
/// view's centre, in screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { zoom: 1.0, pan: Vec2::ZERO }
    }
}

impl Viewport {
    /// The sprite's rectangle on screen inside `view`.
    pub fn sprite_rect(&self, view: Rect, sprite: Vec2) -> Rect {
        Rect::from_center_size(view.center() + self.pan, sprite * self.zoom)
    }

    /// The sprite pixel under screen point `p` (may be outside the sprite).
    pub fn pixel_at(&self, view: Rect, sprite: Vec2, p: Pos2) -> (i32, i32) {
        let r = self.sprite_rect(view, sprite);
        let s = (p - r.min) / self.zoom;
        (s.x.floor() as i32, s.y.floor() as i32)
    }

    /// Changes the zoom, keeping the sprite point under `anchor` fixed on screen.
    pub fn zoom_to(&mut self, z: f32, view: Rect, sprite: Vec2, anchor: Pos2) {
        let z = z.clamp(MIN, MAX);
        let r = self.sprite_rect(view, sprite);
        let s = (anchor - r.min) / self.zoom;
        let new_min = anchor - s * z;
        self.pan = new_min + sprite * z / 2.0 - view.center();
        self.zoom = z;
    }

    /// Zoom to fit and centre.
    pub fn fit(&mut self, view: Rect, sprite: Vec2) {
        self.zoom = fit(sprite, view.size(), 16.0);
        self.pan = Vec2::ZERO;
    }

    /// Centre the sprite without changing the zoom (Scroll to center of canvas).
    pub fn center(&mut self) {
        self.pan = Vec2::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_matches_the_measured_levels() {
        let pct: Vec<String> = LADDER.iter().map(|&z| label(z)).collect();
        assert_eq!(pct.first().map(String::as_str), Some("1.6%"));
        assert!(pct.contains(&"16.7%".to_string()) && pct.contains(&"33.3%".to_string()));
        assert_eq!(pct.last().map(String::as_str), Some("6400.0%"));
        assert!(LADDER.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn stepping_walks_the_ladder_and_stops_at_the_ends() {
        assert_eq!(step_in(1.0), 2.0);
        assert_eq!(step_in(2.0), 3.0);
        assert_eq!(step_in(8.0), 12.0);
        assert_eq!(step_out(1.0), 0.5);
        assert_eq!(step_out(0.5), 1.0 / 3.0);
        assert_eq!(step_in(MAX), MAX);
        assert_eq!(step_out(MIN), MIN);
        // Off-ladder levels snap to the neighbour in the right direction.
        assert_eq!(step_in(2.5), 3.0);
        assert_eq!(step_out(2.5), 2.0);
    }

    #[test]
    fn fit_picks_the_largest_level_that_fits() {
        assert_eq!(fit(Vec2::new(32.0, 32.0), Vec2::new(800.0, 600.0), 16.0), 16.0);
        assert_eq!(fit(Vec2::new(1024.0, 1024.0), Vec2::new(800.0, 600.0), 16.0), 0.5);
        assert_eq!(fit(Vec2::new(1.0e6, 1.0e6), Vec2::new(10.0, 10.0), 16.0), MIN);
    }

    #[test]
    fn zoom_keeps_the_pixel_under_the_pointer() {
        let view = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let sprite = Vec2::new(32.0, 32.0);
        let mut vp = Viewport { zoom: 4.0, pan: Vec2::new(10.0, -20.0) };
        let p = Pos2::new(380.0, 260.0);
        let before = vp.pixel_at(view, sprite, p);
        vp.zoom_to(step_in(vp.zoom), view, sprite, p);
        assert_eq!(vp.zoom, 5.0);
        assert_eq!(vp.pixel_at(view, sprite, p), before);
    }
}
