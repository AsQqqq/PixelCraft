//! A composited frame as GPU textures. The GPU caps a texture's side (`max_texture_side`: 8192
//! or 16384 on most desktops, 4096 on some WebGL, 2048 in tests) and egui panics past it, so an
//! image bigger than that — a photo — is cut into tiles no larger than the cap, each drawn into
//! its share of the sprite's rectangle.

use egui::{Color32, Rect, pos2, vec2};
use pixelcraft_engine::raster::Image;

#[derive(Default)]
pub struct Tiles {
    /// Each texture and the sprite pixels it covers (x, y, w, h).
    tiles: Vec<(egui::TextureHandle, [usize; 4])>,
    size: [usize; 2],
}

impl Tiles {
    /// Uploads `img` (RGBA, unpremultiplied) as textures named `name`; pixels stay sharp when
    /// magnified and are filtered when shrunk.
    pub fn upload(ctx: &egui::Context, name: &str, img: &Image) -> Self {
        let options = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Linear,
            ..egui::TextureOptions::NEAREST
        };
        let side = ctx.input(|i| i.max_texture_side).max(64);
        let (w, h) = (img.size().w as usize, img.size().h as usize);
        let bytes = img.bytes();
        let mut tiles = Vec::new();
        for y0 in (0..h).step_by(side) {
            for x0 in (0..w).step_by(side) {
                let (tw, th) = (side.min(w - x0), side.min(h - y0));
                let mut rgba = Vec::with_capacity(tw * th * 4);
                for y in y0..y0 + th {
                    let start = (y * w + x0) * 4;
                    if let Some(row) = bytes.get(start..start + tw * 4) {
                        rgba.extend_from_slice(row);
                    }
                }
                if rgba.len() != tw * th * 4 {
                    continue;
                }
                let color = egui::ColorImage::from_rgba_unmultiplied([tw, th], &rgba);
                tiles.push((ctx.load_texture(format!("{name}-{x0}-{y0}"), color, options), [x0, y0, tw, th]));
            }
        }
        Self { tiles, size: [w, h] }
    }

    /// Draws the image stretched over `r`, the sprite's rectangle on screen.
    pub fn paint(&self, painter: &egui::Painter, r: Rect) {
        let [w, h] = self.size;
        if w == 0 || h == 0 {
            return;
        }
        let k = vec2(r.width() / w as f32, r.height() / h as f32);
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        for (tex, [x, y, tw, th]) in &self.tiles {
            let min = r.min + vec2(*x as f32 * k.x, *y as f32 * k.y);
            let tile = Rect::from_min_size(min, vec2(*tw as f32 * k.x, *th as f32 * k.y));
            if painter.clip_rect().intersects(tile) {
                painter.image(tex.id(), tile, uv, Color32::WHITE);
            }
        }
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.tiles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelcraft_engine::color::ColorMode;
    use pixelcraft_engine::geom::Size;

    #[test]
    fn big_images_are_cut_at_the_texture_limit() {
        let ctx = egui::Context::default();
        ctx.input_mut(|i| i.max_texture_side = 100);
        let img = Image::new(ColorMode::Rgba, Size::new(250, 100)).unwrap();
        let tiles = Tiles::upload(&ctx, "t", &img);
        assert_eq!(tiles.count(), 3);
        assert_eq!(tiles.tiles.iter().map(|(_, r)| *r).collect::<Vec<_>>(), vec![[0, 0, 100, 100], [100, 0, 100, 100], [200, 0, 50, 100]]);
        let small = Tiles::upload(&ctx, "s", &Image::new(ColorMode::Rgba, Size::new(32, 32)).unwrap());
        assert_eq!(small.count(), 1);
    }
}
