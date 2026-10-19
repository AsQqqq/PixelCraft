//! The CPU compositor: flattens one frame of a sprite into an RGBA image.
//!
//! This is the reference ("oracle") renderer: simple, exact and testable. A faster GPU path may
//! come later and must match it pixel for pixel.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use pixelcraft_color::{ColorMode, Rgba8};
use pixelcraft_doc::Sprite;
use pixelcraft_geom::Point;
use pixelcraft_raster::{Image, RasterError};

/// Source-over of straight-alpha `src` (scaled by `opacity`) onto `dst`.
pub fn blend_normal(dst: Rgba8, src: Rgba8, opacity: u8) -> Rgba8 {
    let sa = u32::from(src.a) * u32::from(opacity) / 255;
    if sa == 0 {
        return dst;
    }
    let da = u32::from(dst.a);
    let out_a = sa + da * (255 - sa) / 255;
    if out_a == 0 {
        return Rgba8::TRANSPARENT;
    }
    let mix = |s: u8, d: u8| ((u32::from(s) * sa + u32::from(d) * da * (255 - sa) / 255) / out_a) as u8;
    Rgba8::new(mix(src.r, dst.r), mix(src.g, dst.g), mix(src.b, dst.b), out_a as u8)
}

/// Renders `frame` of `sprite` (visible layers, bottom to top) as an RGBA image of the canvas size.
pub fn render_frame(sprite: &Sprite, frame: usize) -> Result<Image, RasterError> {
    let mut out = Image::new(ColorMode::Rgba, sprite.size)?;
    for (li, layer) in sprite.layers.iter().enumerate() {
        if !layer.visible {
            continue;
        }
        let Some(cel) = sprite.cel(li, frame) else { continue };
        let opacity = (u32::from(layer.opacity) * u32::from(cel.opacity) / 255) as u8;
        let size = cel.image.size();
        for y in 0..size.h as i32 {
            for x in 0..size.w as i32 {
                let src = cel.image.rgba(Point::new(x, y), &sprite.palette);
                let at = cel.position.offset(x, y);
                let dst = out.rgba(at, &sprite.palette);
                let c = blend_normal(dst, src, opacity);
                out.put_pixel(at, &[c.r, c.g, c.b, c.a]);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelcraft_doc::Sprite;
    use pixelcraft_geom::Size;
    use std::sync::Arc;

    #[test]
    fn opaque_over_anything_is_the_source() {
        assert_eq!(blend_normal(Rgba8::WHITE, Rgba8::BLACK, 255), Rgba8::BLACK);
        assert_eq!(blend_normal(Rgba8::WHITE, Rgba8::BLACK, 0), Rgba8::WHITE);
        assert_eq!(blend_normal(Rgba8::TRANSPARENT, Rgba8::TRANSPARENT, 255), Rgba8::TRANSPARENT);
    }

    #[test]
    fn hidden_layers_do_not_render() {
        let mut s = Sprite::new(Size::new(2, 2), ColorMode::Rgba).unwrap();
        if let Some(cel) = s.cel_mut(0, 0) {
            Arc::make_mut(&mut cel.image).put_pixel(Point::new(0, 0), &[255, 0, 0, 255]);
        }
        let pal = s.palette.clone();
        assert_eq!(render_frame(&s, 0).unwrap().rgba(Point::new(0, 0), &pal), Rgba8::opaque(255, 0, 0));
        s.layers[0].visible = false;
        assert_eq!(render_frame(&s, 0).unwrap().rgba(Point::new(0, 0), &pal), Rgba8::TRANSPARENT);
        assert!(render_frame(&s, 99).is_ok());
    }
}
